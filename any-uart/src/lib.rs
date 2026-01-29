//! # any-uart
//!
//! Universal Asynchronous Receiver-Transmitter (UART) driver library.
//!
//! This library provides a unified UART interface abstraction for bare-metal or OS kernel environments,
//! supporting multiple common UART controllers:
//!
//! - [NS16550](https://en.wikipedia.org/wiki/16550_UART) and compatible controllers (e.g., 8250, 16550A)
//! - [PL011](https://developer.arm.com/documentation/ddi0183/g/) ARM PrimeCell UART
//! - [Aux Mini](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html) BCM2835 Auxiliary UART
//!
//! ## Core Features
//!
//! - Automatic UART detection and initialization via device tree (FDT)
//! - Synchronous blocking send and receive interfaces
//! - Interrupt-driven operation (optional)
//! - Unified abstraction supporting multiple I/O modes (port I/O, memory-mapped I/O)
//!
//! # Examples
//!
//! ### Initialize via Device Tree
//!
//! Use the `init()` function to automatically detect and initialize UART from device tree.
//! Use `Sender::write_str_blocking()` method for sending data.
//!
//! ### Manual UART Instance Creation
//!
//! For x86 platforms, you can use `new_port_8250()` to create a port I/O-based UART instance.
//!
//! ### Create via Device Tree Node
//!
//! Use `new_by_fdt_node()` to create a UART instance from a specified device tree node.
//!
//! ## Interrupt Mode
//!
//! Interrupt-driven UART operation requires using `set_irq_enable` and `get_irq_event` methods.

#![cfg_attr(not(test), no_std)]

use core::{
    ptr::NonNull,
    sync::atomic::{Ordering, fence},
};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod api;

pub use core::fmt::Write;
pub use embedded_hal_nb::nb::block;
pub use embedded_hal_nb::serial::ErrorKind;

use aux_mini::AuxMini;
pub use fdt_parser::Node;
use fdt_parser::{Chosen, Fdt};
use ns16550::Ns16550;
use pl011::Pl011;

mod aux_mini;
mod ns16550;
mod pl011;

/// UART error type.
///
/// This is a non-blocking error type used to represent error conditions that may occur during UART operations.
pub type Error = embedded_hal_nb::nb::Error<ErrorKind>;

/// Physical to virtual address conversion function type.
///
/// This function is used to convert physical addresses from device tree to accessible virtual addresses.
pub type FnPhysToVirt = fn(usize) -> *mut u8;

/// UART driver instance.
///
/// This is the main type of the library, encapsulating hardware abstraction of a specific UART controller.
/// Each instance contains a transmitter and receiver, which can independently perform data transmission and reception.
///
/// # Fields
///
/// - `tx`: Optional transmitter for sending data
/// - `rx`: Optional receiver for receiving data
pub struct Uart {
    data: UartData,
    pub tx: Option<Sender>,
    pub rx: Option<Receiver>,
    op: UartOp,
}

impl Uart {
    fn _new<C: Console>(data: UartData) -> Self {
        let op = C::to_op();
        C::open(data);

        Self {
            data,
            tx: Some(Sender { uart: data, op }),
            rx: Some(Receiver { uart: data, op }),
            op,
        }
    }

    /// Creates a new 8250/16550 port UART instance.
    ///
    /// This method creates an NS16550-compatible UART controller instance based on port I/O.
    /// Suitable for classic COM ports on x86 platforms.
    ///
    /// # Arguments
    ///
    /// * `base` - Port base address, e.g., 0x3f8 (COM1), 0x2f8 (COM2), etc.
    ///

    pub fn new_port_8250(base: usize) -> Self {
        let data = UartData::new(base as _, IoKind::Port, |p| p as _);
        Self::_new::<Ns16550>(data)
    }

    /// Adjusts the memory-mapped I/O base address offset.
    ///
    /// This method can dynamically adjust the UART's memory-mapped base address,
    /// used for scenarios requiring address remapping.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset to add to the current base address
    ///
    /// # Note
    ///
    /// This operation affects the base address used by both the transmitter and receiver.
    pub fn mmio_base_add(&mut self, offset: usize) {
        self.data.base += offset;
    }

