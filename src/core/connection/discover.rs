//! We need to be able to discover the labjack device on the network, we
//! can do this through UDP broadcast.
//!
//! Support seen [here](https://support.labjack.com/docs/protocol-details-direct-modbus-tcp#ProtocolDetails%5BDirectModbusTCP%5D-ReadT-SeriesProductID(Searchnetworkforadevice)).
//! UDP Broadcast is shown to be used internally
//! by LJM's `ListAll` function, which is the
//! logical equivalent we are aiming to replicate.

use std::net::UdpSocket;
use std::time::Duration;

use crate::core::{
    modbus::{Error, ModbusFeedbackFunction, TcpCompositor},
    ConnectionType, DeviceType, LabJackDevice,
};

const BROADCAST_IP: &str = "255.255.255.255";
const MODBUS_PORT: u16 = 502;

pub struct Discover;

impl Discover {
    pub fn search(
        _device_type: DeviceType,
        _connection_type: ConnectionType,
    ) -> Result<Vec<LabJackDevice>, Error> {
        // Send broadcast request.
        let broadcast = Discover::broadcast(Duration::from_secs(2))?;
        let mut transaction_id = 0;
        let mut compositor = TcpCompositor::new(&mut transaction_id, 1);

        let read_product_id = ModbusFeedbackFunction::ReadRegisters(0xEA60, 1);
        let (buf, _, _) = compositor.compose_feedback(&[read_product_id])?;

        broadcast.send_to(&buf, (BROADCAST_IP, MODBUS_PORT))?;

        // Collect all devices from the
        std::iter::from_fn(|| {
            let mut buf = [0u8; 1024];
            match broadcast.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    println!("LabJack Found! PacketSize={}, Addr={}", size, addr);
                    println!("=> Content={:?}", buf);

                    Some(Ok(LabJackDevice {
                        ip_address: addr.ip(),
                        port: addr.port(),
                        device_type: DeviceType::ANY,
                        connection_type: ConnectionType::ANY,
                        max_bytes_per_megabyte: 0,
                        serial_number: 0,
                    }))
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => None,
                Err(error) => Some(Err(Error::Io(error))),
            }
        })
        .collect::<Result<Vec<_>, _>>()
    }

    fn broadcast(duration: Duration) -> Result<UdpSocket, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(duration))?;
        Ok(socket)
    }
}

#[cfg(test)]
mod test {
    use crate::core::{
        discover::FEEDBACK_FUNCTION,
        modbus::{ModbusFeedbackFunction, TcpCompositor},
    };

    #[test]
    fn feedback_function() {
        let mut transaction_id = 0;
        let mut compositor = TcpCompositor::new(&mut transaction_id, 1);

        let read_product_id = ModbusFeedbackFunction::ReadRegisters(0xEA60, 2);
        let (buf, _, _) = compositor
            .compose_feedback(&[read_product_id])
            .expect("Could not compose MBFB message");

        assert_eq!(
            buf,
            vec![
                0x00, 0x01, // Transaction Identifier (arbitrary)
                0x00, 0x00, // Protocol Identifier (Modbus TCP/IP)
                0x00, 0x06, // Length (6 bytes to follow)
                0x01, // Unit Identifier (slave address, usually 1)
                FEEDBACK_FUNCTION, // Function Code (Read Holding Registers)
                0x00, // Frame Type
                0xEA, 0x60, // Starting Register (60,000 = T7 Product ID)
                0x02, // Quantity of Registers (2)
            ]
        )
    }
}
