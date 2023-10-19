use ledger_sdk_sys::{LEDGER_BLE_receive, LEDGER_BLE_send, LEDGER_BLE_set_recv_buffer};

pub fn receive(apdu_buffer: &mut [u8], spi_buffer: &[u8]) {
    unsafe {
        LEDGER_BLE_set_recv_buffer(apdu_buffer.as_mut_ptr(), apdu_buffer.len() as u16);
        LEDGER_BLE_receive(spi_buffer.as_ptr());
    }
}

pub fn send(buffer: &[u8]) {
    unsafe {
        LEDGER_BLE_send(buffer.as_ptr(), buffer.len() as u16);
    }
}
