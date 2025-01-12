use std::fmt::Debug;
use crate::prelude::modbus::ReadFunction;
use crate::prelude::{LabJackDataValue, WriteFunction};

pub trait Transport: Debug {
    type Error: From<std::io::Error> + Sized;

    fn write(&mut self, function: WriteFunction) -> Result<(), Self::Error>;

    fn read(&mut self, function: ReadFunction) -> Result<LabJackDataValue, Self::Error>;

    // TODO: Return type should be feedback values not bytes
    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error>;
}
