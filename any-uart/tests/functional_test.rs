//! Functional tests for any-uart crate
//!
//! These tests verify the basic functionality and API contracts
//! of the UART library.

#[cfg(test)]
mod functional_tests {
    use any_uart::{Error, ErrorKind};

    #[test]
    fn test_error_types() {
        // Test that error types are accessible
        let _error: Error = Error::WouldBlock;
        let _error: Error = Error::Other(ErrorKind::Overrun);
        let _error: Error = Error::Other(ErrorKind::Parity);
        let _error: Error = Error::Other(ErrorKind::Other);
    }

    #[test]
    fn test_io_kind_variants() {
        // Test IoKind enum variants
        use any_uart::IoKind;

        let _port = IoKind::Port;
        let _mmio = IoKind::Mmio;
        let _mmio16 = IoKind::Mmio16;
        let _mmio32 = IoKind::Mmio32;
        let _mmio32be = IoKind::Mmio32be;
    }

    #[test]
    fn test_io_kind_from_str() {
        // Test IoKind parsing from strings
        use any_uart::IoKind;

        let port = IoKind::from("port");
        assert_eq!(port, IoKind::Port);

        let mmio = IoKind::from("mmio");
        assert_eq!(mmio, IoKind::Mmio);

        let mmio32 = IoKind::from("mmio32");
        assert_eq!(mmio32, IoKind::Mmio32);

        let mmio16 = IoKind::from("mmio16");
        assert_eq!(mmio16, IoKind::Mmio16);

        let mmio32be = IoKind::from("mmio32be");
        assert_eq!(mmio32be, IoKind::Mmio32be);

        let invalid = IoKind::from("invalid");
        assert_eq!(invalid, IoKind::Port); // Default for unknown strings
    }

    #[test]
    fn test_io_kind_width() {
        // Test IoKind width calculation (returns bytes, not bits)
        use any_uart::IoKind;

        assert_eq!(IoKind::Port.width(), 1);
        assert_eq!(IoKind::Mmio.width(), 4);
        assert_eq!(IoKind::Mmio16.width(), 2);
        assert_eq!(IoKind::Mmio32.width(), 4);
        assert_eq!(IoKind::Mmio32be.width(), 4);
    }

    #[test]
    fn test_irq_event_default() {
        // Test IrqEvent default values
        let event = any_uart::IrqEvent::default();
        assert!(!event.rx);
        assert!(!event.tx);
    }

    #[test]
    fn test_irq_event_creation() {
        // Test IrqEvent creation with specific values
        let event = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        assert!(event.rx);
        assert!(!event.tx);

        let event2 = any_uart::IrqEvent {
            rx: false,
            tx: true,
        };
        assert!(!event2.rx);
        assert!(event2.tx);
    }

    #[test]
    fn test_irq_event_copy() {
        // Test IrqEvent Copy and Clone
        let event = any_uart::IrqEvent { rx: true, tx: true };
        let event2 = event;
        let event3 = event;

        assert_eq!(event.rx, event2.rx);
        assert_eq!(event.tx, event3.tx);
    }
}

#[cfg(test)]
mod api_tests {
    use any_uart::IoKind;

    #[test]
    fn test_io_kind_partial_eq() {
        // Test IoKind equality
        assert_eq!(IoKind::Port, IoKind::Port);
        assert_eq!(IoKind::Mmio32, IoKind::Mmio32);
        assert_ne!(IoKind::Port, IoKind::Mmio);
    }

    #[test]
    fn test_irq_event_partial_eq() {
        // Test IrqEvent equality
        let event1 = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        let event2 = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        let event3 = any_uart::IrqEvent {
            rx: false,
            tx: true,
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_io_kind_debug() {
        // Test IoKind Debug implementation
        let port = IoKind::Port;
        let debug_str = format!("{:?}", port);
        assert!(debug_str.contains("Port"));
    }

    #[test]
    fn test_irq_event_debug() {
        // Test IrqEvent Debug implementation
        let event = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("rx") || debug_str.contains("tx"));
    }
}

#[cfg(test)]
mod type_validation_tests {
    #[test]
    fn test_sender_type() {
        // Test Sender type compiles
        type Sender = any_uart::Sender;
        let _sender: Option<Sender> = None;
    }

    #[test]
    fn test_receiver_type() {
        // Test Receiver type compiles
        type Receiver = any_uart::Receiver;
        let _receiver: Option<Receiver> = None;
    }

    #[test]
    fn test_uart_type() {
        // Test Uart type compiles
        type Uart = any_uart::Uart;
        let _uart: Option<Uart> = None;
    }
}

#[cfg(all(test, feature = "fdt"))]
mod fdt_integration_tests {
    use core::ptr::NonNull;

    #[test]
    fn test_device_tree_init_signature() {
        // Test that init function has the correct signature
        let fdt = [0u8; 1024];
        let fdt_addr = NonNull::new(fdt.as_ptr() as usize as *mut u8).unwrap();

        // This should compile if the signature is correct
        let _uart = any_uart::init(fdt_addr, |r| r as *mut u8);
    }

    #[test]
    fn test_device_tree_file_exists() {
        // Verify device tree file is accessible for integration tests
        let fdt = include_bytes!("../../dtb/rk3568-firefly-roc-pc-se.dtb");
        assert!(!fdt.is_empty());
        assert!(fdt.len() > 1000); // Should have some content
    }
}
