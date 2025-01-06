use crate::prelude::data_types::Register;
use crate::prelude::*;

#[test]
pub fn assert_correct_address() {
    assert_eq!(Ain55.address(), 110);
    assert_eq!(Ain55.data_type().data_type(), LabJackDataType::Float32);
}
