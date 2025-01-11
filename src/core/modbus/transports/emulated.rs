use std::collections::HashMap;
use std::time::Duration;

use crate::prelude::*;

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

    fn write(&mut self, function: WriteFunction) -> Result<(), Self::Error> {
        self.addresses
            .insert(function.0.address, EmulatedValue::transparent(function.1));
        Ok(())
    }

    fn read(&mut self, function: ReadFunction) -> Result<LabJackDataValue, Self::Error> {
        let EmulatedValue { base, function: _ } = self
            .addresses
            .get(&function.0.address)
            .unwrap_or(EmulatedValue::floating());

        EmulatedDecoder { value: *base }.decode_as(function.0.data_type)
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
