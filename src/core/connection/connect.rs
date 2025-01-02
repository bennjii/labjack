use std::net::{SocketAddr, TcpStream};
use crate::core::discover::MODBUS_COMMUNICATION_PORT;
use crate::core::LabJackDevice;
use crate::core::modbus::{Client, Error};
use crate::prelude::modbus::{TcpTransport, Transport};

type Connection<T> = Box<dyn Client<Error=<T as Transport>::Error>>;

pub trait Connect {
    type Transport: Transport;

    fn forge<'a>(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error>;
}

// scratch impl; remove.

pub struct Tcp;

impl Connect for Tcp {
    type Transport = TcpTransport;


    fn forge(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error>{
        // TODO: Somehow emulate the device
        // if device.serial_number.is_emulated() {
        //     return Box::new(TcpTransport::new(TcpStream::connect()));
        // }

        let addr = SocketAddr::new(device.ip_address, MODBUS_COMMUNICATION_PORT);
        let stream = TcpStream::connect(addr).map_err(Error::Io)?;

        Ok(Box::new(TcpTransport::new(stream)))
    }
}