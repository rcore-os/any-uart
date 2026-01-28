//! BCM2835 Auxiliary UART (Aux Mini) driver implementation.
//!
//! This module implements the auxiliary UART controller driver for Raspberry Pi BCM2835 SoC.

use crate::{Console, ErrorKind, IrqEvent, UartData};

/// BCM2835 auxiliary UART controller.
///
/// This is the auxiliary UART used on Raspberry Pi BCM2835 SoC. Unlike the main PL011 UART,
/// it is typically used for lower-speed communication scenarios.
///
/// # Device Tree Identification
///
/// This controller is identified by the `"brcm,bcm2835-aux-uart"` compatible string.
pub struct AuxMini {}

impl Console for AuxMini {
    /// Writes a byte to the UART.
    fn put(uart: UartData, byte: u8) -> Result<(), ErrorKind> {
        unsafe {
            let data = uart.reg::<u32>(0);
            data.write_volatile(byte as _);
            Ok(())
        }
    }

    /// Reads a byte from the UART.
    fn get(uart: UartData) -> Result<u8, ErrorKind> {
        unsafe {
            let data = uart.reg::<u32>(0);

            Ok(data.read_volatile() as _)
        }
    }

    /// Sets interrupt enable status (currently not implemented).
    fn set_irq_enable(_uart: UartData, _enable: bool) {}

    /// Gets interrupt enable status (currently not implemented).
    fn get_irq_enable(_uart: UartData) -> bool {
        todo!()
    }

    /// Clears interrupt event (currently not implemented).
    fn clean_irq_event(_uart: UartData, _event: IrqEvent) {}

    /// Checks if data can be written.
    ///
    /// Checks the TXFF flag of the AUX_MU_LSR register.
    fn can_put(uart: UartData) -> bool {
        const TXFF: u32 = 1 << 5;
        let state = uart.reg_u8(0x24) as *const u32;
        unsafe { state.read_volatile() & TXFF == 0 }
    }

    /// Checks if data is available to read.
    ///
    /// Checks the RX_READY flag of the AUX_MU_LSR register.
    fn can_get(uart: UartData) -> bool {
        const RX_READY: u32 = 1 << 0;
        let state = uart.reg_u8(0x24) as *const u32;
        unsafe { state.read_volatile() & RX_READY != 0 }
    }

    /// Gets interrupt event (currently not implemented).
    fn get_irq_event(_uart: UartData) -> IrqEvent {
        todo!()
    }

    /// Initializes the UART controller (currently empty implementation).
    fn open(_uart: UartData) {}
}
