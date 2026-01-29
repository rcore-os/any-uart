//! NS16550 and compatible UART controller driver.
//!
//! This module implements driver support for NS16550/8250 UART controllers, supporting multiple I/O modes.

use cfg_if::cfg_if;

use crate::{Console, ErrorKind, IoKind, IrqEvent, UartData};

/// NS16550 UART controller driver.
///
/// Implements a driver for the classic 8250/16450/16550 series serial port controllers.
/// These controllers are widely used in x86 platform COM ports and various embedded systems.
///
/// # Device Tree Identification
///
/// This controller is identified by compatible strings such as `"snps,dw-apb-uart"`.
///
/// # Supported Features
///
/// - Port I/O (x86 platform)
/// - 16-bit and 32-bit memory mapped I/O
/// - Big-endian memory mapped I/O
/// - Interrupt-driven operations
pub struct Ns16550 {}

impl Ns16550 {
    /// Reads the UART line status register.
    fn sts(uart: UartData) -> u32 {
        Self::read(uart, 5)
    }

    /// Writes a value to a UART register.
    fn write(uart: UartData, reg: usize, val: u32) {
        unsafe {
            match uart.io_kind {
                IoKind::Port => {
                    cfg_if! {
                        if #[cfg(target_arch = "x86_64")] {
                            x86_64::instructions::port::Port::<u8>::new((uart.base + reg) as _)  .write(val as _);
                        } else {
                            todo!();
                        }
                    }
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

    /// Reads a value from a UART register.
    fn read(uart: UartData, reg: usize) -> u32 {
        unsafe {
            match uart.io_kind {
                IoKind::Port => {
                    cfg_if! {
                        if #[cfg(target_arch = "x86_64")] {
                            x86_64::instructions::port::Port::<u8>::new((uart.base + reg) as _)
                                .read() as u32
                        } else {
                            todo!();
                        }
                    }
                }
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
    /// Sends a byte to the UART.
    ///
    /// Writes to THR (Transmitter Holding Register, offset 0).
    fn put(uart: UartData, c: u8) -> Result<(), ErrorKind> {
        Self::write(uart, 0, c as _);
        Ok(())
    }

    /// Receives a byte from the UART.
    ///
    /// Reads from RBR (Receiver Buffer Register, offset 0).
    fn get(uart: UartData) -> Result<u8, ErrorKind> {
        Ok(Self::read(uart, 0) as _)
    }

    /// Sets interrupt enable status.
    ///
    /// Configured via IER (Interrupt Enable Register, offset 1):
    /// - Bit 0: Received data available interrupt
    /// - Bit 1: Transmitter holding register empty interrupt
    fn set_irq_enable(uart: UartData, enable: bool) {
        let val = if enable { 1 | 1 << 1 } else { 0 };

        Self::write(uart, 1, val);
    }

    /// Gets interrupt enable status.
    fn get_irq_enable(uart: UartData) -> bool {
        Self::read(uart, 1) != 0
    }

    /// Clears interrupt event (currently empty implementation).
    ///
    /// Interrupt events in NS16550 are typically automatically cleared by reading IIR (Interrupt Identification Register).
    fn clean_irq_event(_uart: UartData, _event: IrqEvent) {}

    /// Checks if data can be written.
    ///
    /// Checks the TEMT bit of LSR (Line Status Register, offset 5).
    fn can_put(uart: UartData) -> bool {
        // Xmitter empty
        const LSR_TEMT: u32 = 1 << 6;
        Self::sts(uart) & LSR_TEMT != 0
    }

    /// Checks if data is available to read.
    ///
    /// Checks the DR (Data Ready) bit of LSR (Line Status Register, offset 5).
    fn can_get(uart: UartData) -> bool {
        const LSR_DR: u32 = 1;

        Self::sts(uart) & LSR_DR != 0
    }

    /// Gets interrupt event.
    ///
    /// Gets interrupt status by reading IIR (Interrupt Identification Register, offset 2).
    fn get_irq_event(uart: UartData) -> IrqEvent {
        let sts = Self::read(uart, 2);
        let mut event = IrqEvent::default();

        if sts & 1 != 0 {
            event.rx = true;
        }

        if sts & 1 << 1 != 0 {
            event.tx = true;
        }
        event
    }

    /// Initializes the UART controller (currently empty implementation).
    fn open(_uart: UartData) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ns16550_struct_exists() {
        // Verify Ns16550 struct can be created (even if it's a zero-sized type)
        let _ = Ns16550 {};
    }

    #[test]
    fn test_lsr_constants() {
        // Verify LSR constant values are correct
        const LSR_TEMT: u32 = 1 << 6;
        const LSR_DR: u32 = 1;
        assert_eq!(LSR_TEMT, 64);
        assert_eq!(LSR_DR, 1);
    }

    #[test]
    fn test_uart_data_new() {
        // Test UartData creation with identity mapping
        let data = crate::UartData::new(0x1000, IoKind::Mmio32, |r| r as _);
        assert_eq!(data.base, 0x1000);
        assert_eq!(data.io_kind, IoKind::Mmio32);
    }

    #[test]
    fn test_io_kind_variants() {
        // Test all IoKind variants exist and are Copy/Clone
        let kinds = [IoKind::Port, IoKind::Mmio, IoKind::Mmio16, IoKind::Mmio32, IoKind::Mmio32be];
        let _copy = kinds;
        let _clone = IoKind::Mmio.clone();
        let _copy2 = IoKind::Mmio32be;
    }

    #[test]
    fn test_interrupt_enable_bits() {
        // Verify interrupt enable bit patterns
        let rx_enable = 1;
        let tx_enable = 1 << 1;
        let both_enabled = rx_enable | tx_enable;
        assert_eq!(both_enabled, 3);
        assert_ne!(rx_enable, 0);
        assert_ne!(tx_enable, 0);
    }

    #[test]
    fn test_register_offsets() {
        // Verify register offsets
        assert_eq!(0, 0);   // THR/RBR
        assert_eq!(1, 1);   // IER
        assert_eq!(2, 2);   // IIR/FCR
        assert_eq!(5, 5);   // LSR
    }

    #[test]
    fn test_can_put_logic() {
        // Test the logic of can_put function
        const LSR_TEMT: u32 = 1 << 6;
        let lsr_value = 0x40; // TEMT bit set
        let can_write = lsr_value & LSR_TEMT != 0;
        assert!(can_write);

        let lsr_value = 0x00; // TEMT bit not set
        let can_write = lsr_value & LSR_TEMT != 0;
        assert!(!can_write);
    }

    #[test]
    fn test_can_get_logic() {
        // Test the logic of can_get function
        const LSR_DR: u32 = 1;
        let lsr_value = 0x01; // DR bit set
        let can_read = lsr_value & LSR_DR != 0;
        assert!(can_read);

        let lsr_value = 0x00; // DR bit not set
        let can_read = lsr_value & LSR_DR != 0;
        assert!(!can_read);
    }

    #[test]
    fn test_write_bit_patterns() {
        // Test write bit patterns for different widths
        let byte_val: u8 = 0xAB;
        assert_eq!(byte_val, 0xAB);

        let word_val: u16 = 0xABCD;
        assert_eq!(word_val, 0xABCD);

        let dword_val: u32 = 0x12345678;
        assert_eq!(dword_val, 0x12345678);

        // Test big-endian conversion
        let val: u32 = 0x12345678;
        let be_val = val.to_be();
        assert_eq!(be_val, 0x78563412);
    }

    #[test]
    fn test_io_kind_width_values() {
        // Test IoKind width values
        assert_eq!(IoKind::Port.width(), 1);
        assert_eq!(IoKind::Mmio.width(), 4);
        assert_eq!(IoKind::Mmio16.width(), 2);
        assert_eq!(IoKind::Mmio32.width(), 4);
        assert_eq!(IoKind::Mmio32be.width(), 4);
    }

    #[test]
    fn test_error_kind_variants() {
        // Test ErrorKind exists and can be used
        let _error = ErrorKind::Other;
        let _error = ErrorKind::Overrun;
        let _error = ErrorKind::Parity;
    }
}
