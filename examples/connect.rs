use labjack::core::LabJack;

fn main() {
    let device = LabJack::connect_by_id(470031743).expect("Failed to connect to LabJack device");

    println!(
        "Connected to a device on {}:{}",
        device.ip_address, device.port
    );
}
