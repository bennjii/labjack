use crate::core::data_types::{Decode, Register};
use crate::prelude::*;

pub const MODBUS_UNIT_ID: u8 = 1;
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
        self.read::<Reg>(&ReadFunction(register))
    }

    fn write_register<Reg>(
        &mut self,
        register: Reg,
        value: <<Reg as Register>::DataType as DataType>::Value,
    ) -> Result<(), Self::Error>
    where
        Reg: Register,
    {
        self.write(&WriteFunction(register, value))
    }
}

impl<T> Client for T where T: Transport {}
