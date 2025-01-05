use crate::core::LabJackDataValue;

pub trait Adc {
    type Digital;

    fn to_digital(&self, voltage: LabJackDataValue) -> Self::Digital;
}

// Default/Pass-through implementation (NO-OP)
impl Adc for () {
    type Digital = LabJackDataValue;

    fn to_digital(&self, voltage: LabJackDataValue) -> Self::Digital {
        voltage
    }
}
