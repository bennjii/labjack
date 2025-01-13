use crate::core::data_types::Register;
use crate::prelude::*;
use crate::queue::buffer::Topic;
use std::sync::Arc;

pub const MODBUS_UNIT_ID: u8 = 1;
pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub trait Client: Transport {
    async fn read_register(&mut self, register: Register) -> Result<LabJackDataValue, Self::Error> {
        self.read(ReadFunction(register)).await
    }

    async fn write_register(
        &mut self,
        register: Register,
        value: LabJackDataValue,
    ) -> Result<(), Self::Error> {
        self.write(WriteFunction(register, value)).await
    }
}

impl<T> Client for T where T: Transport {}
