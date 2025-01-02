use std::time::Instant;
use log::info;

use labjack::prelude::*;

fn main() {
    env_logger::init();

    let time = Instant::now();

    // Example for looking for a specific LabJack, using its serial number.
    let device = LabJack::discover_with_id(LabJackSerialNumber(470033971))
        .expect("Failed to connect to LabJack device");

    info!(
        "Found a device on {}:{}",
        device.ip_address, device.port
    );
    info!("Took: {}ms", time.elapsed().as_millis());
}
