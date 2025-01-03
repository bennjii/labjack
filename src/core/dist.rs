use log::{debug, warn};

use crate::prelude::*;

pub struct LabJack;

impl LabJack {
    pub fn discover(
        device_type: DeviceType,
        serial_number: LabJackSerialNumber,
    ) -> Result<LabJackDevice, Error> {
        let devices = Discover::search_all()?;

        devices
            .filter_map(|device| match device {
                Err(error) => {
                    warn!("Failure retrieving device, {:?}", error);
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

    pub fn discover_with_id(id: LabJackSerialNumber) -> Result<LabJackDevice, Error> {
        if id.is_emulated() {
            return Ok(LabJackDevice::emulated())
        }

        LabJack::discover(DeviceType::ANY, id)
    }

    pub fn connect<T>(id: LabJackSerialNumber) -> Result<LabJackClient<<T as Connect>::Transport>, Error> where T: Connect {
        let device = LabJack::discover_with_id(id)?;
        let transport = T::forge(device)?;

        Ok(LabJackClient::new(device, transport))
    }
}
