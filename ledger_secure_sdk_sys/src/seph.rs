use crate::{os_io_rx_evt, os_io_tx_cmd};

mod canary {
    // This module provides stack overflow protection by initializing and checking
    // a canary value to detect if the stack has grown too much and is overlapping
    // with the .bss section. The canary is checked on every APDU I/O operation and
    // will panic if corruption is detected.
    // This might later be removed if such protection is provided in the C SDK.

    extern "C" {
        /// Stack canary symbol provided by the linker script
        static mut app_stack_canary: u32;
    }

    const APP_STACK_CANARY_MAGIC: u32 = 0xDEAD0031;
    static mut CANARY_INITIALIZED: bool = false;

    /// Initialize the stack canary with the magic value
    fn init_canary() {
        unsafe {
            core::ptr::write_volatile(&raw mut app_stack_canary, APP_STACK_CANARY_MAGIC);
            CANARY_INITIALIZED = true;
        }
    }

    /// Ensure canary is initialized and check if it's still intact
    #[inline]
    pub(super) fn init_and_check() {
        unsafe {
            if !CANARY_INITIALIZED {
                init_canary();
            }

            let canary_value = core::ptr::read_volatile(&raw const app_stack_canary);
            if canary_value != APP_STACK_CANARY_MAGIC {
                panic!("Stack canary corruption detected!");
            }
        }
    }
}

/// Receive the next APDU into 'buffer'
pub fn io_rx(buffer: &mut [u8], check_se_event: bool) -> i32 {
    canary::init_and_check();
    unsafe {
        os_io_rx_evt(
            buffer.as_ptr() as _,
            buffer.len() as u16,
            core::ptr::null_mut(),
            check_se_event,
        )
    }
}

pub fn io_tx(apdu_type: u8, buffer: &[u8], length: usize) -> i32 {
    canary::init_and_check();
    unsafe {
        os_io_tx_cmd(
            apdu_type,
            buffer.as_ptr() as _,
            length as u16,
            core::ptr::null_mut(),
        )
    }
}
