use core::sync::atomic::{Ordering, fence};

use crate::{Console, Error, IoKind, UartData};

pub struct Ns16550 {}

impl Ns16550 {
    fn sts(uart: UartData) -> u32 {
        Self::read(uart, 5)
    }

    fn write(uart: UartData, reg: usize, val: u32) {
        unsafe {
            match uart.io_kind {
                IoKind::Port => {
                    uart.reg::<u8>(reg).write_volatile(val as _);
                }
                IoKind::Mmio16 => {
                    uart.reg::<u16>(reg).write_volatile(val as _);
                }
                IoKind::Mmio32 | IoKind::Mmio => {
                    uart.reg::<u32>(reg).write_volatile(val);
                }
                IoKind::Mmio32be => {
                    uart.reg::<u32>(reg).write_volatile(val.to_be());
                }
            }
        }
    }

    fn read(uart: UartData, reg: usize) -> u32 {
        unsafe {
            match uart.io_kind {
                IoKind::Port => uart.reg::<u8>(reg).read_volatile() as _,
                IoKind::Mmio16 => uart.reg::<u16>(reg).read_volatile() as _,
                IoKind::Mmio32 | IoKind::Mmio => uart.reg::<u32>(reg).read_volatile(),
                IoKind::Mmio32be => {
                    let reg = uart.reg::<u32>(reg);
                    let val = reg.read_volatile();
                    u32::from_be(val)
                }
            }
        }
    }
}

impl Console for Ns16550 {
    fn put(uart: UartData, c: u8) -> Result<(), crate::Error> {
        // Xmitter empty
        const LSR_TEMT: u32 = 1 << 6;
        if Self::sts(uart) & LSR_TEMT == 0 {
            return Err(Error::WouldBlock);
        }
        fence(Ordering::SeqCst);
        Self::write(uart, 0, c as _);
        Ok(())
    }

    fn get(uart: UartData) -> Result<u8, crate::Error> {
        const LSR_DR: u32 = 1;

        if Self::sts(uart) & LSR_DR == 0 {
            return Err(Error::WouldBlock);
        }

        fence(Ordering::SeqCst);
        Ok(Self::read(uart, 0) as _)
    }
}
