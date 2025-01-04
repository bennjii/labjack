use crate::prelude::modbus::ReadFunction;
use crate::prelude::{FeedbackFunction, WriteFunction};

pub trait Transport {
    type Error: From<std::io::Error> + Sized;

    fn write(&mut self, function: &WriteFunction) -> Result<(), Self::Error>;
    // TODO: Return type should be register values not bytes - Take impl from client.rs:44
    fn read(&mut self, function: &ReadFunction) -> Result<Box<[u8]>, Self::Error>;
    // TODO: Return type should be feedback values not bytes
    fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error>;
}
