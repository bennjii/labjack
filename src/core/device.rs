use super::{ConnectionType, DeviceType, LabJack};
use crate::prelude::discover::MODBUS_COMMUNICATION_PORT;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Deref;

pub const EMULATED_DEVICE_SERIAL_NUMBER: i32 = -2;

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
pub struct LabJackSerialNumber(pub i32);

impl LabJackSerialNumber {
    pub fn is_emulated(&self) -> bool {
        self.0 == EMULATED_DEVICE_SERIAL_NUMBER
    }

    pub fn emulated() -> LabJackSerialNumber {
        LabJackSerialNumber(EMULATED_DEVICE_SERIAL_NUMBER)
    }
}

impl From<i32> for LabJackSerialNumber {
    fn from(value: i32) -> Self {
        LabJackSerialNumber(value)
    }
}

impl Deref for LabJackSerialNumber {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LabJackDevice {
    pub device_type: DeviceType,
    pub connection_type: ConnectionType,
    pub ip_address: std::net::IpAddr,

    pub serial_number: LabJackSerialNumber,
    pub port: u16,
}

impl Display for LabJackDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // DT on CT @ 000.000.000:0000 => SERIAL_NUMBER
        write!(
            f,
            "{} on {} @ {}:{} => Serial({:?})",
            self.device_type, self.connection_type, self.ip_address, self.port, self.serial_number
        )
    }
}

impl LabJackDevice {
    /// Creates an emulated LabJack device that is used to run tests against.
    pub fn emulated() -> LabJackDevice {
        LabJackDevice {
            device_type: DeviceType::EMULATED(EMULATED_DEVICE_SERIAL_NUMBER),
            connection_type: ConnectionType::ETHERNET,
            ip_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            serial_number: LabJackSerialNumber::emulated(),
            port: MODBUS_COMMUNICATION_PORT,
        }
    }

    /// Used to create a [`LabJackDevice`] given the `ip`, `device_type` and `serial_number`
    /// are known beforehand. This allows for static definition of devices, skipping the resolution/discovery
    /// step in connecting to the device.
    ///
    /// Commonly paired with the [`LabJack::connect_with`] method.
    pub fn known(
        ip: IpAddr,
        device_type: DeviceType,
        serial: impl Into<LabJackSerialNumber>,
    ) -> LabJackDevice {
        LabJackDevice {
            connection_type: ConnectionType::ETHERNET,
            port: MODBUS_COMMUNICATION_PORT,
            ip_address: ip,
            device_type,
            serial_number: serial.into(),
        }
    }
}
