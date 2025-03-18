use core::sync::atomic::{Ordering, fence};

use super::{Error, ErrorKind};

use crate::Console;

pub struct Pl011 {}

impl Console for Pl011 {
    fn put(base: usize, byte: u8) -> Result<(), Error> {
        const TXFF: u8 = 1 << 5;

        unsafe {
            let state = (base + 0x18) as *mut u8;
            if state.read_volatile() & TXFF != 0 {
                return Err(Error::WouldBlock);
            }
            let put = (base) as *mut u8;
            fence(Ordering::SeqCst);
            put.write_volatile(byte);
            Ok(())
        }
    }

    fn get(base: usize) -> Result<u8, Error> {
        const RXFE: u8 = 0x10;

        unsafe {
            let state = (base + 0x18) as *mut u8;
            if state.read_volatile() & RXFE != 0 {
                return Err(Error::WouldBlock);
            }

            let data = (base as *mut u32).read_volatile();

            if data & 0xFFFFFF00 != 0 {
                // Clear the error
                ((base + 0x4) as *mut u32).write_volatile(0xFFFFFFFF);
                return Err(Error::Other(ErrorKind::Other));
            }

            Ok(data as _)
        }
    }
}
