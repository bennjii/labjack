use crate::core::modbus::Error;
use crate::core::LabJackDevice;
use crate::prelude::modbus::Transport;
use std::io::Sink;

pub trait Connect {
    type Transport: Transport;

    fn connect(
        device: LabJackDevice,
    ) -> impl std::future::Future<Output = Result<Self::Transport, Error>> + Send;

    fn sink() -> Result<Sink, Error> {
        todo!()
    }
}
