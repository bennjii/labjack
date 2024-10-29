use std::time::Instant;

use labjack::core::LabJack;

fn main() {
    let time = Instant::now();
    let device = LabJack::connect_by_id(470033971)
        .expect("Failed to connect to LabJack device");

    println!(
        "Connected to a device on {}:{}",
        device.ip_address, device.port
    );
    println!("Took: {}ms", time.elapsed().as_millis());
}
