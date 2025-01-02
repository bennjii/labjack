use std::time::Instant;
use log::info;

use labjack::prelude::*;

fn main() {
    env_logger::init();

    let time = Instant::now();
    let device = LabJack::connect_by_id(LabJackSerialNumber(470033971))
        .expect("Failed to connect to LabJack device");

    info!(
        "Connected to a device on {}:{}",
        device.device.ip_address, device.device.port
    );
    info!("Took: {}ms", time.elapsed().as_millis());
}
