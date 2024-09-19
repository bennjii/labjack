use crate::prelude::*;

#[test]
fn test_mod() {
    let from_raw = translate::ADDRESSES.get("AIN55");
    let from_crate = LabJack::name_to_address("AIN55");

    assert_eq!(*from_raw.unwrap(), from_crate.unwrap())
}
