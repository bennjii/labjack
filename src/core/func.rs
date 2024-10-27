#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
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

pub struct LabJackEntity {
    pub address: u32,
    pub data_type: LabJackDataType,
}

impl LabJackEntity {
    pub const fn new(address: u32, data_type: u32) -> LabJackEntity {
        LabJackEntity {
            address,
            data_type: LabJackDataType::from_u32(data_type),
        }
    }
}

pub trait LabJackFunctionality {
    fn read(&self, item: LabJackEntity) -> f64;
}
