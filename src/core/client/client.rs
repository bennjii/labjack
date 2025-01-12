use crate::prelude::*;

use either::Either;
use enum_primitive::FromPrimitive;
use std::any::Any;

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
    pub fn read<An>(
        &mut self,
        address: Register,
        channel: An,
    ) -> Result<<An as Adc>::Digital, Either<Error, <T as Transport>::Error>>
    where
        An: Adc,
    {
        let value = self.read_register(address)?;
        Ok(channel.to_digital(value).into())
    }

    pub fn read_register(
        &mut self,
        address: Register,
    ) -> Result<LabJackDataValue, Either<Error, <T as Transport>::Error>> {
        self.transport
            .read_register(address)
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
        let value = device.read(*AIN55, end);

        assert!(value.is_ok(), "result={:?}", value);

        let value = value.unwrap();
        assert_eq!(value, LabJackDataValue::Uint16(100));
    }

    #[test]
    fn read_butt_no_filter() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        let value = device.read(*AIN55, ());

        assert!(value.is_ok(), "result={:?}", value);

        let value = value.unwrap();
        assert_eq!(value.as_f64(), 0f64);
    }

    #[test]
    fn read_singular() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        let value = device.read_register(*AIN55).expect("!");
        println!("{:?}", value);
    }

    #[test]
    fn read_many() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        // Static-Typing will only go so far.
        //
        // See the below where we can aggregate on registers with a common data type.
        // We could not perform this same aggregation if there would be a discrepancy
        // in their data types. For that, we would need a layer of indirection and an
        // aggregated type to represent the multi-typed result.

        // let registers: Vec<&dyn Register<DataType = Float32>> = vec![&Ain55, &Ain56];
        //
        // for register in registers {
        //     let value = device.read_register(&register).expect("!");
        //     println!("{:?}", value);
        // }
    }

    #[test]
    fn read_many_indirected() {
        let mut device =
            LabJack::connect::<Emulated>(LabJackSerialNumber::emulated()).expect("Must connect");

        // We can opt for indirection, through use of enumerations.
        // Meaning, we specify the `LookupTable` entry, which is Sized
        // and can therefore group any differently-sized registers together
        // and read them all back into data value (enumerated) variants.
        let registers = vec![*AIN55, *AIN56];

        for register in registers.into_iter() {
            let value = device.read(register, ()).expect("!");
            println!("{:?}", value);

            // But if we needed to unionise the values
            // into some specific target type, we can simply
            // implement that for the data value.
            //
            // For example, the `as_f64` method...
            println!("AsF64={}", value.as_f64())
        }
    }
}
