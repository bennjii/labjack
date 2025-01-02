use crate::core::LabJackDataValue;
use super::{adc, dac};

pub trait Daq: adc::Adc + dac::Dac
{
    type Digital: From<<Self as adc::Adc>::Digital>;
    type Voltage<'a>: From<LabJackDataValue> where Self: 'a;
}

impl<T> Daq for T where T: adc::Adc + dac::Dac {
    type Digital = <T as adc::Adc>::Digital;
    type Voltage<'a> = <T as adc::Adc>::Voltage<'a> where T: 'a;
}