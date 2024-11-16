use std::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum ConnectionType {
    USB,
    ETHERNET,
    WIFI,
    ANY,
    UNKNOWN(i32),
}

impl From<i32> for ConnectionType {
    fn from(value: i32) -> Self {
        match value {
            1 => ConnectionType::USB,
            3 => ConnectionType::ETHERNET,
            4 => ConnectionType::WIFI,
            value => ConnectionType::UNKNOWN(value),
        }
    }
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ConnectionType::USB => "USB",
            ConnectionType::WIFI => "WIFI",
            ConnectionType::ETHERNET => "ETHERNET",
            ConnectionType::ANY | ConnectionType::UNKNOWN(_) => "ANY",
        }
        .to_string();
        write!(f, "{}", str)
    }
}
