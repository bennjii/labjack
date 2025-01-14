use std::fmt::Debug;

use crate::prelude::*;

pub trait Transport: Debug {
    type Error: From<std::io::Error> + Sized;

    fn write(
        &mut self,
        function: WriteFunction,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    fn read(
        &mut self,
        function: ReadFunction,
    ) -> impl std::future::Future<Output = Result<LabJackDataValue, Self::Error>> + Send;

    // TODO: Return type should be feedback values not bytes
    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error>;
}
