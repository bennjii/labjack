use crate::prelude::translate::ADDRESSES;
use std::string::ToString;

pub struct LabJack;

impl LabJack {
    pub fn name_to_address<T>(identifier: T) -> Option<u32>
    where
        T: ToString,
    {
        ADDRESSES.get(identifier.to_string().as_str()).cloned()
    }
}
