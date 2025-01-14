use crate::prelude::*;

pub const MODBUS_UNIT_ID: u8 = 1;
pub const MODBUS_HEADER_SIZE: usize = 7;
pub const MODBUS_MAX_PACKET_SIZE: usize = 260;

pub trait Client: Transport {
    fn read_register(
        &mut self,
        register: Register,
    ) -> impl std::future::Future<Output = Result<LabJackDataValue, Self::Error>> + Send {
        self.read(ReadFunction(register))
    }

    fn write_register(
        &mut self,
        register: Register,
        value: LabJackDataValue,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        self.write(WriteFunction(register, value))
    }
}

impl<T> Client for T where T: Transport {}
