use core::sync::atomic::{Ordering, fence};

use crate::Console;

pub struct AuxMini {}

impl Console for AuxMini {
    fn put(&self, base: usize, byte: u8) {
        const TXFF: u32 = 1 << 5;

        unsafe {
            let state = (base + 0x24) as *const u32;
            loop {
                let lsr = state.read_volatile();

                fence(Ordering::Release);
                if lsr & TXFF == 0 {
                    break;
                }
            }
            let data = base as *mut u32;
            data.write_volatile(byte as _);
        }
    }
}
