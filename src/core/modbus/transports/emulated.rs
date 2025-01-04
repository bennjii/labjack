use std::collections::HashMap;
use std::time::Duration;
use byteorder::{BigEndian, WriteBytesExt};
use crate::prelude::*;

pub struct EmulatedValue {
    base: LabJackDataValue,
    function: fn(LabJackDataValue, Duration) -> LabJackDataValue,
}

impl EmulatedValue {
    fn transparent(base: LabJackDataValue) -> EmulatedValue {
        EmulatedValue {
            base,
            function: |a, _| a
        }
    }

    fn floating() -> &'static EmulatedValue {
        &EmulatedValue {
            base: LabJackDataValue::Uint16(0),
            function: |a, _| a
        }
    }
}

pub struct EmulatedTransport {
    addresses: HashMap<Address, EmulatedValue>,
    device: LabJackDevice
}

impl EmulatedTransport {
    fn new(device: LabJackDevice) -> EmulatedTransport {
        EmulatedTransport {
            addresses: HashMap::new(),
            device
        }
    }
}

impl Transport for EmulatedTransport {
    type Error = Error;

    fn write(&mut self, function: &WriteFunction) -> Result<(), Self::Error> {
        match function {
            WriteFunction::SingleRegister(addr, val) => {
                self.addresses.insert(*addr, EmulatedValue::transparent(*val));
            }
            WriteFunction::MultipleRegisters(addr, values) => {
                for (index, value) in values.into_iter().enumerate() {
                    self.addresses.insert(*addr + index as Address, EmulatedValue::transparent(*value));
                }
            }
        }

        Ok(())
    }

    fn read(&mut self, function: &ReadFunction) -> Result<Box<[u8]>, Self::Error> {
        match function {
            ReadFunction::InputRegisters(addr, quantity)
            | ReadFunction::HoldingRegisters(addr, quantity) => {
                let mut total = 0;
                let mut bytes = vec![];

                for q in 0..*quantity {
                    let addr = (addr + q) as Address;
                    let EmulatedValue { base, function: _ } = self.addresses.get(&addr)
                        .unwrap_or(EmulatedValue::floating());

                    total += base.r#type().size();
                    match base {
                        LabJackDataValue::Uint16(v) => bytes.write_u16::<BigEndian>(*v)?,
                        LabJackDataValue::Uint32(v) => bytes.write_u32::<BigEndian>(*v)?,
                        LabJackDataValue::Float32(v) => bytes.write_f32::<BigEndian>(*v)?,
                        LabJackDataValue::Int32(v) => bytes.write_i32::<BigEndian>(*v)?,
                    }
                }

                let mut total = vec![total as u8];
                total.extend(bytes);

                Ok(Box::from(total.as_slice()))
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

    fn connect(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error> {
        Ok(Box::new(EmulatedTransport::new(device)))
    }
}
