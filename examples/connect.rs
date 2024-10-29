use std::time::Instant;

use labjack::core::LabJack;
use log::info;

fn main() {
    env_logger::init();
    
    let time = Instant::now();
    let device = LabJack::connect_by_id(470033971)
        .expect("Failed to connect to LabJack device");

    info!(
        "Connected to a device on {}:{}",
        device.ip_address, device.port
    );
    info!("Took: {}ms", time.elapsed().as_millis());
}
