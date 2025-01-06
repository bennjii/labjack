use crate::core::data_types::Decode;
use crate::prelude::data_types::{Coerce, Register};
use crate::prelude::*;
use either::Either;

pub struct LabJackClient<T>
where
    T: Transport,
{
    pub device: LabJackDevice,
    transport: T,
}

impl<T> LabJackClient<T>
where
    T: Transport,
{
    pub fn new(device: LabJackDevice, transport: T) -> LabJackClient<T> {
        LabJackClient { device, transport }
    }

    /// Reads a singular value from a given address on the LabJack.
    pub fn read<An, Reg>(
        &mut self,
        address: Reg,
        channel: An,
    ) -> Result<<An as Adc>::Digital, Either<Error, <T as Transport>::Error>>
    where
        An: Adc,
        Reg: Register,
    {
        let value = self
            .transport
            .read::<Reg>(&ReadFunction::InputRegister(address))
            .map_err(|e| Either::Right(e))?;

        // Utilising the ADC functions, so we convert it accordingly.
        let data = <Reg::DataType as Coerce>::coerce(value);
        Ok(channel.to_digital(data).into())
    }

    pub fn read_register<Reg>(
        &mut self,
        address: Reg,
    ) -> Result<<Reg::DataType as DataType>::Value, Either<Error, <T as Transport>::Error>>
    where
        Reg: Register,
    {
        self.transport
            .read::<Reg>(&ReadFunction::HoldingRegister(address))
            .map_err(|e| Either::Right(e))
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    /// A mocked DAQ used to override the values
    /// provided by conversions to test how the unit value operates.
    struct ButtEnd(LabJackDataValue);

    impl Adc for ButtEnd {
        type Digital = LabJackDataValue;

        fn to_digital(&self, _voltage: LabJackDataValue) -> Self::Digital {
            self.0
        }
    }

    impl Dac for ButtEnd {
        type Digital<'a> = LabJackDataValue;

        fn to_voltage(&self, _digital: Self::Digital<'_>) -> LabJackDataValue {
            self.0
        }
    }

    #[test]
    fn read_butt() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        let end = ButtEnd(LabJackDataValue::Uint16(100));
        let value = device.read(Ain55, end);

        assert!(value.is_ok(), "result={:?}", value);

        let value = value.unwrap();
        assert_eq!(value, LabJackDataValue::Uint16(100));
    }

    #[test]
    fn read_butt_no_filter() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        let value = device.read(Ain55, ());

        assert!(value.is_ok(), "result={:?}", value);

        let value = value.unwrap();
        assert_eq!(value.as_f64(), 0f64);
    }

    #[test]
    fn k() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        let value = device.read_register(Ain55).expect("!");
        println!("{:?}", value);
    }
}
