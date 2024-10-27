use crate::prelude::*;

#[test]
pub fn assert_correct_address() {
    assert_eq!(translate::LookupTable::Ain55.raw(), (110, 3));
}
