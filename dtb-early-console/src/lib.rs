#![no_std]

use core::ptr::NonNull;

pub use core::fmt::Write;
pub use embedded_hal_nb::nb::block;
pub use embedded_hal_nb::serial::ErrorKind;

use aux_mini::AuxMini;
use fdt_parser::Fdt;
use pl011::Pl011;

mod aux_mini;
mod pl011;

pub type Error = embedded_hal_nb::nb::Error<ErrorKind>;

pub struct Sender {
    mmio: usize,
    f: fn(usize, u8) -> Result<(), Error>,
}

impl Sender {
    pub fn write(&mut self, word: u8) -> Result<(), Error> {
        (self.f)(self.mmio, word)
    }

    pub fn write_str_blocking(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            let _ = block!(self.write(c));
        }
        Ok(())
    }
}

pub struct Receiver {
    mmio: usize,
    f: fn(usize) -> Result<u8, Error>,
}

impl Receiver {
    pub fn read(&mut self) -> Result<u8, Error> {
        (self.f)(self.mmio)
    }
}

pub trait Console {
    fn put(mmio: usize, c: u8) -> Result<(), Error>;
    fn get(mmio: usize) -> Result<u8, Error>;

    fn to_uart(mmio: usize) -> (Sender, Receiver) {
        (
            Sender { mmio, f: Self::put },
            Receiver { mmio, f: Self::get },
        )
    }
}

pub fn init(fdt_addr: NonNull<u8>) -> Option<(Sender, Receiver)> {
    let fdt = Fdt::from_ptr(fdt_addr).ok()?;

    if let Some(u) = fdt_stdout(&fdt) {
        return Some(u);
    }

    None
}

fn fdt_stdout(fdt: &Fdt<'_>) -> Option<(Sender, Receiver)> {
    let stdout = fdt.chosen()?.stdout()?;
    let reg = stdout.node.reg()?.next()?;

    let mmio = reg.address as usize;
    for c in stdout.node.compatibles() {
        if c.contains("brcm,bcm2835-aux-uart") {
            return Some(AuxMini::to_uart(mmio));
        }

        if c.contains("arm,pl011") || c.contains("arm,primecell") {
            return Some(Pl011::to_uart(mmio));
        }
    }

    None
}
