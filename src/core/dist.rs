use log::{debug, warn};
use crate::prelude::client::client::LabJackClient;
use crate::prelude::modbus::TcpTransport;
use super::{discover::Discover, modbus::Error, DeviceType, LabJackDevice, LabJackSerialNumber};

pub struct LabJack;

impl LabJack {
    pub fn connect(
        device_type: DeviceType,
        serial_number: LabJackSerialNumber,
    ) -> Result<LabJackDevice, Error> {
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

    pub fn connect_by_id(id: LabJackSerialNumber) -> Result<LabJackDevice, Error> {
        LabJack::connect(DeviceType::ANY, id)
    }

    ///
    /// connect::<Tcp>() style-?
    /// where Tcp: Connect
    /// so we have: fn connect<T>(serial) -> Result<Client, ...> where T: Connect { ... }
    /// or realistically; Transport*ABLE*.
    pub fn tcp_by_id(id: LabJackSerialNumber) -> Result<LabJackClient<TcpTransport>, Error> {
        Ok(LabJackClient::new(LabJack::connect_by_id(id)?)?)
    }
}
