use crate::{os_io_rx_evt, os_io_tx_cmd};

/// Receive the next APDU into 'buffer'
pub fn io_rx(buffer: &mut [u8], check_se_event: bool) -> i32 {
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
    unsafe {
        os_io_tx_cmd(
            apdu_type,
            buffer.as_ptr() as _,
            length as u16,
            core::ptr::null_mut(),
        )
    }
}
