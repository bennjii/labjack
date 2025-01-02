use std::{
    borrow::BorrowMut,
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
};
use std::collections::HashSet;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::FromPrimitive;
use crate::prelude::modbus::ReadFunction;
use super::{
    Error, ExceptionCode, Function, ModbusFeedbackFunction, Reason, Transport, MODBUS_HEADER_SIZE,
    MODBUS_MAX_PACKET_SIZE, MODBUS_PROTOCOL_TCP,
};

/// As referenced in the LabJack manual fields documentation for ModBus messages,
/// the UnitID field is not used (as bridging is not used). Therefore, the default
/// value is suggested to be the u8 literal, 1. Alternatively, `0b00000001`.
///
/// Referenced Documentation: [LabJack Modbus Protocol Details: Fields](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-Fields).
const BASE_UNIT_ID: u8 = 1;

/// The base transaction ID. We use this value to identify a unique transaction,
/// such that the LabJack will relay this value back to us.
///
/// We have [`u16::MAX`] (or 65535) values. The use of values is implementation-dependent
/// but often uses a cycle to perform a checked addition to the existing transaction id
/// for each new message.
///
/// Referenced Documentation: [LabJack Modbus Protocol Details: Fields](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-Fields).
const STARTING_TRANSACTION_ID: u16 = 1;

/// Ephemeral structure created from the transport to compose messages. It's internal state is
/// only of a mutable extension of the [`TcpTransport`] explicitly only containing domain-specific
/// information with an emphasis on which properties can be mutated in the base transport.
///
/// It is used to compose messages for use over modbus.
pub struct TcpCompositor<'a> {
    transaction_id: &'a mut u16,
    unit_id: u8,
}

// TODO: Redo the responsibilities of the transaction id here...

pub struct TcpTransport {
    transaction_id: u16,
    unit_id: u8,
    stream: TcpStream,

    /// A hashset of existing transactions to indicate which values
    /// the transaction_id can take. When it's length is equal to
    /// [`u16::MAX`], no more transactions can be made. It is key
    /// that upon the completion of a transaction, it's identifier
    /// is removed from this set.
    existing_transactions: HashSet<u16>
}

impl TcpTransport {
    pub fn new(stream: TcpStream) -> TcpTransport {
        TcpTransport {
            unit_id: BASE_UNIT_ID,
            transaction_id: STARTING_TRANSACTION_ID,

            stream,
            existing_transactions: HashSet::new()
        }
    }

    fn compositor(&mut self) -> TcpCompositor {
        TcpCompositor {
            transaction_id: &mut self.transaction_id,
            unit_id: self.unit_id,
        }
    }

