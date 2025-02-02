use crate::core::LabJackDataValue;

pub trait Adc<Ctx> {
    type Digital;

    fn to_digital(&self, context: Ctx, voltage: LabJackDataValue) -> Self::Digital;
}

// Default/Pass-through implementation (NO-OP)
impl Adc<()> for () {
    type Digital = LabJackDataValue;

    fn to_digital(&self, context: (), voltage: LabJackDataValue) -> Self::Digital {
        voltage
    }
}
