use log::{debug, warn};

use crate::prelude::*;

/// The entry-point structure for accessing and using LabJack functionality.
///
/// This structure does not need to be initialised, but is rather a namespace for helper methods
/// to connect and discover LabJack devices.
///
/// An example of connecting to an emulated device is shown below, note the [turbofish](https://rust.code-maven.com/turbofish)
/// syntax used in specifying the [`Emulated`] transport.
///
/// ```rust
/// use labjack::prelude::*;
///
/// let emulated = LabJackSerialNumber::emulated();
/// let mut device = LabJack::connect::<Emulated>(emulated).expect("Must connect");
///
/// println!("Connected to device {:?}", device);
/// ```
///
/// Once connected, use the utility methods from the returned [`LabJackDevice`] structure to
/// interact with your LabJack. This means functionality is *instanced*. By contrast, the
/// LJM library uses handle identification for each connected device, which is decentralised.
/// This does not support the same guarantees as instancing, which is why this approach has been
/// preferred in this case.
///
/// ## Transports
///
/// Below is a list of the provided transports. This allows you to select the method you wish
/// to use to connect to a device.
///
/// - [`Tcp`].
///     Used to connect over Ethernet. Wi-Fi is supported over this measure but is not recommended.
///     See the [`MAX_DATA_LENGTH`] for why.
///
/// - [`Emulated`].
///     Allows for testing behaviour without a device present. Similar to the [Demo Mode](https://support.labjack.com/docs/open-ljm-user-s-guide#Open[LJMUser'sGuide]-Identifier[in]) connection.
///     Therefore, does not require a device present. Not fully-featured, but can be used for unit and integration testing.
///
///  > Notice there is no `Usb` transport. This is not yet supported. You are welcome to contribute if you require this feature.
///
pub struct LabJack;

impl LabJack {
    /// Allows you to discover devices, filtered by a device type.
    ///
    /// ```
    /// // Write an example.
    /// use labjack::prelude::*;
    ///
    /// let devices = LabJack::discover(DeviceType::T7).expect("Could not start broadcast");
    /// devices.for_each(|device| {
    ///     println!("Found device {}", device);
    /// });
    /// ```
    pub fn discover(device_type: DeviceType) -> Result<impl Iterator<Item = LabJackDevice>, Error> {
        let devices = Discover::search_all()?;

        Ok(devices.filter_map(move |device| match device {
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
        }))
    }

    /// Discovers LabJack device with a given serial number. This returns the [`LabJackDevice`]
    /// if found, otherwise an appropriate error. Note: There is a 10s timeout.
    pub fn discover_with_id(serial_number: LabJackSerialNumber) -> Result<LabJackDevice, Error> {
        if serial_number.is_emulated() {
            return Ok(LabJackDevice::emulated());
        }

        LabJack::discover(DeviceType::ANY)?
            .find(|device| device.serial_number == serial_number)
            .ok_or(Error::DeviceNotFound)
    }

    /// The preferred way to connect to a LabJack device.
    ///
    /// This returns the usable [`LabJackClient`] structure with the appropriated transport,
    /// given the [`LabJackSerialNumber`] parameter. This contains the connected [`LabJackDevice`]
    /// inside.
    ///
    /// ```
    /// use labjack::prelude::*;
    ///
    /// // Connect to a device with an emulated Serial Number, over Tcp.
    /// let mut device = LabJack::connect::<Emulated>(LabJackSerialNumber::emulated())
    ///     .expect("Must connect");
    ///
    /// println!("Connected to device {:?}", device);
    /// ```
    ///
    /// If you have obtained a [`LabJackDevice`] from any discovery method, you may
    /// instead choose to skip this step, and connect directly using the [`LabJack::connect_with`] method.
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

    /// Connects to a device using the specified transport, given a device has already been located.
    ///
    /// ```rust
    /// use labjack::prelude::*;
    ///
    /// let devices = LabJack::discover(DeviceType::T7).expect("Could not start broadcast");
    /// devices.for_each(|device| {
    ///     // After the device has been located, we can connect to it using `Emulated`, however `Tcp` should be used in practice.
    ///     let connected_device = LabJack::connect_with::<Emulated>(device).expect("Could not connect to device");
    ///     println!("Connected to device {:?}", connected_device);
    /// });
    /// ```
    ///
    /// This allows the consumer to skip the location step if the device
    /// location is known beforehand. This device can be created using the
    /// [`LabJackDevice::known`] method.
    ///
    /// ```
    /// use std::net::IpAddr;
    /// use std::str::FromStr;
    /// use labjack::prelude::*;
    ///
    /// // Can be set on the LabJack as a static IP address.
    /// // Note: This can be set using the `ETHERNET_IP_DEFAULT` register.
    /// let known_ip = IpAddr::from_str("192.168.1.25").expect("Must resolve");
    /// let known_device = LabJackDevice::known(known_ip, DeviceType::TSERIES, 470000000);
    ///
    /// let connected = LabJack::connect_with::<Emulated>(known_device);
    /// println!("Connected to known device {:?}", connected);
    /// ```
    ///
    pub fn connect_with<T>(
        device: LabJackDevice,
    ) -> Result<LabJackClient<<T as Connect>::Transport>, Error>
    where
        T: Connect,
    {
        let transport = T::connect(device)?;
        Ok(LabJackClient::new(device, transport))
    }
}
