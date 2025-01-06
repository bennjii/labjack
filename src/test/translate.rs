use crate::prelude::data_types::Register;
use crate::prelude::*;

#[test]
pub fn assert_correct_address() {
    assert_eq!(Ain55::ADDRESS, 110);
    assert_eq!(
        <<Ain55 as Register>::DataType as DataType>::data_type(),
        LabJackDataType::Float32
    );
}
