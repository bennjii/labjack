use crate::prelude::data_types::Register;
use crate::prelude::*;

#[test]
pub fn assert_correct_address() {
    assert_eq!(AIN55.address, 110);
    assert_eq!(AIN55.data_type, LabJackDataType::Float32);
}