    /// Creates a UART instance from a device tree node.
    ///
    /// This method automatically identifies the UART type by parsing the device tree node's
    /// compatible property and creates the corresponding instance.
    ///
    /// # Supported Device Tree Compatible Strings
    ///
    /// - `"brcm,bcm2835-aux-uart"` - BCM2835 Auxiliary UART
    /// - `"arm,pl011"` or `"arm,primecell"` - ARM PL011 UART
    /// - `"snps,dw-apb-uart"` - Synopsys DesignWare APB UART
    ///
    /// # Arguments
    ///
    /// * `node` - Device tree node reference
    /// * `f` - Physical to virtual address conversion function
    ///
    /// # Returns
    ///
    /// Returns `Some(Uart)` if successfully identified and created, otherwise `None`.
    pub fn new_by_fdt_node(node: &Node<'_>, f: FnPhysToVirt) -> Option<Self> {
        let reg = node.reg()?.next()?;

        let io_kind = IoKind::Mmio32;

        // TODO: support io kind detect

        let uart = UartData::new(reg.address, io_kind, f);

        for c in node.compatibles() {
            macro_rules! of_uart {
                ($name:ty, $compatible:expr) => {
                    for want in $compatible {
                        if c.contains(want) {
                            return Some(Uart::_new::<$name>(uart));
                        }
                    }
                };
            }

            of_uart!(AuxMini, ["brcm,bcm2835-aux-uart"]);
            of_uart!(Pl011, ["arm,pl011", "arm,primecell"]);
            of_uart!(Ns16550, ["snps,dw-apb-uart"]);
        }
        None
    }

    /// Enables or disables UART interrupts.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable interrupts, `false` to disable interrupts
    ///

    pub fn set_irq_enable(&mut self, enable: bool) {
        (self.op.set_irq_enable)(self.data, enable);
    }

    /// Checks if UART interrupts are enabled.
    ///
    /// # Returns
    ///
    /// Returns `true` if interrupts are enabled, otherwise `false`.
    pub fn get_irq_enable(&mut self) -> bool {
        (self.op.get_irq_enable)(self.data)
    }

    /// Clears the specified interrupt event.
    ///
    /// This method is used to clear processed interrupt flags in the interrupt handler.
    ///
    /// # Arguments
    ///
    /// * `event` - Interrupt event to clear
    pub fn clean_irq_event(&mut self, event: IrqEvent) {
        (self.op.clean_irq_event)(self.data, event);
    }

    /// Gets the current pending interrupt event.
    ///
    /// # Returns
    ///
    /// Returns an `IrqEvent` structure containing pending receive and transmit interrupt flags.
    ///

    pub fn get_irq_event(&mut self) -> IrqEvent {
        (self.op.get_irq_event)(self.data)
    }
}

/// UART interrupt event flags.
///
/// Represents interrupt event types generated by the UART controller.
///
/// # Fields
///
/// - `rx` - Receive interrupt flag, indicating data is available for reading
/// - `tx` - Transmit interrupt flag, indicating transmit buffer is available
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct IrqEvent {
    pub rx: bool,
    pub tx: bool,
}

#[derive(Clone, Copy)]
struct UartOp {
    can_put: fn(UartData) -> bool,
    put: fn(UartData, u8) -> Result<(), ErrorKind>,
    can_get: fn(UartData) -> bool,
    get: fn(UartData) -> Result<u8, ErrorKind>,
    set_irq_enable: fn(UartData, bool),
    get_irq_enable: fn(UartData) -> bool,
    get_irq_event: fn(UartData) -> IrqEvent,
    clean_irq_event: fn(UartData, IrqEvent),
}

/// UART transmitter.
///
/// Provides non-blocking serial data transmission functionality.
///

pub struct Sender {
    uart: UartData,
    op: UartOp,
}

