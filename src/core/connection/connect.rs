use crate::core::modbus::{Client, Error};
use crate::core::LabJackDevice;
use crate::prelude::modbus::Transport;
use std::io::Sink;

pub type Connection<T> = Box<dyn Client<Error = <T as Transport>::Error>>;

pub trait Connect {
    type Transport: Transport;

    fn connect<'a>(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error>;

    fn sink() -> Result<Sink, Error> {
        todo!()
    }
}
