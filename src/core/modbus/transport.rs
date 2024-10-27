use super::Function;

pub trait Transport {
    type Error;

    fn write(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn read(&mut self, function: &Function) -> Result<Box<[u8]>, Self::Error>;
}
