#![no_std]

use core::ptr::NonNull;

pub use core::fmt::Write;
pub use embedded_hal_nb::nb::block;
pub use embedded_hal_nb::serial::ErrorKind;

use aux_mini::AuxMini;
use fdt_parser::{Chosen, Fdt};
use ns16550::Ns16550;
use pl011::Pl011;

mod aux_mini;
mod ns16550;
mod pl011;

pub type Error = embedded_hal_nb::nb::Error<ErrorKind>;

pub struct Sender {
    uart: UartData,
    f: fn(UartData, u8) -> Result<(), Error>,
}

impl Sender {
    pub fn write(&mut self, word: u8) -> Result<(), Error> {
        (self.f)(self.uart, word)
    }

    pub fn write_str_blocking(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            let _ = block!(self.write(c));
        }
        Ok(())
    }
}

pub struct Receiver {
    uart: UartData,
    f: fn(UartData) -> Result<u8, Error>,
}

impl Receiver {
    pub fn read(&mut self) -> Result<u8, Error> {
        (self.f)(self.uart)
    }
}

pub(crate) trait Console {
    fn put(uart: UartData, c: u8) -> Result<(), Error>;
    fn get(uart: UartData) -> Result<u8, Error>;

    fn to_uart(uart: UartData) -> (Sender, Receiver) {
        (
            Sender { uart, f: Self::put },
            Receiver { uart, f: Self::get },
        )
    }
}

pub fn init(fdt_addr: NonNull<u8>) -> Option<(Sender, Receiver)> {
    let fdt = Fdt::from_ptr(fdt_addr).ok()?;

    let chosen = fdt.chosen()?;

    if let Some(u) = fdt_stdout(&chosen) {
        return Some(u);
    }

    fdt_bootargs(&chosen)
}

#[derive(Clone, Copy)]
pub(crate) struct UartData {
    pub base: usize,
    pub io_kind: IoKind,
}

impl UartData {
    pub fn reg_u8(&self, offset: usize) -> *mut u8 {
        self.reg(offset)
    }

    pub fn reg<T: Sized>(&self, offset: usize) -> *mut T {
        unsafe {
            let ptr = self.base as *mut T;
            ptr.add(offset)
        }
    }
}

fn fdt_stdout(chosen: &Chosen<'_>) -> Option<(Sender, Receiver)> {
    let stdout = chosen.stdout()?;
    let reg = stdout.node.reg()?.next()?;
    let io_kind = IoKind::Mmio32;
    let mmio = reg.address as usize;
    let uart = UartData {
        base: mmio,
        io_kind,
    };

    for c in stdout.node.compatibles() {
        macro_rules! of_uart {
            ($name:ty, $compatible:expr) => {
                for want in $compatible {
                    if c.contains(want) {
                        return Some(<$name>::to_uart(uart));
                    }
                }
            };
        }

        of_uart!(AuxMini, &["brcm,bcm2835-aux-uart"]);
        of_uart!(Pl011, &["arm,pl011", "arm,primecell"]);
    }

    None
}

#[derive(Clone, Copy)]
pub enum IoKind {
    Port,
    Mmio,
    Mmio16,
    Mmio32,
    Mmio32be,
}

impl IoKind {
    pub fn width(&self) -> usize {
        match self {
            IoKind::Port => 1,
            IoKind::Mmio => 4,
            IoKind::Mmio16 => 2,
            IoKind::Mmio32 => 4,
            IoKind::Mmio32be => 4,
        }
    }
}

impl From<&str> for IoKind {
    fn from(value: &str) -> Self {
        match value {
            "mmio" => IoKind::Mmio,
            "mmio16" => IoKind::Mmio16,
            "mmio32" => IoKind::Mmio32,
            "mmio32be" => IoKind::Mmio32be,
            "mmio32native" => {
                if cfg!(target_endian = "little") {
                    IoKind::Mmio32
                } else {
                    IoKind::Mmio32be
                }
            }
            _ => IoKind::Port,
        }
    }
}

fn fdt_bootargs(chosen: &Chosen<'_>) -> Option<(Sender, Receiver)> {
    let bootargs = chosen.bootargs()?;

    let earlycon = bootargs
        .split_ascii_whitespace()
        .find(|&arg| arg.contains("earlycon"))?;

    let mut tmp = earlycon.split('=');
    let _ = tmp.next()?;
    let values = tmp.next()?;

    let mut values = values.split(',');

    let name = values.next()?;

    if !name.contains("uart") {
        return None;
    }

    let param2 = values.next()?;
    let addr_str;
    let io_kind = if param2.contains("0x") {
        addr_str = param2;
        IoKind::Mmio
    } else {
        addr_str = values.next()?;
        IoKind::from(param2)
    };

    let mmio = u64::from_str_radix(addr_str, 16).ok()? as usize;
    let uart = UartData {
        base: mmio,
        io_kind,
    };

    if name.contains("8250") || name.contains("16550") {
        return Some(Ns16550::to_uart(uart));
    }

    None
}
