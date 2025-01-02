use crate::prelude::LabJackDataValue;

pub trait Adc {
    type Digital;

    type Voltage<'a>: From<LabJackDataValue>
    where
        Self: 'a;

    fn to_digital(&self, voltage: Self::Voltage<'_>) -> Self::Digital;
}
