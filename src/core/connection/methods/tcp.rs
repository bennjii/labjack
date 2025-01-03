use std::net::{SocketAddr, TcpStream};
use crate::core::modbus::tcp::TcpTransport;
use crate::prelude::*;

pub struct Tcp;

impl Connect for Tcp {
    type Transport = TcpTransport;

    fn forge(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error>{
        let addr = SocketAddr::new(device.ip_address, MODBUS_COMMUNICATION_PORT);
        let stream = TcpStream::connect(addr).map_err(Error::Io)?;

        Ok(Box::new(TcpTransport::new(stream)))
    }
}