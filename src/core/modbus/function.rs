use crate::prelude::data_types::Register;
use crate::prelude::LabJackDataValue;

pub type Address = u16;
pub type Quantity = u16;

pub enum FeedbackFunction {
    ReadRegister(Register),
    WriteRegister(Register, LabJackDataValue),
}

impl FeedbackFunction {
    pub fn register(&self) -> Register {
        match self {
            FeedbackFunction::ReadRegister(r) => *r,
            FeedbackFunction::WriteRegister(r, _) => *r,
        }
    }
}

/// Write all registers corresponding to the entity, with given value.
/// Must assert that the entity and value match register variants on types provided.
pub struct WriteFunction(pub Register, pub LabJackDataValue);

/// Read all registers corresponding to the entity.
pub struct ReadFunction(pub Register);

pub trait Function {
    fn code(&self) -> u8;
}

impl Function for ReadFunction {
    fn code(&self) -> u8 {
        0x03
    }
}

impl Function for WriteFunction {
    fn code(&self) -> u8 {
        0x10 // 16
    }
}

impl Function for FeedbackFunction {
    fn code(&self) -> u8 {
        match *self {
            FeedbackFunction::ReadRegister(..) => 0x00,
            FeedbackFunction::WriteRegister(..) => 0x01,
        }
    }
}