impl Sender {
    /// Writes a byte in a non-blocking manner.
    ///
    /// Returns `Error::WouldBlock` if the transmit buffer is full.
    ///
    /// # Arguments
    ///
    /// * `word` - Byte to write
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful write, `Err(Error)` on failure.    ///

    pub fn write(&mut self, word: u8) -> Result<(), Error> {
        if !self.can_write() {
            return Err(Error::WouldBlock);
        }
        fence(Ordering::Release);
        unsafe { self.write_uncheck(word)? };
        Ok(())
    }

    /// Adjusts the memory-mapped I/O base address offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset to add to the current base address
    pub fn mmio_base_add(&mut self, offset: usize) {
        self.uart.base += offset;
    }

    /// Checks if data can be written immediately.
    ///
    /// # Returns
    ///
    /// Returns `true` if transmit buffer is available, otherwise `false`.
    ///

    pub fn can_write(&self) -> bool {
        (self.op.can_put)(self.uart)
    }

    /// Writes a byte without checking status.
    ///
    /// # Safety
    ///
    /// Must ensure UART can receive data (call `can_write()` to check) before calling this function.
    ///
    /// # Arguments
    ///
    /// * `word` - Byte to write
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful write, `Err(ErrorKind)` on failure.
    pub unsafe fn write_uncheck(&mut self, word: u8) -> Result<(), ErrorKind> {
        (self.op.put)(self.uart, word)
    }

    /// Writes a string in a blocking manner.
    ///
    /// This method waits until all bytes are written, suitable for scenarios that don't need non-blocking semantics.
    ///
    /// # Arguments
    ///
    /// * `s` - String to write
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful write.
    ///

    pub fn write_str_blocking(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            let _ = block!(self.write(c));
        }
        Ok(())
    }

    /// Gets the current memory-mapped I/O base address.
    ///
    /// # Returns
    ///
    /// Returns the current MMIO base address.
    pub fn mmio(&self) -> usize {
        self.uart.base
    }
}

/// UART receiver.
///
/// Provides non-blocking serial data reception functionality.
///

pub struct Receiver {
    uart: UartData,
    op: UartOp,
}

impl Receiver {
    /// Non-blocking read of one byte.
    ///
    /// Returns `Error::WouldBlock` if the receive buffer is empty.
    ///
    /// # Returns
    ///
    /// Returns `Ok(byte)` on success, `Err(Error)` on failure.
    ///

    pub fn read(&mut self) -> Result<u8, Error> {
        if !self.can_read() {
            return Err(Error::WouldBlock);
        }
        fence(Ordering::Release);
        let byte = unsafe { self.read_uncheck()? };
        Ok(byte)
    }

    /// Checks if there is data available to read.
    ///
    /// # Returns
    ///
    /// Returns `true` if data is available to read, `false` otherwise.
    ///

    pub fn can_read(&self) -> bool {
        (self.op.can_get)(self.uart)
    }

    /// Adjusts the memory mapped I/O base address offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset to add to current base address
    pub fn mmio_base_add(&mut self, offset: usize) {
        self.uart.base += offset;
    }

    /// Reads a byte without checking state.
    ///
    /// # Safety
    ///
    /// Must ensure UART has data available to read before calling (check with `can_read()`).
    ///
    /// # Returns
    ///
    /// Returns `Ok(byte)` on success, `Err(ErrorKind)` on failure.
    pub unsafe fn read_uncheck(&mut self) -> Result<u8, ErrorKind> {
        (self.op.get)(self.uart)
    }
}

/// UART controller trait (internal use only).
///
/// Provides a unified abstract interface for different types of UART controllers.
/// Users typically do not need to use this trait directly; instead, use the `Uart`, `Sender`, and `Receiver` types.
pub(crate) trait Console {
    fn open(uart: UartData);
    fn can_put(uart: UartData) -> bool;
    fn put(uart: UartData, c: u8) -> Result<(), ErrorKind>;
    fn can_get(uart: UartData) -> bool;
    fn get(uart: UartData) -> Result<u8, ErrorKind>;
    fn set_irq_enable(uart: UartData, enable: bool);
    fn get_irq_enable(uart: UartData) -> bool;
    fn get_irq_event(uart: UartData) -> IrqEvent;
    fn clean_irq_event(uart: UartData, event: IrqEvent);

