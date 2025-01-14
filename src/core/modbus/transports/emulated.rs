use std::collections::HashMap;
use std::time::Duration;

use crate::prelude::*;

#[derive(Clone, Debug)]
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
}

#[derive(Debug)]
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

    fn write(&mut self, function: WriteFunction) -> impl std::future::Future<Output = Result<(), Self::Error>> {
        async move {
            self.addresses
                .insert(function.0.address, EmulatedValue::transparent(function.1));
            Ok(())
        }
    }

    fn read(&mut self, function: ReadFunction) -> impl std::future::Future<Output = Result<LabJackDataValue, Self::Error>> {
        async move {
            let EmulatedValue {
                base: value,
                function: _,
            } = self
                .addresses
                .get(&function.0.address)
                .cloned()
                .unwrap_or(EmulatedValue::transparent(function.0.data_type.floating()));

            EmulatedDecoder { value }.decode_as(function.0.data_type)
        }
    }

    // fn feedback(&mut self, data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error> {
    //     todo!()
    // }
}

pub struct Emulated;

impl Connect for Emulated {
    type Transport = EmulatedTransport;

    async fn connect(device: LabJackDevice) -> Result<Self::Transport, Error> {
        Ok(EmulatedTransport::new(device))
    }
}
