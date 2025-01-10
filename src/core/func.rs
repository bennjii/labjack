use crate::prelude::modbus::{Error, Quantity, Reason};

use num::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[repr(u32)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum LabJackDataType {
    Uint16 = 0,
    Uint32 = 1,
    Int32 = 2,
    Float32 = 3,
    Uint64 = 4,
    String = 98,
    Byte = 99,
}

pub trait DataType: Debug {
    type Value: FromPrimitive + ToPrimitive + Clone + Debug;

    fn data_type(&self) -> LabJackDataType;
    fn bytes(&self, value: &Self::Value) -> Vec<u8>;
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
            LabJackDataType::Byte | LabJackDataType::Uint16 => 1,
            LabJackDataType::Uint64 => 4,
            // All other types are 32-bit.
            _ => 2,
        }
    }
}

pub struct DataValue<T: DataType> {
    pub value: T::Value,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum LabJackDataValue {
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Int32(i32),
    Float32(f32),
    Byte(u8),
}

impl From<LabJackDataValue> for f64 {
    fn from(value: LabJackDataValue) -> Self {
        match value {
            LabJackDataValue::Uint16(x) => x as f64,
            LabJackDataValue::Uint32(x) => x as f64,
            LabJackDataValue::Uint64(x) => x as f64,
            LabJackDataValue::Int32(x) => x as f64,
            LabJackDataValue::Float32(x) => x as f64,
            LabJackDataValue::Byte(x) => x as f64,
        }
    }
}

impl LabJackDataValue {
    pub fn r#type(&self) -> LabJackDataType {
        match self {
            LabJackDataValue::Uint16(_) => LabJackDataType::Uint16,
            LabJackDataValue::Uint32(_) => LabJackDataType::Uint32,
            LabJackDataValue::Uint64(_) => LabJackDataType::Uint64,
            LabJackDataValue::Int32(_) => LabJackDataType::Int32,
            LabJackDataValue::Float32(_) => LabJackDataType::Float32,
            LabJackDataValue::Byte(_) => LabJackDataType::Byte,
        }
    }

    /// Union-Backed Downcast to a HOT.
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
            .to_f64(),
            4 => u32::from_be_bytes(
                bytes
                    .try_into()
                    .map_err(|_| Error::InvalidData(Reason::DecodingError))?,
            )
            .to_f64(),
            _ => None,
        };

        be_value
            .and_then(T::from_f64)
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
            LabJackDataType::Uint64 => Ok(LabJackDataValue::Uint64(
                LabJackDataValue::decode_bytes::<u64>(bytes)?, // f32::from_be_bytes(bytes.try_into().map_err(|_| Error::InvalidData(Reason::DecodingError))?)
            )),
            LabJackDataType::Byte => unimplemented!(),
            LabJackDataType::String => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct LabJackEntity {
    pub entry: Register,
    pub address: u32,
    pub data_type: LabJackDataType,
}

impl LabJackEntity {
    pub const fn new(
        address: u32,
        entry: Register,
        data_type: LabJackDataType,
    ) -> LabJackEntity {
        LabJackEntity {
            address,
            entry,
            data_type,
        }
    }
}

impl Display for LabJackEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.entry)
    }
}
