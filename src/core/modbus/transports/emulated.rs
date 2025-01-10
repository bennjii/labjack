use crate::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::core::data_types::{Decode, EmulatedDecoder, StandardDecoder};
use crate::prelude::data_types::{Coerce, Register};

pub struct EmulatedValue {
    base: LabJackDataValue,
    #[allow(dead_code)]
    function: fn(LabJackDataValue, Duration) -> LabJackDataValue,
}

impl EmulatedValue {
    fn transparent(base: LabJackDataValue) -> EmulatedValue {
        EmulatedValue {
            base,
            function: |a, _| a,
        }
    }

    fn floating() -> &'static EmulatedValue {
        &EmulatedValue {
            base: LabJackDataValue::Uint16(0),
            function: |a, _| a,
        }
    }
}

pub struct EmulatedTransport {
    addresses: HashMap<Address, EmulatedValue>,
    device: LabJackDevice,
}

impl EmulatedTransport {
    fn new(device: LabJackDevice) -> EmulatedTransport {
        EmulatedTransport {
            addresses: HashMap::new(),
            device,
        }
    }
}

impl Transport for EmulatedTransport {
    type Error = Error;

    fn write<R>(&mut self, function: &WriteFunction<R>) -> Result<(), Self::Error>
    where
        R: Register,
    {
        let data_value = function.0.data_type().coerce(function.1.clone());
        // let data_value = <R::DataType as Coerce>::coerce(function.1.clone());
        self.addresses
            .insert(function.0.address(), EmulatedValue::transparent(data_value));
        Ok(())
    }

    fn read<R>(
        &mut self,
        function: &ReadFunction<R>,
    ) -> Result<<R::DataType as DataType>::Value, Self::Error>
    where
        R: Register,
    {
        match function {
            ReadFunction(reg) => {
                let EmulatedValue { base, function: _ } = self
                    .addresses
                    .get(&reg.address())
                    .unwrap_or(EmulatedValue::floating());

                function
                    .0
                    .data_type()
                    .try_decode(&EmulatedDecoder { value: *base })
                // <R::DataType as Decode>::try_decode(EmulatedDecoder { value: *base })
            }
        }
    }

    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error> {
    //     todo!()
    // }
}

pub struct Emulated;

impl Connect for Emulated {
    type Transport = EmulatedTransport;

    fn connect(device: LabJackDevice) -> Result<Self::Transport, Error> {
        Ok(EmulatedTransport::new(device))
    }
}