    fn validate_response_header(req: &Header, resp: &Header) -> Result<(), Error> {
        if req.transaction_id != resp.transaction_id || resp.protocol_id != MODBUS_PROTOCOL_TCP {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    fn validate_response_code(req: &[u8], res: &[u8]) -> Result<(), Error> {
        let req_code = *req.get(7).ok_or(Error::InvalidResponse)?;
        let res_code = *res.get(7).ok_or(Error::InvalidResponse)?;

        match res_code {
            code if code == req_code + 0x80 => {
                let exception = *res.get(8).ok_or(Error::InvalidResponse)?;
                match ExceptionCode::from_u8(exception) {
                    Some(code) => Err(Error::Exception(code)),
                    None => Err(Error::InvalidResponse),
                }
            }
            code if code == req_code => Ok(()),
            _ => Err(Error::InvalidResponse),
        }
    }

    fn get_reply_data(reply: &[u8], expected_bytes: usize) -> Result<&[u8], Error> {
        let given_response_length = *reply
            .get(8)
            .ok_or(Error::InvalidData(Reason::UnexpectedReplySize))?
            as usize;
        let reply_length_does_not_match = reply.len() != MODBUS_HEADER_SIZE + expected_bytes + 2;

        if given_response_length != expected_bytes || reply_length_does_not_match {
            return Err(Error::InvalidData(Reason::UnexpectedReplySize));
        }

        let reply_data = reply
            .get(MODBUS_HEADER_SIZE + 2..)
            .ok_or(Error::InvalidData(Reason::UnexpectedReplySize))?;

        Ok(reply_data)
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.stream.shutdown(Shutdown::Both).map_err(Error::Io)
    }
}

#[derive(Debug, PartialEq)]
pub struct Header {
    transaction_id: u16,
    protocol_id: u16,
    length: u16,
    unit_id: u8,
}

impl Header {
    fn new(transport: &mut TcpCompositor, len: u16) -> Header {
        Header {
            transaction_id: *transport.new_tid(),
            protocol_id: MODBUS_PROTOCOL_TCP,
            length: len - MODBUS_HEADER_SIZE as u16,
            unit_id: transport.unit_id,
        }
    }

    fn pack(&self) -> Result<Vec<u8>, Error> {
        let mut buff = vec![];
        buff.write_u16::<BigEndian>(self.transaction_id)?;
        buff.write_u16::<BigEndian>(self.protocol_id)?;
        buff.write_u16::<BigEndian>(self.length)?;
        buff.write_u8(self.unit_id)?;
        Ok(buff)
    }

    fn unpack(buff: &[u8]) -> Result<Header, Error> {
        let mut rdr = io::Cursor::new(buff);
        Ok(Header {
            transaction_id: rdr.read_u16::<BigEndian>()?,
            protocol_id: rdr.read_u16::<BigEndian>()?,
            length: rdr.read_u16::<BigEndian>()?,
            unit_id: rdr.read_u8()?,
        })
    }
}

impl<'a> TcpCompositor<'a> {
    pub fn new(transaction_id: &'a mut u16, unit_id: u8) -> TcpCompositor<'a> {
        TcpCompositor {
            transaction_id,
            unit_id,
        }
    }

    fn new_tid(&mut self) -> &u16 {
        *self.transaction_id = self.transaction_id.wrapping_add(1);
        self.transaction_id
    }

    pub fn compose_read(&mut self, function: &ReadFunction) -> Result<(Vec<u8>, Header, usize), Error> {
        let (addr, count, expected_bytes) = match *function {
            ReadFunction::HoldingRegisters(a, c)
            | ReadFunction::InputRegisters(a, c) => {
                (a, c, 2 * c as usize)
            }
        };

        if count < 1 {
            return Err(Error::InvalidData(Reason::RecvBufferEmpty));
        }

        if count as usize > MODBUS_MAX_PACKET_SIZE {
            return Err(Error::InvalidData(Reason::UnexpectedReplySize));
        }

        // The length in a feedback function might be different if
        // using a different frame type.
        let header = Header::new(self, MODBUS_HEADER_SIZE as u16 + 6u16);
        let mut buff = header.pack()?;

        buff.write_u8(function.code())?;

        buff.write_u16::<BigEndian>(addr)?;
        buff.write_u16::<BigEndian>(count)?;

        Ok((buff, header, expected_bytes))
    }

    pub fn compose_write(&mut self, buf: &mut [u8]) -> Result<(Vec<u8>, Header), Error> {
        if buf.is_empty() {
            return Err(Error::InvalidData(Reason::SendBufferEmpty));
        }

        if buf.len() > MODBUS_MAX_PACKET_SIZE {
            return Err(Error::InvalidData(Reason::SendBufferTooBig));
        }

        let header = Header::new(self, buf.len() as u16 + 1u16);
        let head_buff = header.pack()?;
        {
            let mut start = io::Cursor::new(buf.borrow_mut());
            start.write_all(&head_buff)?;
        }

        Ok((head_buff, header))
    }

    pub fn compose_feedback(
        &mut self,
        fns: &[ModbusFeedbackFunction],
    ) -> Result<(Vec<u8>, Header, usize), Error> {
        let mut read_return_size = 0;

        // Must account for unit ID and function ID (2 bytes) + base header size
        let composed_size = fns.iter().fold(MODBUS_HEADER_SIZE + 2, |acc, f| match f {
            ModbusFeedbackFunction::ReadRegisters(_, _) => {
                read_return_size += 2;
                acc + 4
            }
            ModbusFeedbackFunction::WriteRegisters(_, values) => acc + 4 + values.len(),
        });

        let header = Header::new(self, composed_size as u16);
        let mut buff = header.pack()?;

        buff.write_u8(Function::Feedback(fns).code())?;

        for frame in fns {
            buff.write_u8(frame.code())?;

            match frame {
                ModbusFeedbackFunction::ReadRegisters(addr, quant) => {
                    buff.write_u16::<BigEndian>(*addr)?;
                    buff.write_u8(*quant)?;
                }
                ModbusFeedbackFunction::WriteRegisters(addr, values) => {
                    buff.write_u16::<BigEndian>(*addr)?;
                    buff.write_u8(values.len() as u8)?;
                    for v in *values {
                        buff.write_u8(*v)?;
                    }
                }
            }
        }

        Ok((buff, header, 7 + read_return_size))
    }
}

impl Transport for TcpTransport {
    type Error = Error;

    fn write(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let (buf, header) = self.compositor().compose_write(buf)?;

        match self.stream.write_all(&buf) {
            Ok(_s) => {
                let reply = &mut [0; 12];
                match self.stream.read(reply) {
                    Ok(_s) => {
                        let resp_hd = Header::unpack(reply)?;
                        TcpTransport::validate_response_header(&header, &resp_hd)?;
                        TcpTransport::validate_response_code(buf.as_slice(), reply)
                    }
                    Err(e) => Err(Error::Io(e)),
                }
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn read(&mut self, function: &super::ReadFunction) -> Result<Box<[u8]>, Self::Error> {
        let (buf, header, expected_bytes) = self.compositor().compose_read(function)?;
        let mut reply = vec![0; MODBUS_HEADER_SIZE + expected_bytes + 2].into_boxed_slice();

        self.stream.write_all(&buf).map_err(Error::Io)?;
        self.stream.read(&mut reply).map_err(Error::Io)?;

        let reply_header_raw = &reply
            .get(..MODBUS_HEADER_SIZE)
            .ok_or(Error::InvalidResponse)?;
        let resp_hd = Header::unpack(reply_header_raw)?;

        TcpTransport::validate_response_header(&header, &resp_hd)?;
        TcpTransport::validate_response_code(&buf, &reply)?;
        TcpTransport::get_reply_data(&reply, expected_bytes).map(Box::from)
    }
}
