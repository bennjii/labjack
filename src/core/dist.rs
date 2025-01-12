use log::{debug, warn};

use crate::prelude::*;

pub struct LabJack;

impl LabJack {
    pub fn discover(
        device_type: DeviceType,
        serial_number: LabJackSerialNumber,
    ) -> Result<LabJackDevice, Error> {
        let devices = Discover::search_all()?;
        let serial = serial_number.into();

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
            .find(|device| device.serial_number == serial)
            .ok_or(Error::DeviceNotFound)
    }

    pub fn discover_with_id(id: LabJackSerialNumber) -> Result<LabJackDevice, Error> {
        if id.is_emulated() {
            return Ok(LabJackDevice::emulated());
        }

        LabJack::discover(DeviceType::ANY, id)
    }

    pub fn connect<T>(
        id: impl Into<LabJackSerialNumber>,
    ) -> Result<LabJackClient<<T as Connect>::Transport>, Error>
    where
        T: Connect,
    {
        let serial = id.into();
        let device = if serial.is_emulated() {
            LabJackDevice::emulated()
        } else {
            LabJack::discover_with_id(serial)?
        };

        let transport = T::connect(device)?;
        Ok(LabJackClient::new(device, transport))
    }
}
