use std::fmt::Debug;

use crate::prelude::*;

pub trait Transport: Debug {
    type Error: From<std::io::Error> + Sized;

    async fn write(&mut self, function: WriteFunction) -> Result<(), Self::Error>;

    async fn read(&mut self, function: ReadFunction) -> Result<LabJackDataValue, Self::Error>;

    // TODO: Return type should be feedback values not bytes
    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error>;
}