use crate::core::connection;

use super::{
    modbus::Error, ConnectionType, DeviceType, LabJackDevice
};

pub struct LabJack;

impl LabJack {
    pub fn connect(
        device_type: DeviceType,
        connection_type: ConnectionType,
        id: i32,
    ) -> Result<LabJackDevice, Error> {
        let devices = connection::discover::Discover::search(device_type, connection_type)?;
        devices.iter().find(|device| device.serial_number == id).copied().ok_or(Error::DeviceNotFound)
    }
    
    pub fn connect_by_id(id: i32) -> Result<LabJackDevice, Error> {
        LabJack::connect(DeviceType::ANY, ConnectionType::ANY, id)
    }
}
