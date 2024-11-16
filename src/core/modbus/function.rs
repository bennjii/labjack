pub type Address = u16;
pub type Quantity = u16;
pub type Value = u16;

pub enum ModbusFeedbackFunction<'a> {
    ReadRegisters(Address, u8),
    WriteRegisters(Address, &'a [u8]),
}

pub enum Function<'a> {
    ReadHoldingRegisters(Address, Quantity),
    ReadInputRegisters(Address, Quantity),

    WriteRegister(Address, Value),
    WriteRegisters(Address, Quantity, &'a [u8]),

    // Feedback can
    Feedback(&'a [ModbusFeedbackFunction<'a>]),
}

impl<'a> Function<'a> {
    pub(crate) fn code(&self) -> u8 {
        match *self {
            Function::ReadHoldingRegisters(_, _) => 0x03,
            Function::ReadInputRegisters(_, _) => 0x04,

            Function::WriteRegister(_, _) => 0x06,
            Function::WriteRegisters(_, _, _) => 0x10,

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
