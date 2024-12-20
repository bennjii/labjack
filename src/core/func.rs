use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::prelude::translate::LookupTable;

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum LabJackDataType {
    Uint16 = 0,
    Uint32 = 1,
    Int32 = 2,
    Float32 = 3,
}

impl LabJackDataType {
    const fn from_u32(value: u32) -> Self {
        match value {
            0 => LabJackDataType::Uint16,
            1 => LabJackDataType::Uint32,
            2 => LabJackDataType::Int32,
            3 => LabJackDataType::Float32,
            _ => panic!("Invalid data type. Must be between 0 and 3, inclusive."),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct LabJackEntity {
    pub entry: LookupTable,

    pub address: u32,
    pub data_type: LabJackDataType,
}

impl From<LookupTable> for LabJackEntity {
    fn from(val: LookupTable) -> Self {
        val.raw()
    }
}

impl LabJackEntity {
    pub const fn new(address: u32, data_type: u32, entry: LookupTable) -> LabJackEntity {
        LabJackEntity {
            address,
            entry,
            data_type: LabJackDataType::from_u32(data_type),
        }
    }
}

impl Display for LabJackEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.entry)
    }
}
