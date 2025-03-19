use core::sync::atomic::{Ordering, fence};

use super::{Error, ErrorKind};

use crate::{Console, UartData};

pub struct Pl011 {}

impl Console for Pl011 {
    fn put(uart: UartData, byte: u8) -> Result<(), Error> {
        const TXFF: u8 = 1 << 5;

        unsafe {
            if uart.reg_u8(0x18).read_volatile() & TXFF != 0 {
                return Err(Error::WouldBlock);
            }
            let put = uart.reg_u8(0);
            fence(Ordering::SeqCst);
            put.write_volatile(byte);
            Ok(())
        }
    }

    fn get(uart: UartData) -> Result<u8, Error> {
        const RXFE: u8 = 0x10;

        unsafe {
            if uart.reg_u8(0x18).read_volatile() & RXFE != 0 {
                return Err(Error::WouldBlock);
            }

            let data = uart.reg::<u32>(0).read_volatile();

            if data & 0xFFFFFF00 != 0 {
                // Clear the error
                uart.reg::<u32>(1).write_volatile(0xFFFFFFFF);
                return Err(Error::Other(ErrorKind::Other));
            }

            Ok(data as _)
        }
    }
}
