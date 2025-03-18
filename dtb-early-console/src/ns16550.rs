use crate::Console;

pub struct Ns16550 {}

impl Console for Ns16550 {
    fn put(mmio: usize, c: u8) -> Result<(), crate::Error> {
        todo!()
    }

    fn get(mmio: usize) -> Result<u8, crate::Error> {
        todo!()
    }
}
