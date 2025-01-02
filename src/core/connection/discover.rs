//! We need to be able to discover the labjack device on the network, we
//! can do this through UDP broadcast.
//!
//! Support seen [here](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails%5BDirectModbusTCP%5D-ReadT-SeriesProductID(Searchnetworkforadevice)).
//! UDP Broadcast is shown to be used internally
//! by LJM's `ListAll` function, which is the
//! logical equivalent we are aiming to replicate.

use std::net::UdpSocket;
use std::time::Duration;

use log::debug;

use crate::{
    core::{
        modbus::{Error, ModbusFeedbackFunction, TcpCompositor},
        ConnectionType, DeviceType, LabJackDevice, LabJackSerialNumber,
    },
    prelude::translate,
};

pub const BROADCAST_IP: &str = "192.168.255.255";
pub const MODBUS_FEEDBACK_PORT: u16 = 52362;
pub const MODBUS_COMMUNICATION_PORT: u16 = 502;

pub struct Discover;

impl Discover {
    pub fn search_all() -> Result<impl Iterator<Item = Result<LabJackDevice, Error>>, Error> {
        // Send broadcast request.
        let broadcast = Discover::broadcast(Duration::from_secs(10))?;
        let mut transaction_id = 0;
        let mut compositor = TcpCompositor::new(&mut transaction_id, 1);

        let product_id_addr = translate::LookupTable::ProductId.raw().address as u16;
        let serial_number_addr = translate::LookupTable::SerialNumber.raw().address as u16;

        let read_product_id = ModbusFeedbackFunction::ReadRegisters(product_id_addr, 2);
        let read_serial_number = ModbusFeedbackFunction::ReadRegisters(serial_number_addr, 2);

        let (buf, _, _) = compositor.compose_feedback(&[read_product_id, read_serial_number])?;
        broadcast.send_to(&buf, (BROADCAST_IP, MODBUS_FEEDBACK_PORT))?;

        // Collect all devices from the UDP broadcast
        Ok(std::iter::from_fn(move || {
            let mut buf = [0u8; 1024];
            match broadcast.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    debug!("Some LabJack Found! PacketSize={}, Addr={}", size, addr);

                    // Device ID's taken from the LabJack UDP broadcast docs:
                    // https://support.labjack.com/docs/protocol-details-direct-modbus-tcp?search=product%20id#ProtocolDetails%5BDirectModbusTCP%5D-ReadT-SeriesProductID(Searchnetworkforadevice)
                    let device_type = match <[u8; 4]>::try_from(&buf[8..12]) {
                        Ok([0x41, 0x00, 0x00, 0x00]) => DeviceType::T8,
                        Ok([0x40, 0xE0, 0x00, 0x00]) => DeviceType::T7,
                        Ok([0x40, 0x80, 0x00, 0x00]) => DeviceType::T4,
                        Ok(const_sized) => DeviceType::UNKNOWN(i32::from_be_bytes(const_sized)),
                        Err(err) => {
                            eprint!("Could not decode LabJack device type: {}", err);
                            DeviceType::UNKNOWN(0)
                        }
                    };

                    let serial_number =
                        LabJackSerialNumber(match <[u8; 4]>::try_from(&buf[12..16]) {
                            Ok(serial) => i32::from_be_bytes(serial),
                            Err(error) => {
                                eprint!("Could not decode LabJack serial number: {}", error);
                                0
                            }
                        });

                    Some(Ok(LabJackDevice {
                        ip_address: addr.ip(),
                        port: addr.port(),
                        device_type,
                        serial_number,
                        // Only supports ethernet for now.
                        connection_type: ConnectionType::ETHERNET,
                    }))
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => None,
                Err(error) => Some(Err(Error::Io(error))),
            }
        }))
    }

    pub fn search() -> Result<impl Iterator<Item = LabJackDevice>, Error> {
        match Self::search_all() {
            Ok(iter) => Ok(iter.filter_map(|item| item.ok())),
            Err(error) => Err(error),
        }
    }

    fn broadcast(duration: Duration) -> Result<UdpSocket, std::io::Error> {
        let socket = UdpSocket::bind(("0.0.0.0", 0))?;
        debug!("Listening through ephemeral: {}", socket.local_addr()?);

        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(duration))?;
        Ok(socket)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        core::modbus::{ModbusFeedbackFunction, TcpCompositor},
        prelude::translate,
    };

    // Feedback Response:
    //       Echo     Len  UID Fn      Data
    //    +--------+  +--+  +  +   +-----------+
    // => 0, 1, 0, 0, 0, 6, 1, 76, 64, 224, 0, 0
    // Therefore, we recieve: [64, 224, 0, 0].
    // That is: 0x40E00000 = 1088421888
    // Which is the LabJack Product ID.

    #[test]
    fn feedback_function() {
        let mut transaction_id: u16 = 0;
        let mut compositor = TcpCompositor::new(&mut transaction_id, 1);
        let product_id_addr = translate::LookupTable::ProductId.raw().address as u16;

        let read_product_id = ModbusFeedbackFunction::ReadRegisters(product_id_addr, 2);
        let (buf, _, _) = compositor
            .compose_feedback(&[read_product_id])
            .expect("Could not compose ModbusFeedback message");

        let as_be = transaction_id.to_be_bytes();

        // Transaction Identifier (arbitrary)
        assert_eq!(buf[0..2], as_be[..]);
        assert_eq!(
            buf[2..],
            vec![
                0x00, 0x00, // Protocol Identifier (Modbus TCP/IP)
                0x00, 0x06, // Length (6 bytes to follow)
                0x01, // Unit Identifier (slave address, usually 1)
                0x4c, // Function Code (Read Holding Registers)
                0x00, // Frame Type
                0xEA, 0x60, // Starting Register (60,000 = T7 Product ID)
                0x02, // Quantity of Registers (2)
            ]
        )
    }
}
