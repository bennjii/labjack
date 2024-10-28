use std::{
    borrow::BorrowMut,
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::FromPrimitive;

use super::{
    Client, Error, ExceptionCode, Function, ModbusFeedbackFunction, Reason, Transport,
    MODBUS_HEADER_SIZE, MODBUS_MAX_PACKET_SIZE, MODBUS_PROTOCOL_TCP,
};

pub struct TcpCompositor<'a> {
    transaction_id: &'a mut u16,
    unit_id: u8,
}

pub struct TcpTransport {
    tid: u16,
    uid: u8,
    stream: TcpStream,
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

impl TcpTransport {
    fn compositor(&mut self) -> TcpCompositor {
        TcpCompositor {
            transaction_id: &mut self.tid,
            unit_id: self.uid,
        }
    }

    fn validate_response_header(req: &Header, resp: &Header) -> Result<(), Error> {
        if req.transaction_id != resp.transaction_id || resp.protocol_id != MODBUS_PROTOCOL_TCP {
            Err(Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    fn validate_response_code(req: &[u8], resp: &[u8]) -> Result<(), Error> {
        if req[7] + 0x80 == resp[7] {
            match ExceptionCode::from_u8(resp[8]) {
                Some(code) => Err(Error::Exception(code)),
                None => Err(Error::InvalidResponse),
            }
        } else if req[7] == resp[7] {
            Ok(())
        } else {
            Err(Error::InvalidResponse)
        }
    }

    fn get_reply_data(reply: &[u8], expected_bytes: usize) -> Result<&[u8], Error> {
        if reply[8] as usize != expected_bytes
            || reply.len() != MODBUS_HEADER_SIZE + expected_bytes + 2
        {
            Err(Error::InvalidData(Reason::UnexpectedReplySize))
        } else {
            Ok(&reply[MODBUS_HEADER_SIZE + 2..])
        }
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.stream.shutdown(Shutdown::Both).map_err(Error::Io)
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

    pub fn compose_read(&mut self, function: &Function) -> Result<(Vec<u8>, Header, usize), Error> {
        let (addr, count, expected_bytes) = match *function {
            Function::ReadHoldingRegisters(a, c) | Function::ReadInputRegisters(a, c) => {
                (a, c, 2 * c as usize)
            }
            _ => return Err(Error::InvalidFunction),
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
    type Error = crate::prelude::modbus::Error;

    fn read(&mut self, function: &super::Function) -> Result<Box<[u8]>, Self::Error> {
        let (buf, header, expected_bytes) = self.compositor().compose_read(function)?;

        match self.stream.write_all(&buf) {
            Ok(_s) => {
                let mut reply = vec![0; MODBUS_HEADER_SIZE + expected_bytes + 2].into_boxed_slice();
                match self.stream.read(&mut reply) {
                    Ok(_s) => {
                        let resp_hd = Header::unpack(&reply[..MODBUS_HEADER_SIZE])?;
                        TcpTransport::validate_response_header(&header, &resp_hd)?;
                        TcpTransport::validate_response_code(&buf, &reply)?;
                        TcpTransport::get_reply_data(&reply, expected_bytes).map(Box::from)
                    }
                    Err(e) => Err(Error::Io(e)),
                }
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

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
}

impl Client for TcpTransport {}
