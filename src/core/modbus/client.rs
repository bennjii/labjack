use super::{binary, function::*, Transport};
use byteorder::{BigEndian, WriteBytesExt};

pub const MODBUS_PROTOCOL_TCP: u16 = 0x0000;
pub const MODBUS_TCP_DEFAULT_PORT: u16 = 502;
pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub trait Client: Transport {
    fn read_holding_registers(
        &mut self,
        addr: Address,
        quant: Quantity,
    ) -> Result<Vec<Value>, Self::Error> {
        let bytes = self.read(&ReadFunction::HoldingRegisters(addr, quant))?;
        binary::pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn read_input_registers(
        &mut self,
        addr: Address,
        quant: Quantity,
    ) -> Result<Vec<Value>, Self::Error> {
        let bytes = self.read(&ReadFunction::InputRegisters(addr, quant))?;
        binary::pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn write_register(&mut self, addr: Address, value: Value) -> Result<(), Self::Error> {
        self.write(&WriteFunction::SingleRegister(addr, value))
    }

    fn write_registers(&mut self, addr: Address, values: &[Value]) -> Result<(), Self::Error> {
        let bytes = binary::unpack_bytes(values);
        let quantity = values.len() as Value;

        self.write(&WriteFunction::MultipleRegisters(addr, quantity, &bytes))
    }
}

impl<T> Client for T where T: Transport {}
