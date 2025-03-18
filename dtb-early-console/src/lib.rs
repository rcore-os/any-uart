#![no_std]

pub use core::fmt::Write;
use core::ptr::NonNull;

use aux_mini::AuxMini;
use fdt_parser::Fdt;
use pl011::Pl011;

mod aux_mini;
mod pl011;

pub struct Sender {
    mmio: usize,
    f: fn(usize, u8),
}

pub struct Receiver {
    mmio: usize,
    f: fn(usize) -> u8,
}

impl Sender {
    pub fn put(&mut self, c: u8) {
        (self.f)(self.mmio, c);
    }
}

impl Write for Sender {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

pub trait Console {
    fn put(mmio: usize, c: u8);
    fn get(mmio: usize) -> u8;

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
