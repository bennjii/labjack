use super::{ConnectionType, DeviceType, LabJackDataValue};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::{SocketAddr, TcpStream};
use std::ops::Deref;
use crate::core::discover::MODBUS_COMMUNICATION_PORT;
use crate::prelude::modbus::{Error, TcpTransport};

const EMULATED_DEVICE_SERIAL_NUMBER: i32 = -2;

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
    pub fn transport(&self) -> Result<TcpTransport, Error> {
        let addr = SocketAddr::new(self.ip_address, MODBUS_COMMUNICATION_PORT);
        let stream = TcpStream::connect(addr).map_err(Error::Io)?;

        Ok(TcpTransport::new(stream))
    }

    pub fn is_emulated(&self) -> bool {
        self.serial_number.is_emulated()
    }
}
