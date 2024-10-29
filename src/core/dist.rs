use log::warn;

use super::{
    discover::Discover, modbus::Error, ConnectionType, DeviceType, LabJackDevice
};

pub struct LabJack;

impl LabJack {
    pub fn connect(
        device_type: DeviceType,
        connection_type: ConnectionType,
        id: i32,
    ) -> Result<LabJackDevice, Error> {
        let devices = Discover::search(device_type, connection_type)?;

        devices
            .filter_map(|device| {
                match device {
                    Err(error) => {
                        warn!("Failure retriving device, {:?}", error);
                        None
                    },
                    Ok(device) => Some(device)
                }
            })
            .find(|device| device.serial_number == id)
            .ok_or(Error::DeviceNotFound)
    }

    pub fn connect_by_id(id: i32) -> Result<LabJackDevice, Error> {
        LabJack::connect(DeviceType::ANY, ConnectionType::ANY, id)
    }
}
