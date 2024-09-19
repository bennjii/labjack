use labjack::prelude::*;
use std::time::Instant;

pub fn main() {
    let now = Instant::now();
    let ain55_reg = translate::ADDRESSES.get("AIN55");
    let ain55_ef = translate::ADDRESSES.get("AIN55_EF_CONFIG_A");
    println!("{:#?} <=> {:#?}", ain55_reg, ain55_ef);
    println!("Elapsed Time: {}", now.elapsed().as_millis());

    LabJack::name_to_address("AIN55");
}
