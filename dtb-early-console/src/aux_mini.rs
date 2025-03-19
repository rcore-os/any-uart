use core::sync::atomic::{Ordering, fence};

use crate::{Console, Error, UartData};

pub struct AuxMini {}

impl Console for AuxMini {
    fn put(uart: UartData, byte: u8) -> Result<(), Error> {
        const TXFF: u32 = 1 << 5;

        unsafe {
            let state = uart.reg_u8(0x24) as *const u32;

            if state.read_volatile() & TXFF != 0 {
                return Err(Error::WouldBlock);
            }
            fence(Ordering::Release);
            let data = uart.reg::<u32>(0);
            data.write_volatile(byte as _);

            Ok(())
        }
    }

    fn get(uart: UartData) -> Result<u8, Error> {
        const RX_READY: u32 = 1 << 0;
        let state = uart.reg_u8(0x24) as *const u32;

        // Wait until there is data in the FIFO
        unsafe {
            if state.read_volatile() & RX_READY == 0 {
                return Err(Error::WouldBlock);
            }
            let data = uart.reg::<u32>(0);

            Ok(data.read_volatile() as _)
        }
    }
}
