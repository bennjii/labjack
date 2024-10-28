use super::{binary, function::*, Transport};
use byteorder::{BigEndian, WriteBytesExt};

pub const MODBUS_PROTOCOL_TCP: u16 = 0x0000;
pub const MODBUS_TCP_DEFAULT_PORT: u16 = 502;
pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub trait Client: Transport {
    fn read_holding_registers(&mut self, addr: Address, quant: Quantity) -> Result<Vec<Value>, Self::Error> {
        let bytes = self.read(&Function::ReadHoldingRegisters(addr, quant))?;
        binary::pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn read_input_registers(&mut self, addr: Address, quant: Quantity) -> Result<Vec<Value>, Self::Error> {
        let bytes = self.read(&Function::ReadInputRegisters(addr, quant))?;
        binary::pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn write_register(&mut self, addr: Address, value: Value) -> Result<(), Self::Error> {
        let mut buff = vec![0; MODBUS_HEADER_SIZE]; // Header gets filled in later
        buff.write_u8(Function::WriteRegister(addr, value).code())?;
        buff.write_u16::<BigEndian>(addr)?;
        buff.write_u16::<BigEndian>(value)?;
        self.write(&mut buff)
    }

    fn write_registers(&mut self, addr: Address, values: &[Value]) -> Result<(), Self::Error> {
        let bytes = binary::unpack_bytes(values);
        let quantity = values.len() as Value;

        let mut buff = vec![0; MODBUS_HEADER_SIZE]; // Header gets filled in later
        buff.write_u8(Function::WriteRegisters(addr, quantity, &bytes).code())?;
        buff.write_u16::<BigEndian>(addr)?;
        buff.write_u16::<BigEndian>(quantity)?;
        buff.write_u8(bytes.len() as u8)?;

        for v in bytes {
            buff.write_u8(v)?;
        }

        self.write(&mut buff)
    }

    fn feedback(&mut self, fns: &[ModbusFeedbackFunction]) -> Result<(), Self::Error> {
        let mut buff = vec![0; MODBUS_HEADER_SIZE];
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
        
        self.write(&mut buff)
    }
}
