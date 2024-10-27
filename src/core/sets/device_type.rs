use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum DeviceType {
    T4,
    T7,
    T8,
    TSERIES,
    DIGIT,
    ANY,
    EMULATED(i32),
    UNKNOWN(i32),
}

impl From<i32> for DeviceType {
    fn from(value: i32) -> Self {
        match value {
            4 => DeviceType::T4,
            7 => DeviceType::T7,
            8 => DeviceType::T8,
            200 => DeviceType::DIGIT,
            -999..=-1 => DeviceType::EMULATED(value),
            value => DeviceType::UNKNOWN(value),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DeviceType::T4 => "T4".to_string(),
            DeviceType::T7 => "T7".to_string(),
            DeviceType::T8 => "T8".to_string(),
            DeviceType::TSERIES => "TSERIES".to_string(),

            DeviceType::DIGIT => "DIGIT".to_string(),
            DeviceType::ANY => "ANY".to_string(),

            DeviceType::EMULATED(value) => format!("EMULATED::[{value}]"),
            DeviceType::UNKNOWN(value) => format!("ANY::[{value}]"),
        };

        write!(f, "{}", str)
    }
}