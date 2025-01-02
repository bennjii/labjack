use std::io;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    /// Modbus exception codes returned from the server.
    pub enum ExceptionCode {
        IllegalFunction         = 0x01,
        IllegalDataAddress      = 0x02,
        IllegalDataValue        = 0x03,
        SlaveOrServerFailure    = 0x04,
        Acknowledge             = 0x05,
        SlaveOrServerBusy       = 0x06,
        NegativeAcknowledge     = 0x07,
        MemoryParity            = 0x08,
        NotDefined              = 0x09,
        GatewayPath             = 0x0a,
        GatewayTarget           = 0x0b
    }
}

#[derive(Debug)]
pub enum Reason {
    UnexpectedReplySize,
    BytecountNotEven,
    SendBufferEmpty,
    RecvBufferEmpty,
    SendBufferTooBig,
    DecodingError,
    EncodingError,
    InvalidByteorder,
    RegisterMismatch,
    Custom(String),
}

impl From<Reason> for Error {
    fn from(reason: Reason) -> Error {
        Error::InvalidData(reason)
    }
}

#[derive(Debug)]
pub enum Error {
    Exception(ExceptionCode),
    Io(io::Error),
    InvalidResponse,
    InvalidData(Reason),
    InvalidFunction,
    ParseCoilError,
    ParseInfoError,
    DeviceNotFound,
}

impl From<ExceptionCode> for Error {
    fn from(err: ExceptionCode) -> Error {
        Error::Exception(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
