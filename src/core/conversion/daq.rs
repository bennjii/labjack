use super::{adc, dac};

pub trait Daq: adc::Adc + dac::Dac {
    type Digital: From<<Self as adc::Adc>::Digital>;
}

impl<T> Daq for T
where
    T: adc::Adc + dac::Dac,
{
    type Digital = <T as adc::Adc>::Digital;
}
