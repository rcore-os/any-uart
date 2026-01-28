//! ARM PL011 UART controller driver.
//!
//! This module implements driver support for the ARM PrimeCell PL011 UART controller.

use bitflags::bitflags;

use crate::{Console, ErrorKind, IrqEvent, UartData};

bitflags! {
    /// PL011 UART interrupt flags.
    ///
    /// Defines various interrupt types for the PL011 UART controller.
    struct Interrupts: u32 {
        /// Overrun error interrupt.
        const OEI = 1 << 10;
        /// Break error interrupt.
        const BEI = 1 << 9;
        /// Parity error interrupt.
        const PEI = 1 << 8;
        /// Framing error interrupt.
        const FEI = 1 << 7;
        /// Receive timeout interrupt.
        const RTI = 1 << 6;
        /// Transmit interrupt.
        const TXI = 1 << 5;
        /// Receive interrupt.
        const RXI = 1 << 4;
        /// nUARTDSR modem interrupt.
        const DSRMI = 1 << 3;
        /// nUARTDCD modem interrupt.
        const DCDMI = 1 << 2;
        /// nUARTCTS modem interrupt.
        const CTSMI = 1 << 1;
        /// nUARTRI modem interrupt.
        const RIMI = 1 << 0;
    }
}

const UARTCR: usize = 0x030 / 4;
const IMSC: usize = 0x038 / 4;
const RIS: usize = 0x03C / 4;
const MIS: usize = 0x040 / 4;
const ICR: usize = 0x044 / 4;

/// ARM PL011 UART controller driver.
///
/// Implements a driver for the ARM PrimeCell PL011 UART controller.
/// This is a commonly used serial port controller on ARM platforms, widely used in various ARM SoCs.
///
/// # Device Tree Identification
///
/// This controller is identified by the `"arm,pl011"` or `"arm,primecell"` compatible strings.
///
/// # Supported Features
///
/// - 32-bit memory mapped I/O
/// - Transmit and receive interrupts
/// - Error detection (overrun, frame error, parity error, etc.)
pub struct Pl011 {}

impl Console for Pl011 {
    /// Writes a byte to the UART.
    ///
    /// Writes to DR (Data Register, offset 0x00).
    fn put(uart: UartData, byte: u8) -> Result<(), ErrorKind> {
        unsafe {
            let put = uart.reg_u8(0);
            put.write_volatile(byte);
            Ok(())
        }
    }

    /// Reads a byte from the UART.
    ///
    /// Reads from DR (Data Register, offset 0x00).
    /// If an error is detected (high bits are non-zero), it clears the error status and returns an error.
    fn get(uart: UartData) -> Result<u8, ErrorKind> {
        unsafe {
            let data = uart.reg::<u32>(0).read_volatile();

            if data & 0xFFFFFF00 != 0 {
                // Clear the error
                uart.reg::<u32>(1).write_volatile(0xFFFFFFFF);
                return Err(ErrorKind::Other);
            }

            Ok(data as _)
        }
    }

    /// Sets interrupt enable status.
    ///
    /// Configured via IMSC (Interrupt Mask Set/Clear Register, offset 0x38).
    /// When enabled, both receive and transmit interrupts are enabled.
    fn set_irq_enable(uart: UartData, enable: bool) {
        unsafe {
            let imsc = uart.reg::<u32>(IMSC);
            if enable {
                imsc.write_volatile((Interrupts::RXI | Interrupts::TXI).bits());
            } else {
                imsc.write_volatile(0);
            }
        }
    }

    /// Gets interrupt enable status.
    fn get_irq_enable(uart: UartData) -> bool {
        unsafe {
            let imsc = uart.reg::<u32>(IMSC);
            imsc.read_volatile() != 0
        }
    }

    /// Clears the specified interrupt event.
    ///
    /// Clears interrupt flags via ICR (Interrupt Clear Register, offset 0x44).
    fn clean_irq_event(uart: UartData, event: IrqEvent) {
        let mut irqs = Interrupts::empty();
        if event.rx {
            irqs |= Interrupts::RXI
        }

        if event.tx {
            irqs |= Interrupts::TXI
        }

        unsafe {
            let icr = uart.reg::<u32>(ICR);
            icr.write_volatile(irqs.bits());
        }
    }

    /// Checks if data can be written.
    ///
    /// Checks the TXFF flag of FR (Flag Register, offset 0x18).
    fn can_put(uart: UartData) -> bool {
        const TXFF: u8 = 1 << 5;
        unsafe { uart.reg_u8(0x18).read_volatile() & TXFF == 0 }
    }

    /// Checks if data is available to read.
    ///
    /// Checks the RXFE flag of FR (Flag Register, offset 0x18).
    fn can_get(uart: UartData) -> bool {
        const RXFE: u8 = 0x10;
        unsafe { uart.reg_u8(0x18).read_volatile() & RXFE == 0 }
    }

    /// Gets interrupt event.
    ///
    /// Gets interrupt status by reading RIS (Raw Interrupt Status Register) and MIS (Masked Interrupt Status Register).
    fn get_irq_event(uart: UartData) -> IrqEvent {
        let mut event = IrqEvent::default();

        unsafe {
            let ris = uart.reg::<u32>(RIS).read_volatile();
            let mis = uart.reg::<u32>(MIS).read_volatile();

            let sts = Interrupts::from_bits_retain(ris & mis);

            if sts.contains(Interrupts::RXI) {
                event.rx = true;
            }

            if sts.contains(Interrupts::TXI) {
                event.tx = true;
            }
        }
        event
    }

    /// Initializes the UART controller.
    ///
    /// Enables UART and transmitter (TXE bit 8, UARTEN bit 0).
    fn open(uart: UartData) {
        unsafe {
            // Enable UART, set TXE (bit 8) and UARTEN (bit 0)
            let cr = uart.reg::<u32>(UARTCR);
            let current_value = cr.read_volatile();
            // TXE = bit 8, UARTEN = bit 0
            cr.write_volatile(current_value | (1 << 8) | (1 << 0));
        }
    }
}
