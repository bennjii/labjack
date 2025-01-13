use crate::core::modbus::Error;
use crate::core::LabJackDevice;
use crate::prelude::modbus::Transport;
use std::io::Sink;

pub trait Connect {
    type Transport: Transport;

    async fn connect(device: LabJackDevice) -> Result<Self::Transport, Error>;

    fn sink() -> Result<Sink, Error> {
        todo!()
    }
}
