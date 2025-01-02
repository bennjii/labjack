use either::Either;
use crate::{core::conversion::daq::Daq, prelude::LookupTable, core::modbus::Client};
use crate::core::LabJackDataValue;
use crate::core::modbus::ReadFunction;
use crate::prelude::adc::Adc;
use crate::prelude::LabJackDevice;
use crate::prelude::modbus::{Error, Reason, Transport};

pub struct LabJackClient<T> where T: Transport {
    pub device: LabJackDevice,
    transport: Box<dyn Client<Error=T::Error>>,
}

impl<T> LabJackClient<T> where T: Transport {
    pub fn new(device: LabJackDevice, transport: Box<dyn Client<Error=T::Error>>) -> LabJackClient<T> {
        LabJackClient { device, transport }
    }

    /// Reads a singular value from a given address on the LabJack.
    pub fn read<D>(&mut self, channel: D, address: LookupTable) -> Result<<D as Daq>::Digital, Either<Error, <T as Transport>::Error>>
    where
        D: Daq,
    {
        let entity = address.raw();
        let expected_registers = entity.size() * 2;

        let bytes = self.transport.read(
            &ReadFunction::InputRegisters(entity.address as u16, entity.size())
        ).map_err(|e| Either::Right(e))?;

        let num_registers = bytes[0];
        if num_registers != expected_registers as u8 {
            return Err(Either::Left(Reason::RegisterMismatch.into()));
        }

        let value = LabJackDataValue::from_bytes(entity.data_type, &bytes[1..])
            .map_err(|e| Either::Left(e))?;

        // Utilising the ADC functions, so we convert it accordingly.
        let channel_value = <D as Adc>::Voltage::from(value);
        Ok(channel.to_digital(channel_value).into())
    }
}

#[cfg(test)]
mod test {
    use crate::core::{LabJackDataValue, LabJackSerialNumber};
    use crate::prelude::{LabJack};
    use crate::prelude::adc::Adc;
    use crate::prelude::connect::Tcp;
    use crate::prelude::dac::Dac;
    use crate::prelude::LookupTable::Ain55;

    /// A mocked DAQ used to override the values
    /// provided by conversions to test how the unit value operates.
    struct ButtEnd(LabJackDataValue);

    impl Adc for ButtEnd {
        type Digital = LabJackDataValue;
        type Voltage<'a> = f64;

        fn to_digital(&self, _voltage: Self::Voltage<'_>) -> Self::Digital {
            self.0
        }
    }

    impl Dac for ButtEnd {
        type Digital<'a> = LabJackDataValue;
        type Voltage<'a> = f64;

        fn to_voltage(&self, _digital: Self::Digital<'_>) -> Self::Voltage<'_> {
            self.0.as_f64()
        }
    }

    #[test]
    fn read_butt() {
        let mut device = LabJack::connect::<Tcp>(LabJackSerialNumber::emulated())
            .expect("Must connect");

        let end = ButtEnd(LabJackDataValue::Uint16(100));
        let value = device.read(end, Ain55);

        assert!(value.is_ok());

        let value = value.unwrap();
        assert!(value == LabJackDataValue::Uint16(100));
    }
}
