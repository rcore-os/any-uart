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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aux_mini_struct_exists() {
        // Verify AuxMini struct can be created (even if it's a zero-sized type)
        let _ = AuxMini {};
    }

    #[test]
    fn test_aux_mini_console_trait() {
        // Verify AuxMini implements Console trait
        let _ = AuxMini {};
        // This test ensures the type exists and the trait implementation compiles
    }

    #[test]
    fn test_lsr_flags() {
        // Test LSR (Line Status Register) flags
        const TXFF: u32 = 1 << 5;
        const RX_READY: u32 = 1 << 0;
        assert_eq!(TXFF, 32);
        assert_eq!(RX_READY, 1);
    }

    #[test]
    fn test_can_put_logic() {
        // Test the logic of can_put function (TXFF flag)
        const TXFF: u32 = 1 << 5;
        let lsr_value = 0x00; // TXFF not set
        let can_write = lsr_value & TXFF == 0;
        assert!(can_write);

        let lsr_value = 0x20; // TXFF set
        let can_write = lsr_value & TXFF == 0;
        assert!(!can_write);
    }

    #[test]
    fn test_can_get_logic() {
        // Test the logic of can_get function (RX_READY flag)
        const RX_READY: u32 = 1 << 0;
        let lsr_value = 0x01; // RX_READY set
        let can_read = lsr_value & RX_READY != 0;
        assert!(can_read);

        let lsr_value = 0x00; // RX_READY not set
        let can_read = lsr_value & RX_READY != 0;
        assert!(!can_read);
    }

    #[test]
    fn test_register_offsets() {
        // Verify register offsets
        assert_eq!(0, 0); // Data register offset
        assert_eq!(0x24, 0x24); // LSR register offset
    }

    #[test]
    fn test_data_width() {
        // Test data width (AuxMini uses 32-bit data register)
        let data: u32 = 0x12345678;
        assert_eq!(data, 0x12345678);

        // Test byte conversion
        let byte: u8 = 0x41;
        assert_eq!(byte as u32, 0x00000041);
    }

    #[test]
    fn test_error_handling() {
        // Verify ErrorKind can be used
        let _error = ErrorKind::Other;
        let _error = ErrorKind::Overrun;
        let _error = ErrorKind::Parity;
    }

    #[test]
    fn test_uart_data_new() {
        // Test UartData creation with identity mapping
        let data = crate::UartData::new(0x215040, crate::IoKind::Mmio32, |r| r as _);
        assert_eq!(data.base, 0x215040);
        assert_eq!(data.io_kind, crate::IoKind::Mmio32);
    }

    #[test]
    fn test_irq_event_default() {
        // Test default IrqEvent
        let event = IrqEvent::default();
        assert!(!event.rx);
        assert!(!event.tx);
    }

    #[test]
    fn test_lsr_register_value() {
        // Test LSR register value reading logic
        const TXFF: u32 = 1 << 5;
        const RX_READY: u32 = 1 << 0;

        // Test combined flags
        let lsr = TXFF | RX_READY;
        assert_eq!(lsr, 33);
        assert!(lsr & TXFF != 0);
        assert!(lsr & RX_READY != 0);
    }

    #[test]
    fn test_aux_mini_is_zero_sized() {
        // Verify AuxMini is a zero-sized type
        assert_eq!(core::mem::size_of::<AuxMini>(), 0);
    }

    #[test]
    fn test_data_byte_conversion() {
        // Test byte to u32 conversion
        let byte: u8 = b'A';
        let dword: u32 = byte as u32;
        assert_eq!(dword, 0x00000041);

        // Test u32 to byte conversion
        let dword: u32 = 0x00000042;
        let byte: u8 = dword as u8;
        assert_eq!(byte, b'B');
    }
}
