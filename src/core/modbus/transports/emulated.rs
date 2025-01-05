use crate::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::core::data_types::{Decode, EmulatedDecoder};
use crate::prelude::data_types::Register;

pub struct EmulatedValue {
    base: LabJackDataValue,
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

    fn write(&mut self, function: &WriteFunction) -> Result<(), Self::Error> {
        match function {
            WriteFunction::SingleRegister(addr, val) => {
                self.addresses
                    .insert(*addr, EmulatedValue::transparent(*val));
            }
            WriteFunction::MultipleRegisters(addr, values) => {
                for (index, value) in values.into_iter().enumerate() {
                    self.addresses
                        .insert(*addr + index as Address, EmulatedValue::transparent(*value));
                }
            }
        }

        Ok(())
    }

    fn read<R>(&mut self, function: &ReadFunction<R>) -> Result<<R::DataType as DataType>::Value, Self::Error>
    where
        R: Register
    {
        match function {
            ReadFunction::InputRegister(register)
            | ReadFunction::HoldingRegister(register) => {
                let EmulatedValue { base, function: _ } = self
                    .addresses
                    .get(&R::ADDRESS)
                    .unwrap_or(EmulatedValue::floating());

                <R::DataType as Decode>::try_decode(EmulatedDecoder { value: *base })
            }
        }
    }

    fn feedback(&mut self, _data: &[FeedbackFunction]) -> Result<Box<[u8]>, Self::Error> {
        unimplemented!()
    }
}

pub struct Emulated;

impl Connect for Emulated {
    type Transport = EmulatedTransport;

    fn connect(device: LabJackDevice) -> Result<Self::Transport, Error> {
        Ok(EmulatedTransport::new(device))
    }
}
