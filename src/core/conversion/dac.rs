use crate::core::LabJackDataValue;

pub trait Dac {
    type Digital<'a>
    where
        Self: 'a;

    fn to_voltage(&self, digital: Self::Digital<'_>) -> LabJackDataValue;
}

impl Dac for () {
    type Digital<'a> = LabJackDataValue;

    fn to_voltage(&self, digital: Self::Digital<'_>) -> LabJackDataValue {
        digital
    }
}
