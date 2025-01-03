use std::io;
use std::io::Write;
use std::borrow::BorrowMut;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::core::{Error, Function, FeedbackFunction, Reason, MODBUS_HEADER_SIZE, MODBUS_MAX_PACKET_SIZE, MODBUS_PROTOCOL_TCP, WriteFunction};
use crate::prelude::ReadFunction;

/// Ephemeral structure created from the transport to compose messages. It's internal state is
/// only of a mutable extension of the [`Transport`] explicitly only containing domain-specific
/// information with an emphasis on which properties can be mutated in the base transport.
///
/// It is used to compose messages for use over modbus.
pub struct Compositor<'a> {
    pub transaction_id: &'a mut u16,
    pub unit_id: u8,
}

#[derive(Debug)]
pub struct ComposedMessage {
    pub content: Vec<u8>,

    pub(crate) header: Header,
    pub(crate) expected_bytes: usize
}

/// The header on a given modbus message.
///
/// Defines how large the payload will be, and the corresponding transaction, protocol and unit ids.
///
#[derive(Debug, PartialEq)]
pub struct Header {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub length: u16,
    pub unit_id: u8,
}

impl<'a> Compositor<'a> {
    pub fn new(transaction_id: &'a mut u16, unit_id: u8) -> Self {
        Self {
            transaction_id,
            unit_id,
        }
    }

    fn new_tid(&mut self) -> &u16 {
        *self.transaction_id = self.transaction_id.wrapping_add(1);
        self.transaction_id
    }

    pub fn compose_read(&mut self, function: &ReadFunction) -> Result<ComposedMessage, Error> {
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
        let mut content = header.pack()?;

        content.write_u8(function.code())?;

        content.write_u16::<BigEndian>(addr)?;
        content.write_u16::<BigEndian>(count)?;

        Ok(ComposedMessage {
            content, header, expected_bytes
        })
    }

    pub fn compose_write(&mut self, function: &WriteFunction) -> Result<ComposedMessage, Error> {
        let size = match function {
            WriteFunction::SingleRegister(..) => 5,
            WriteFunction::MultipleRegisters(.., bytes) => 6 + bytes.len()
        };

        let header = Header::new(self, size as u16 + 1u16);
        let mut buff = header.pack()?;
        buff.write_u8(function.code())?;

        match *function {
            WriteFunction::SingleRegister(addr, val) => {
                buff.write_u16::<BigEndian>(addr)?;
                buff.write_u16::<BigEndian>(val)?;
            },
            WriteFunction::MultipleRegisters(addr, quantity, bytes) => {
                buff.write_u16::<BigEndian>(addr)?;
                buff.write_u16::<BigEndian>(quantity)?;
                buff.write_u8(bytes.len() as u8)?;

                for v in bytes {
                    buff.write_u8(*v)?;
                }
            }
        }

        Ok(ComposedMessage {
            content, header, expected_bytes: 0usize
        })
    }

    pub fn compose_feedback(&mut self, fns: &[FeedbackFunction]) -> Result<ComposedMessage, Error> {
        let mut read_return_size = 0;

        // Must account for unit ID and function ID (2 bytes) + base header size
        let composed_size = fns.iter().fold(MODBUS_HEADER_SIZE + 2, |acc, f| match f {
            FeedbackFunction::ReadRegisters(_, _) => {
                read_return_size += 2;
                acc + 4
            }
            FeedbackFunction::WriteRegisters(_, values) => acc + 4 + values.len(),
        });

        let header = Header::new(self, composed_size as u16);
        let mut content = header.pack()?;

        content.write_u8(Function::Feedback(fns).code())?;

        for frame in fns {
            content.write_u8(frame.code())?;

            match frame {
                FeedbackFunction::ReadRegisters(addr, quant) => {
                    content.write_u16::<BigEndian>(*addr)?;
                    content.write_u8(*quant)?;
                }
                FeedbackFunction::WriteRegisters(addr, values) => {
                    content.write_u16::<BigEndian>(*addr)?;
                    content.write_u8(values.len() as u8)?;
                    for v in *values {
                        content.write_u8(*v)?;
                    }
                }
            }
        }

        Ok(ComposedMessage {
            content, header, expected_bytes: 7 + read_return_size
        })
    }
}

impl Header {
    fn new(compositor: &mut Compositor, len: u16) -> Header {
        Header {
            transaction_id: *compositor.new_tid(),
            protocol_id: MODBUS_PROTOCOL_TCP,
            length: len - MODBUS_HEADER_SIZE as u16,
            unit_id: compositor.unit_id,
        }
    }

    pub fn pack(&self) -> Result<Vec<u8>, Error> {
        let mut buff = vec![];
        buff.write_u16::<BigEndian>(self.transaction_id)?;
        buff.write_u16::<BigEndian>(self.protocol_id)?;
        buff.write_u16::<BigEndian>(self.length)?;
        buff.write_u8(self.unit_id)?;
        Ok(buff)
    }

    pub fn unpack(buff: &[u8]) -> Result<Header, Error> {
        let mut rdr = io::Cursor::new(buff);
        Ok(Header {
            transaction_id: rdr.read_u16::<BigEndian>()?,
            protocol_id: rdr.read_u16::<BigEndian>()?,
            length: rdr.read_u16::<BigEndian>()?,
            unit_id: rdr.read_u8()?,
        })
    }
}
