use crate::core::LabJackDevice;
use crate::core::modbus::{Client, Error};
use crate::prelude::modbus::Transport;

pub type Connection<T> = Box<dyn Client<Error=<T as Transport>::Error>>;

pub trait Connect {
    type Transport: Transport;

    fn forge<'a>(device: LabJackDevice) -> Result<Connection<Self::Transport>, Error>;
}
