use std::fmt::{Display, Formatter};

use super::{ConnectionType, DeviceType};

#[derive(Clone, Copy, Debug)]
pub struct LabJackDevice {
    pub device_type: DeviceType,
    pub connection_type: ConnectionType,
    pub ip_address: std::net::IpAddr,

    pub serial_number: i32,
    pub port: u16,
}

impl Display for LabJackDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // DT on CT @ 000.000.000:0000 => SERIAL_NUMBER
        write!(
            f,
            "{} on {} @ {}:{} => {}",
            self.device_type,
            self.connection_type,
            self.ip_address,
            self.port,
            self.serial_number
        )
    }
}