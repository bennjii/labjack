use crate::core::data_types::{Decode, Register};
use crate::prelude::*;

pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub trait Client: Transport {
    fn read_register<Reg>(
        &mut self,
        register: Reg,
    ) -> Result<<Reg::DataType as DataType>::Value, Self::Error>
    where
        Reg: Register,
    {
        self.read::<Reg>(&ReadFunction::HoldingRegister(register))
    }

    fn write_register(
        &mut self,
        addr: Address,
        value: LabJackDataValue,
    ) -> Result<(), Self::Error> {
        self.write(&WriteFunction::SingleRegister(addr, value))
    }

    fn write_registers(
        &mut self,
        addr: Address,
        values: &[LabJackDataValue],
    ) -> Result<(), Self::Error> {
        self.write(&WriteFunction::MultipleRegisters(addr, values))
    }
}

impl<T> Client for T where T: Transport {}
