use crate::core::data_types::Register;
use crate::core::DataType;
use crate::prelude::data_types::Decode;
use crate::prelude::modbus::ReadFunction;
use crate::prelude::{FeedbackFunction, LabJackDataValue, WriteFunction};

pub trait Transport {
    type Error: From<std::io::Error> + Sized;

    fn write<R>(&mut self, function: &WriteFunction<R>) -> Result<(), Self::Error>
    where
        R: Register;
    fn read<R>(
        &mut self,
        function: &ReadFunction<R>,
    ) -> Result<<R::DataType as DataType>::Value, Self::Error>
    where
        R: Register;
    // TODO: Return type should be feedback values not bytes
    fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error>;
}
