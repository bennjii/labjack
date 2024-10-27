/// We need to be able to discover the
/// labjack device on the network, we
/// can do this
///
use modbus::tcp;
use modbus::{Client, Coil};

use crate::core::{ConnectionType, DeviceType, LabJackDevice};

pub struct Discover;

impl Discover {
    pub fn search(_device_type: DeviceType, _connection_type: ConnectionType) -> Vec<LabJackDevice> {
        let cfg = tcp::Config::default();
        let mut client = tcp::Transport::new_with_cfg("127.0.0.1", cfg).unwrap();

        assert!(client.write_single_coil(0, Coil::On).is_ok());
        todo!();
    }
}
