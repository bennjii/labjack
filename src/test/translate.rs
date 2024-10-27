use crate::prelude::*;

#[test]
pub fn assert_correct_address() {
    let labjack_entity = translate::LookupTable::Ain55.raw();
    assert_eq!(labjack_entity.address, 110);
    assert_eq!(labjack_entity.data_type, LabJackDataType::Float32);
}
