use crate::prelude::modbus::ReadFunction;

pub trait Transport {
    type Error: From<std::io::Error> + Sized;

    fn write(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn read(&mut self, function: &ReadFunction) -> Result<Box<[u8]>, Self::Error>;
}
