use crate::prelude::data_types::Register;
use crate::prelude::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::Write;

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
    pub(crate) expected_bytes: usize,
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
        let word_size = function.0.data_type.size();
        if word_size < 1 {
            return Err(Error::InvalidData(Reason::RecvBufferEmpty));
        }

        if word_size as usize > MODBUS_MAX_PACKET_SIZE {
            return Err(Error::InvalidData(Reason::UnexpectedReplySize));
        }

        // The length in a feedback function might be different if
        // using a different frame type.
        let header = Header::new(self, 6u16);
        let mut content = header.pack()?;

        content.write_u8(function.code())?;

        content.write_u16::<BigEndian>(function.0.address)?;
        content.write_u16::<BigEndian>(word_size)?;

        Ok(ComposedMessage {
            content,
            header,
            expected_bytes: 2 * word_size as usize,
        })
    }

    pub fn compose_write(&mut self, function: &WriteFunction) -> Result<ComposedMessage, Error> {
        let size = function.0.data_type.size();
        let bytes = size * 2;

        let header = Header::new(self, bytes + MODBUS_HEADER_SIZE as u16);
        let mut content = header.pack()?;

        content.write_u8(function.code())?;
        content.write_u16::<BigEndian>(function.0.address)?;
        content.write_u16::<BigEndian>(size)?;

        let bytes = function.1.bytes();
        content.write_u8(bytes.len() as u8)?;

        for v in bytes {
            content.write_u8(v)?;
        }

        Ok(ComposedMessage {
            content,
            header,
            // Device will relay starting address and num. registers.
            expected_bytes: 4usize,
        })
    }

    pub fn compose_feedback(&mut self, fns: &[FeedbackFunction]) -> Result<ComposedMessage, Error> {
        let mut read_return_size = 0;

        // Must account for unit ID and function ID (2 bytes) + base header size
        // TODO: Simplify- calculation isnt this difficult.
        let composed_size = fns.iter().fold(2, |acc, f| match f {
            FeedbackFunction::ReadRegister(..) => {
                // TODO: Assumes 2width (not allways true)
                read_return_size += 2;
                acc + 4
            }
            FeedbackFunction::WriteRegister(register, data) => acc + 4 + data.bytes().len(),
        });

        let header = Header::new(self, composed_size as u16);
        let mut content = header.pack()?;

        content.write_u8(0x4C)?; // 0x4C is Feedback Code

        for frame in fns {
            content.write_u8(frame.code())?;

            match frame {
                FeedbackFunction::ReadRegister(register) => {
                    content.write_u16::<BigEndian>(register.address)?;
                    content.write_u8(register.data_type.size() as u8)?;
                }
                FeedbackFunction::WriteRegister(register, value) => {
                    let bytes = value.bytes();

                    content.write_u16::<BigEndian>(register.address)?;
                    content.write_u8(bytes.len() as u8)?;
                    content.write_all(&bytes)?;
                }
            }
        }

        Ok(ComposedMessage {
            content,
            header,
            expected_bytes: 7 + read_return_size,
        })
    }
}

impl Header {
    fn new(compositor: &mut Compositor, len: u16) -> Header {
        Header {
            transaction_id: *compositor.new_tid(),
            protocol_id: MODBUS_PROTOCOL_TCP,
            length: len,
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

#[cfg(test)]
mod test {
    use crate::prelude::data_types::Register;
    use crate::prelude::*;

    #[test]
    fn write_standard() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let register = AIN55;
        let write_function = WriteFunction(*register, LabJackDataValue::Float32(16f32));
        let ComposedMessage { content, .. } = compositor
            .compose_write(&write_function)
            .expect("Must-compose");

        let spanning_registers = AIN55.data_type.size();
        let expected_size = (2 * spanning_registers);

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!([0x00, 0x00], content[2..4]); // ProtocolID
        assert_eq!((expected_size + 7).to_be_bytes(), content[4..6]); // Length (MSB-LSB)
        assert_eq!([MODBUS_UNIT_ID, 0x10], content[6..8]); // UnitID & Write Function Code
        assert_eq!(AIN55.address.to_be_bytes(), content[8..10]);
        assert_eq!(spanning_registers.to_be_bytes(), content[10..12]);
        assert_eq!(expected_size as u8, content[12]);

        // Assert data values are correct.
    }

    /// Validating the exemplar command given in the documentation
    ///
    /// [LabJack Reference Documentation](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-WriteDAC0)
    #[test]
    fn write_dac_zero() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let write_function = WriteFunction(*DAC0, LabJackDataValue::Float32(3.3f32));
        let ComposedMessage { content, .. } = compositor
            .compose_write(&write_function)
            .expect("Must-compose");

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!(
            [
                0x00, 0x00, 0x00, 0x0B, 0x01, 0x10, 0x03, 0xE8, 0x00, 0x02, 0x04, 0x40, 0x53, 0x33,
                0x33
            ],
            content[2..]
        );
    }

    #[test]
    fn write_test_u32() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let write_function = WriteFunction(*TEST_UINT32, LabJackDataValue::Uint32(0xC0BCCCCD));
        let ComposedMessage { content, .. } = compositor
            .compose_write(&write_function)
            .expect("Must-compose");

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!(
            [
                0x00, 0x00, 0x00, 0x0B, 0x01, 0x10, 0xD7, 0x50, 0x00, 0x02, 0x04, 0xC0, 0xBC, 0xCC,
                0xCD
            ],
            content[2..]
        );
    }

    #[test]
    fn read_test_u32() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let read_function = ReadFunction(*TEST_UINT32);
        let ComposedMessage { content, .. } = compositor
            .compose_read(&read_function)
            .expect("Must-compose");

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!(
            [0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0xD7, 0x50, 0x00, 0x02],
            content[2..]
        );
    }

    #[test]
    fn read_test_u16() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let read_function = ReadFunction(*TEST_UINT16);
        let ComposedMessage { content, .. } = compositor
            .compose_read(&read_function)
            .expect("Must-compose");

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!(
            [0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0xD7, 0x46, 0x00, 0x01],
            content[2..]
        );
    }

    #[test]
    fn read_fio_zero() {
        let mut transaction_id = 1;
        let mut compositor = Compositor::new(&mut transaction_id, MODBUS_UNIT_ID);

        let read_function = ReadFunction(*FIO0);
        let ComposedMessage { content, .. } = compositor
            .compose_read(&read_function)
            .expect("Must-compose");

        assert_eq!(transaction_id.to_be_bytes(), content[0..2]); // TransactionID
        assert_eq!(
            [0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x07, 0xD0, 0x00, 0x01],
            content[2..]
        );
    }
}