    fn to_op() -> UartOp {
        UartOp {
            can_put: Self::can_put,
            put: Self::put,
            can_get: Self::can_get,
            get: Self::get,
            set_irq_enable: Self::set_irq_enable,
            get_irq_enable: Self::get_irq_enable,
            get_irq_event: Self::get_irq_event,
            clean_irq_event: Self::clean_irq_event,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct UartData {
    pub base: usize,
    pub io_kind: IoKind,
}

impl UartData {
    fn new(base: u64, io_kind: IoKind, f: FnPhysToVirt) -> Self {
        let mmio = f(base as _);

        Self {
            base: mmio as _,
            io_kind,
        }
    }

    pub fn reg_u8(&self, offset: usize) -> *mut u8 {
        self.reg(offset)
    }

    pub fn reg<T: Sized>(&self, offset: usize) -> *mut T {
        unsafe {
            let ptr = self.base as *mut T;
            ptr.add(offset)
        }
    }
}

/// Initialize UART from device tree.
///
/// This function searches for an appropriate UART device from the device tree and automatically initializes it.
/// It first looks for the device specified by the `chosen.stdout-path` property,
/// and if not found, attempts to parse the earlycon parameter from bootargs.
///
/// # Arguments
///
/// * `fdt_addr` - Memory address of the device tree (FDT)
/// * `fn_phys_to_virt` - Physical address to virtual address conversion function
///
/// # Returns
///
/// Returns `Some(Uart)` if an UART is found and initialized successfully, otherwise returns `None`.
pub fn init(fdt_addr: NonNull<u8>, fn_phys_to_virt: FnPhysToVirt) -> Option<Uart> {
    let fdt = Fdt::from_ptr(fdt_addr).ok()?;

    let chosen = fdt.chosen()?;

    let mut io_kind = IoKind::Mmio32;
    let node;
    let mut is_8250 = false;

    match chosen.stdout() {
        Some(n) => node = n.node,
        None => {
            let (n, io) = fdt_bootargs_find_node(&chosen, &fdt)?;
            node = n;
            io_kind = io;
            is_8250 = true;
        }
    };

    let reg = node.reg()?.next()?;

    let uart = UartData::new(reg.address, io_kind, fn_phys_to_virt);

    if is_8250 {
        return Some(Uart::_new::<Ns16550>(uart));
    } else {
        for c in node.compatibles() {
            macro_rules! of_uart {
                ($name:ty, $compatible:expr) => {
                    for want in $compatible {
                        if c.contains(want) {
                            return Some(Uart::_new::<$name>(uart));
                        }
                    }
                };
            }

            of_uart!(AuxMini, ["brcm,bcm2835-aux-uart"]);
            of_uart!(Pl011, ["arm,pl011", "arm,primecell"]);
            of_uart!(Ns16550, ["snps,dw-apb-uart"]);
        }
    }

    None
}

/// UART I/O access type.
///
/// Defines the UART register access methods, supporting port I/O and various memory mapped I/O modes.
///
/// # Variants
///
/// - `Port` - Port I/O, suitable for x86 platform IN/OUT instructions
/// - `Mmio` - Memory mapped I/O, default 32-bit width
/// - `Mmio16` - 16-bit memory mapped I/O
/// - `Mmio32` - 32-bit little-endian memory mapped I/O
/// - `Mmio32be` - 32-bit big-endian memory mapped I/O
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IoKind {
    Port,
    Mmio,
    Mmio16,
    Mmio32,
    Mmio32be,
}

impl IoKind {
    /// Returns the I/O access bit width (in bytes).
    ///
    /// # Returns
    ///
    /// Returns the byte width of this I/O type.
    ///

    pub fn width(&self) -> usize {
        match self {
            IoKind::Port => 1,
            IoKind::Mmio => 4,
            IoKind::Mmio16 => 2,
            IoKind::Mmio32 => 4,
            IoKind::Mmio32be => 4,
        }
    }
}

impl From<&str> for IoKind {
    /// Parse I/O type from string.
    ///
    /// Supported string values:
    /// - "mmio" -> `IoKind::Mmio`
    /// - "mmio16" -> `IoKind::Mmio16`
    /// - "mmio32" -> `IoKind::Mmio32`
    /// - "mmio32be" -> `IoKind::Mmio32be`
    /// - "mmio32native" -> Automatically selected based on target platform endianness
    /// - Other values -> `IoKind::Port`
    ///

    fn from(value: &str) -> Self {
        match value {
            "mmio" => IoKind::Mmio,
            "mmio16" => IoKind::Mmio16,
            "mmio32" => IoKind::Mmio32,
            "mmio32be" => IoKind::Mmio32be,
            "mmio32native" => {
                if cfg!(target_endian = "little") {
                    IoKind::Mmio32
                } else {
                    IoKind::Mmio32be
                }
            }
            _ => IoKind::Port,
        }
    }
}

fn fdt_bootargs_find_node<'a>(chosen: &Chosen<'a>, fdt: &'a Fdt<'a>) -> Option<(Node<'a>, IoKind)> {
    let bootargs = chosen.bootargs()?;

    let earlycon = bootargs
        .split_ascii_whitespace()
        .find(|&arg| arg.contains("earlycon"))?;

    let mut tmp = earlycon.split('=');
    let _ = tmp.next()?;
    let values = tmp.next()?;

    let mut values = values.split(',');

    let name = values.next()?;

    if !name.contains("uart") {
        return None;
    }

    let param2 = values.next()?;
    let addr_str;
    let io_kind = if param2.contains("0x") {
        addr_str = param2;
        IoKind::Mmio
    } else {
        addr_str = values.next()?;
        IoKind::from(param2)
    };

    let mmio = u64::from_str_radix(addr_str.trim_start_matches("0x"), 16).ok()?;

    for node in fdt.all_nodes() {
        if let Some(regs) = node.reg() {
            for reg in regs {
                if reg.address.eq(&mmio) {
                    return Some((node, io_kind));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_init() {
        let fdt = include_bytes!("../../dtb/rk3568-firefly-roc-pc-se.dtb");
        let fdt_addr = NonNull::new(fdt.as_ptr() as usize as _).unwrap();
        let _ = init(fdt_addr, |r| r as _).unwrap();
    }

    #[test]
    fn test_io_kind_width() {
        assert_eq!(IoKind::Port.width(), 1);
        assert_eq!(IoKind::Mmio.width(), 4);
        assert_eq!(IoKind::Mmio16.width(), 2);
        assert_eq!(IoKind::Mmio32.width(), 4);
        assert_eq!(IoKind::Mmio32be.width(), 4);
    }

    #[test]
    fn test_io_kind_from_str() {
        assert_eq!(IoKind::from("mmio"), IoKind::Mmio);
        assert_eq!(IoKind::from("mmio16"), IoKind::Mmio16);
        assert_eq!(IoKind::from("mmio32"), IoKind::Mmio32);
        assert_eq!(IoKind::from("mmio32be"), IoKind::Mmio32be);
        assert_eq!(IoKind::from("mmio32native"), IoKind::Mmio32); // Assuming little-endian
        assert_eq!(IoKind::from("invalid"), IoKind::Port);
        assert_eq!(IoKind::from("port"), IoKind::Port);
    }

    #[test]
    fn test_irq_event_default() {
        let event = IrqEvent::default();
        assert!(!event.rx);
        assert!(!event.tx);
    }

    #[test]
    fn test_irq_event_creation() {
        let event = IrqEvent {
            rx: true,
            tx: false,
        };
        assert!(event.rx);
        assert!(!event.tx);
    }

    #[test]
    fn test_irq_event_copy() {
        let event1 = IrqEvent { rx: true, tx: true };
        let event2 = event1;
        assert_eq!(event1.rx, event2.rx);
        assert_eq!(event1.tx, event2.tx);
    }
}
