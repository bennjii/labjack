use crate::prelude::modbus::{Error, Quantity, Reason};
use crate::prelude::translate::LookupTable;
use num::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use crate::prelude::ModbusRegister;

#[repr(u32)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum LabJackDataType {
    Uint16 = 0,
    Uint32 = 1,
    Int32 = 2,
    Float32 = 3,
}

impl LabJackDataType {
    /// Determines the LabJack size representation over ModBus.
    ///
    /// For example, a U16 FIO0 register is 16bits, or 2 words (see below). Therefore, it's size is 1.
    /// Whereas, U32 AIN0 is 4 words, and so it's size is 2. Note that the returned values
    /// over modbus are stored in big-endian.
    ///
    /// > Note: LabJack's base unit size (word) is 1 standard byte (8bit).
    ///
    /// Referenced Documentation: [Protocol Details - Register Size](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails[DirectModbusTCP]-ModbusRegistersAre16-bit,LabJackValuesAreOneorMoreModbusRegisters)
    pub fn size(&self) -> Quantity {
        match self {
            LabJackDataType::Uint16 => 1,
            // All other types are 32-bit.
            _ => 2,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LabJackDataValue {
    Uint16(u16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
}

impl From<LabJackDataValue> for f64 {
    fn from(value: LabJackDataValue) -> Self {
        match value {
            LabJackDataValue::Uint16(x) => x as f64,
            LabJackDataValue::Uint32(x) => x as f64,
            LabJackDataValue::Int32(x) => x as f64,
            LabJackDataValue::Float32(x) => x as f64,
        }
    }
}

impl LabJackDataValue {
    pub fn r#type(&self) -> LabJackDataType {
        match self {
            LabJackDataValue::Uint16(_) => LabJackDataType::Uint16,
            LabJackDataValue::Uint32(_) => LabJackDataType::Uint32,
            LabJackDataValue::Int32(_) => LabJackDataType::Int32,
            LabJackDataValue::Float32(_) => LabJackDataType::Float32,
        }
    }

    pub fn register(value: ModbusRegister) -> LabJackDataValue {
        LabJackDataValue::Uint16(value)
    }

    pub fn as_f64(&self) -> f64 {
        f64::from(*self)
    }

    pub(crate) fn decode_bytes<T: FromPrimitive>(bytes: &[u8]) -> Result<T, Error> {
        let be_value = match bytes.len() {
            2 => u16::from_be_bytes(
                bytes
                    .try_into()
                    .map_err(|_| Error::InvalidData(Reason::DecodingError))?,
            )
            .to_u64(),
            4 => u32::from_be_bytes(
                bytes
                    .try_into()
                    .map_err(|_| Error::InvalidData(Reason::DecodingError))?,
            )
            .to_u64(),
            _ => None,
        };

        be_value
            .and_then(T::from_u64)
            .ok_or(Error::InvalidData(Reason::DecodingError))
    }

    pub fn from_bytes(data_type: LabJackDataType, bytes: &[u8]) -> Result<Self, Error> {
        match data_type {
            LabJackDataType::Uint16 => Ok(LabJackDataValue::Uint16(
                LabJackDataValue::decode_bytes::<u16>(bytes)?, // u16::from_be_bytes(bytes.try_into().map_err(|_| Error::InvalidData(Reason::DecodingError))?)
            )),
            LabJackDataType::Uint32 => Ok(LabJackDataValue::Uint32(
                LabJackDataValue::decode_bytes::<u32>(bytes)?, // u32::from_be_bytes(bytes.try_into().map_err(|_| Error::InvalidData(Reason::DecodingError))?)
            )),
            LabJackDataType::Int32 => Ok(LabJackDataValue::Int32(
                LabJackDataValue::decode_bytes::<i32>(bytes)?, // i32::from_be_bytes(bytes.try_into().map_err(|_| Error::InvalidData(Reason::DecodingError))?)
            )),
            LabJackDataType::Float32 => Ok(LabJackDataValue::Float32(
                LabJackDataValue::decode_bytes::<f32>(bytes)?, // f32::from_be_bytes(bytes.try_into().map_err(|_| Error::InvalidData(Reason::DecodingError))?)
            )),
        }
    }
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
