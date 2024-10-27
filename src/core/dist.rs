use crate::core::connection;

use super::{
    ConnectionType, DeviceType, LabJackDevice
};

pub struct LabJack;

impl LabJack {
    pub fn connect(
        device_type: DeviceType,
        connection_type: ConnectionType,
        id: i32,
    ) -> Option<LabJackDevice> {
        let devices = connection::discover::Discover::search(device_type, connection_type);
        devices.iter().find(|device| device.serial_number == id).copied()
    }
    
    pub fn connect_by_id(id: i32) -> Option<LabJackDevice> {
        LabJack::connect(DeviceType::ANY, ConnectionType::ANY, id)
    }
}
