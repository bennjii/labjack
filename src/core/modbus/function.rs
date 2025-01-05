use crate::core::DataValue;
use crate::prelude::{DataType, LabJackDataValue, LabJackEntity};
use crate::prelude::data_types::Register;

pub type Address = u16;
pub type Quantity = u16;

pub enum FeedbackFunction<'a> {
    ReadRegisters(Address, u8),
    WriteRegisters(Address, &'a [u8]),
}

// TODO: The addresses are derrived from an entity, this entity will span either a
//       single or multiple registers. Therefore, prefer simply writing a labjack
//       entity, with a value.
pub enum WriteFunction<'a> {
    SingleRegister(Address, LabJackDataValue),
    MultipleRegisters(Address, &'a [LabJackDataValue]),
}

// Write all registers corresponding to the entity, with given value.
// Must assert that the entity and value match register variants on types provided.
// TODO: Can we make this assertion compile-safe?
pub struct WriteFunction2<R: Register>(R, <R::DataType as DataType>::Value);

// Read all registers corresponding to the entity
pub struct ReadFunction2<R: Register>(R);

pub enum ReadFunction<R: Register> {
    HoldingRegister(R),
    // "Seldom Used". Prefer Holding.
    InputRegister(R),
}

impl<R> ReadFunction<R> where R: Register {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            ReadFunction::HoldingRegister(..) => 0x03,
            ReadFunction::InputRegister(..) => 0x04,
        }
    }
}

impl WriteFunction<'_> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            WriteFunction::SingleRegister(..) => 0x06,
            WriteFunction::MultipleRegisters(..) => 0x10,
        }
    }
}

impl<'a> FeedbackFunction<'a> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            FeedbackFunction::ReadRegisters(_, _) => 0x00,
            FeedbackFunction::WriteRegisters(_, _) => 0x01,
        }
    }
}
