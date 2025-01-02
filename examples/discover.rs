use std::time::Instant;
use log::info;

use labjack::prelude::*;

fn main() {
    env_logger::init();

    let time = Instant::now();

    // Example for looking for any labjack available to connect using UDP.
    let search = Discover::search().expect("!");

    search.for_each(|device| {
        info!(
            "Found a device on {}:{}",
            device.ip_address, device.port
        );
    });

    info!("Search concluded in {}ms", time.elapsed().as_millis());
}
