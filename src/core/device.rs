use super::{ConnectionType, DeviceType};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
pub struct LabJackSerialNumber(pub i32);

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
