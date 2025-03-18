use core::sync::atomic::{Ordering, fence};

use crate::{Console, Error};

pub struct AuxMini {}

impl Console for AuxMini {
    fn put(base: usize, byte: u8) -> Result<(), Error> {
        const TXFF: u32 = 1 << 5;

        unsafe {
            let state = (base + 0x24) as *const u32;

            if state.read_volatile() & TXFF != 0 {
                return Err(Error::WouldBlock);
            }
            fence(Ordering::Release);
            let data = base as *mut u32;
            data.write_volatile(byte as _);

            Ok(())
        }
    }

    fn get(mmio: usize) -> Result<u8, Error> {
        const RX_READY: u32 = 1 << 0;
        let state = (mmio + 0x24) as *const u32;

        // Wait until there is data in the FIFO
        unsafe {
            if state.read_volatile() & RX_READY == 0 {
                return Err(Error::WouldBlock);
            }
            let data = mmio as *mut u32;

            Ok(data.read_volatile() as _)
        }
    }
}
