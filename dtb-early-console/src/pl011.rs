use core::sync::atomic::{Ordering, fence};

use crate::Console;

pub struct Pl011 {}

impl Console for Pl011 {
    fn put(base: usize, byte: u8) {
        const TXFF: u8 = 1 << 5;

        unsafe {
            let state = (base + 0x18) as *mut u8;
            let put = (base) as *mut u8;
            while (state.read_volatile() & TXFF) != 0 {}
            fence(Ordering::SeqCst);
            put.write_volatile(byte);
        }
    }
    
    fn get(mmio: usize) -> u8 {
        todo!()
    }
}
