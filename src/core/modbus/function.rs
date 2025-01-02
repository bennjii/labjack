pub type Address = u16;
pub type Quantity = u16;
pub type Value = u16;

pub enum ModbusFeedbackFunction<'a> {
    ReadRegisters(Address, u8),
    WriteRegisters(Address, &'a [u8]),
}

pub enum Function<'a> {
    Read(ReadFunction),
    Write(WriteFunction<'a>),
    Feedback(&'a [ModbusFeedbackFunction<'a>]),
}

pub enum WriteFunction<'a> {
    SingleRegister(Address, Value),
    MultipleRegisters(Address, Quantity, &'a [u8]),
}

pub enum ReadFunction {
    HoldingRegisters(Address, Quantity),
    InputRegisters(Address, Quantity),
}

impl ReadFunction {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            ReadFunction::HoldingRegisters(..) => 0x03,
            ReadFunction::InputRegisters(..) => 0x04,
        }
    }
}

impl WriteFunction<'_> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            WriteFunction::SingleRegister(..) => 0x06,
            WriteFunction::MultipleRegisters(..) => 0x10,
        }
    }
}

impl<'a> Function<'a> {
    pub(crate) fn code(&self) -> u8 {
        match self {
            Function::Read(a) => a.code(),
            Function::Write(a) => a.code(),
            Function::Feedback(_) => 0x4C,
        }
    }
}

impl<'a> ModbusFeedbackFunction<'a> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            ModbusFeedbackFunction::ReadRegisters(_, _) => 0x00,
            ModbusFeedbackFunction::WriteRegisters(_, _) => 0x01,
        }
    }
}
