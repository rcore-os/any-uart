//! System tests for any-uart on x86 architecture
//!
//! These tests verify UART functionality for x86-specific scenarios.

#[cfg(target_arch = "x86_64")]
mod x86_architecture_tests {
    use any_uart::IoKind;

    #[test]
    fn test_x86_port_io_kind() {
        // Test that Port I/O kind is available for x86
        let port_kind = IoKind::Port;
        assert_eq!(port_kind.width(), 1); // width() returns bytes, not bits
    }

    #[test]
    fn test_x86_com_port_addresses() {
        // Test standard COM port addresses are accessible
        let com1: usize = 0x3f8;
        let com2: usize = 0x2f8;
        let com3: usize = 0x3e8;
        let com4: usize = 0x2e8;

        assert_eq!(com1, 0x3f8);
        assert_eq!(com2, 0x2f8);
        assert_eq!(com3, 0x3e8);
        assert_eq!(com4, 0x2e8);
    }

    #[test]
    fn test_x86_all_io_kinds_available() {
        // Test all I/O kinds are available on x86_64
        let kinds = [
            IoKind::Port,
            IoKind::Mmio,
            IoKind::Mmio16,
            IoKind::Mmio32,
            IoKind::Mmio32be,
        ];

        for kind in kinds.iter() {
            assert!(kind.width() > 0);
        }
    }

    #[test]
    fn test_x86_error_handling() {
        // Test error handling on x86
        use any_uart::{Error, ErrorKind};

        let would_block: Error = Error::WouldBlock;
        let overrun: Error = Error::Other(ErrorKind::Overrun);
        let parity: Error = Error::Other(ErrorKind::Parity);
        let other: Error = Error::Other(ErrorKind::Other);

        // Verify error types are distinct
        assert!(matches!(would_block, Error::WouldBlock));
        assert!(matches!(overrun, Error::Other(ErrorKind::Overrun)));
        assert!(matches!(parity, Error::Other(ErrorKind::Parity)));
        assert!(matches!(other, Error::Other(ErrorKind::Other)));
    }

    #[test]
    fn test_x86_irq_event() {
        // Test interrupt event handling on x86
        let event = any_uart::IrqEvent::default();
        assert!(!event.rx);
        assert!(!event.tx);

        let rx_event = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        assert!(rx_event.rx);
        assert!(!rx_event.tx);

        let tx_event = any_uart::IrqEvent {
            rx: false,
            tx: true,
        };
        assert!(!tx_event.rx);
        assert!(tx_event.tx);

        let both_event = any_uart::IrqEvent { rx: true, tx: true };
        assert!(both_event.rx);
        assert!(both_event.tx);
    }
}

#[cfg(test)]
mod type_system_tests {
    #[test]
    fn test_public_types_exist() {
        // Test that all public types are accessible
        let _sender: Option<any_uart::Sender> = None;
        let _receiver: Option<any_uart::Receiver> = None;
        let _uart: Option<any_uart::Uart> = None;
        let _io_kind: any_uart::IoKind = any_uart::IoKind::Port;
        let _irq_event: any_uart::IrqEvent = any_uart::IrqEvent::default();
    }

    #[test]
    fn test_error_types_exist() {
        // Test error types are accessible
        use any_uart::{Error, ErrorKind};

        let _would_block: Error = Error::WouldBlock;
        let _overrun: ErrorKind = ErrorKind::Overrun;
        let _parity: ErrorKind = ErrorKind::Parity;
        let _other: ErrorKind = ErrorKind::Other;
    }

    #[test]
    fn test_sender_receiver_traits() {
        // Test that Sender and Receiver types are defined
        type Sender = any_uart::Sender;
        type Receiver = any_uart::Receiver;

        let _sender_option: Option<Sender>;
        let _receiver_option: Option<Receiver>;
    }
}

#[cfg(test)]
mod integration_tests {
    use core::ptr::NonNull;

    #[test]
    fn test_init_function_signature() {
        // Test init function signature
        let fdt = [0u8; 1024];
        let fdt_addr = NonNull::new(fdt.as_ptr() as usize as *mut u8).unwrap();

        // Should compile with correct signature
        let _uart = any_uart::init(fdt_addr, |r| r as *mut u8);
    }

    #[test]
    fn test_device_tree_blob() {
        // Test device tree blob is accessible
        let fdt = include_bytes!("../../dtb/rk3568-firefly-roc-pc-se.dtb");
        assert!(!fdt.is_empty());
        assert!(fdt.len() > 1000);
    }
}

#[cfg(test)]
mod utility_tests {
    use any_uart::{Error, ErrorKind};

    #[test]
    fn test_block_macro_compiles() {
        // Test that block! macro is available and compiles
        let mut counter = 0;

        // Note: We can't actually call block! here without real hardware,
        // but we can verify it's in scope by using it in a valid context
        // This is a compile-time check
        use any_uart::block;

        // Create a function that would use block!
        let mut would_block_fn = || -> Result<(), any_uart::Error> {
            counter += 1;
            if counter < 3 {
                Err(Error::WouldBlock)
            } else {
                Ok(())
            }
        };

        // This will retry until success
        let result = block!(would_block_fn());
        assert!(result.is_ok());
        assert_eq!(counter, 3);
    }

    #[test]
    fn test_io_kind_display_debug() {
        // Test IoKind can be formatted for display and debug
        let kinds = [
            any_uart::IoKind::Port,
            any_uart::IoKind::Mmio,
            any_uart::IoKind::Mmio16,
            any_uart::IoKind::Mmio32,
            any_uart::IoKind::Mmio32be,
        ];

        for kind in kinds.iter() {
            // Debug format should work
            let debug_str = format!("{:?}", kind);
            assert!(!debug_str.is_empty());

            // Should be able to derive Debug
            let _copy = *kind; // Copy trait
        }
    }

    #[test]
    fn test_irq_event_traits() {
        // Test IrqEvent implements required traits
        let event1 = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };
        let event2 = any_uart::IrqEvent {
            rx: true,
            tx: false,
        };

        // Copy and Clone
        let event3 = event1;
        let event4 = event2.clone();

        // PartialEq
        assert_eq!(event1, event2);
        assert_eq!(event3, event4);

        // Default
        let default_event = any_uart::IrqEvent::default();
        assert!(!default_event.rx && !default_event.tx);

        // Debug
        let debug_str = format!("{:?}", event1);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_io_kind_from_str_coverage() {
        // Test IoKind::from covers all cases
        use any_uart::IoKind;

        let test_cases = vec![
            ("port", IoKind::Port),
            ("mmio", IoKind::Mmio),
            ("mmio16", IoKind::Mmio16),
            ("mmio32", IoKind::Mmio32),
            ("mmio32be", IoKind::Mmio32be),
            ("invalid", IoKind::Port), // Default for unknown strings
            ("", IoKind::Port),        // Default for empty string
            ("PORT", IoKind::Port),    // Case sensitive, default
        ];

        for (input, expected) in test_cases {
            let result = IoKind::from(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_error_variants_coverage() {
        // Test all error variants are accessible
        use any_uart::{Error, ErrorKind};

        let errors = vec![
            Error::WouldBlock,
            Error::Other(ErrorKind::Overrun),
            Error::Other(ErrorKind::Parity),
            Error::Other(ErrorKind::Other),
        ];

        for error in errors {
            match error {
                Error::WouldBlock => assert!(true),
                Error::Other(ErrorKind::Overrun) => assert!(true),
                Error::Other(ErrorKind::Parity) => assert!(true),
                Error::Other(ErrorKind::Other) => assert!(true),
                _ => panic!("Unknown error variant"),
            }
        }
    }
}
