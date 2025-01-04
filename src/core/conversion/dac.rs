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

impl Dac for () {
    type Digital<'a> = f64;

    type Voltage<'a> = f64;

    fn to_voltage(&self, digital: Self::Digital<'_>) -> Self::Voltage<'_> {
        digital
    }
}
