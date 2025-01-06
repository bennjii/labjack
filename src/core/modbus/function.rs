use crate::core::DataValue;
use crate::prelude::data_types::Register;
use crate::prelude::{DataType, LabJackDataValue, LabJackEntity};

pub type Address = u16;
pub type Quantity = u16;

pub enum FeedbackFunction<'a> {
    ReadRegisters(Address, u8),
    WriteRegisters(Address, &'a [u8]),
}

// Write all registers corresponding to the entity, with given value.
// Must assert that the entity and value match register variants on types provided.
pub struct WriteFunction<R: Register>(pub R, pub <R::DataType as DataType>::Value);

// Read all registers corresponding to the entity
pub struct ReadFunction<R: Register>(pub R);

impl<R> ReadFunction<R>
where
    R: Register,
{
    pub(crate) fn code(&self) -> u8 {
        0x03 // 3
    }
}

impl<R> WriteFunction<R>
where
    R: Register,
{
    pub(crate) fn code(&self) -> u8 {
        0x10 // 16
    }
}

impl FeedbackFunction<'_> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            FeedbackFunction::ReadRegisters(..) => 0x00,
            FeedbackFunction::WriteRegisters(..) => 0x01,
        }
    }
}
