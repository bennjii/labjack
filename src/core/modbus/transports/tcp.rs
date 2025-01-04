use enum_primitive::FromPrimitive;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};

use crate::prelude::*;

pub const MODBUS_PROTOCOL_TCP: u16 = 0x0000;

pub const MODBUS_TCP_DEFAULT_PORT: u16 = 502;

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
    existing_transactions: HashSet<u16>,
}

impl TcpTransport {
    pub fn new(stream: TcpStream) -> TcpTransport {
        TcpTransport {
            unit_id: BASE_UNIT_ID,
            transaction_id: STARTING_TRANSACTION_ID,

            stream,
            existing_transactions: HashSet::new(),
        }
    }

    fn compositor(&mut self) -> Compositor {
        Compositor {
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

impl Transport for TcpTransport {
    type Error = Error;

    fn write(&mut self, function: &WriteFunction) -> Result<(), Self::Error> {
        let ComposedMessage {
            content, header, ..
        } = self.compositor().compose_write(function)?;

        match self.stream.write_all(&content) {
            Ok(_s) => {
                let reply = &mut [0; 12];
                match self.stream.read(reply) {
                    Ok(_s) => {
                        let resp_hd = Header::unpack(reply)?;
                        TcpTransport::validate_response_header(&header, &resp_hd)?;
                        TcpTransport::validate_response_code(content.as_slice(), reply)
                    }
                    Err(e) => Err(Error::Io(e)),
                }
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn read(&mut self, function: &ReadFunction) -> Result<Box<[u8]>, Self::Error> {
        let ComposedMessage {
            content,
            header,
            expected_bytes,
        } = self.compositor().compose_read(function)?;
        let mut reply = vec![0; MODBUS_HEADER_SIZE + expected_bytes + 2].into_boxed_slice();

        self.stream.write_all(&content).map_err(Error::Io)?;
        self.stream.read(&mut reply).map_err(Error::Io)?;

        let reply_header_raw = &reply
            .get(..MODBUS_HEADER_SIZE)
            .ok_or(Error::InvalidResponse)?;
        let resp_hd = Header::unpack(reply_header_raw)?;

        TcpTransport::validate_response_header(&header, &resp_hd)?;
        TcpTransport::validate_response_code(&content, &reply)?;
        TcpTransport::get_reply_data(&reply, expected_bytes).map(Box::from)
    }

    fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error> {
        let ComposedMessage {
            content,
            header,
            expected_bytes,
        } = self.compositor().compose_feedback(data)?;
        let mut reply = vec![0; MODBUS_HEADER_SIZE + expected_bytes + 2].into_boxed_slice();

        self.stream.write_all(&content).map_err(Error::Io)?;
        self.stream.read(&mut reply).map_err(Error::Io)?;

        let reply_header_raw = &reply
            .get(..MODBUS_HEADER_SIZE)
            .ok_or(Error::InvalidResponse)?;
        let resp_hd = Header::unpack(reply_header_raw)?;

        TcpTransport::validate_response_header(&header, &resp_hd)?;
        TcpTransport::validate_response_code(&content, &reply)?;
        TcpTransport::get_reply_data(&reply, expected_bytes).map(Box::from)
    }
}

/// The TCP ModBus client.
///
/// Example:
/// ```
/// // Import prelude items
/// use labjack::prelude::*;
/// // Import the specific pin we wish to read
/// use labjack::prelude::LookupTable::Ain55;
///
/// // Connect to our LabJack over TCP
/// let mut device = LabJack::connect::<Emulated>(-2).expect("Must connect");
/// // Read the AIN55 pin without an extended feature
/// let voltage = device.read(Ain55, ()).expect("Must read");
///
/// println!("Voltage={}", voltage);
/// ```
pub struct Tcp;

impl Connect for Tcp {
    type Transport = TcpTransport;

    fn connect(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error> {
        let addr = SocketAddr::new(device.ip_address, MODBUS_COMMUNICATION_PORT);
        let stream = TcpStream::connect(addr).map_err(Error::Io)?;

        Ok(Box::new(TcpTransport::new(stream)))
    }
}
