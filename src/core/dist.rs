use log::{debug, warn};

use super::{discover::Discover, modbus::Error, DeviceType, LabJackDevice};

pub struct LabJack;

impl LabJack {
    pub fn connect(device_type: DeviceType, serial_number: i32) -> Result<LabJackDevice, Error> {
        let devices = Discover::search()?;

        devices
            .filter_map(|device| match device {
                Err(error) => {
                    warn!("Failure retriving device, {:?}", error);
                    None
                }
                Ok(device) if device.device_type == device_type || device.device_type == DeviceType::ANY => Some(device),
                Ok(device) => {
                    debug!(
                        "Found LabJack with different device type to specified. Expected {}, got {}. Device: {}", 
                        device_type, device.device_type, device
                    );
                    None
                },
            })
            .find(|device| device.serial_number == serial_number)
            .ok_or(Error::DeviceNotFound)
    }

    pub fn connect_by_id(id: i32) -> Result<LabJackDevice, Error> {
        LabJack::connect(DeviceType::ANY, id)
    }
}
