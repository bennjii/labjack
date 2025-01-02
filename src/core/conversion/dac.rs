use crate::core::LabJackDataValue;

pub trait Dac {
    type Digital<'a>
    where
        Self: 'a;

    type Voltage<'a>: From<LabJackDataValue>
    where
        Self: 'a;

    fn to_voltage(&self, digital: Self::Digital<'_>) -> Self::Voltage<'_>;
}
