use crate::prelude::*;

pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub type ModbusRegister = u16;

pub trait Client: Transport {
    fn read_holding_registers(
        &mut self,
        addr: Address,
        quant: Quantity,
    ) -> Result<Vec<ModbusRegister>, Self::Error> {
        let bytes = self.read(&ReadFunction::HoldingRegisters(addr, quant))?;
        pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn read_input_registers(
        &mut self,
        addr: Address,
        quant: Quantity,
    ) -> Result<Vec<ModbusRegister>, Self::Error> {
        let bytes = self.read(&ReadFunction::InputRegisters(addr, quant))?;
        pack_bytes(&bytes[..]).map_err(Self::Error::from)
    }

    fn write_register(&mut self, addr: Address, value: LabJackDataValue) -> Result<(), Self::Error> {
        self.write(&WriteFunction::SingleRegister(addr, value))
    }

    fn write_registers(&mut self, addr: Address, values: &[LabJackDataValue]) -> Result<(), Self::Error> {
        self.write(&WriteFunction::MultipleRegisters(addr, values))
    }
}

impl<T> Client for T where T: Transport {}
