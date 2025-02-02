use super::{adc, dac};

pub trait Daq<Ctx>: adc::Adc<Ctx> + dac::Dac {
    type Digital: From<<Self as adc::Adc<Ctx>>::Digital>;
}

impl<T, Ctx> Daq<Ctx> for T
where
    T: adc::Adc<Ctx> + dac::Dac,
{
    type Digital = <T as adc::Adc<Ctx>>::Digital;
}
