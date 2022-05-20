extern "C" {
    pub fn LEDGER_BLE_init();
    pub fn LEDGER_BLE_send(packet: *const u8, packet_length: u16);
    pub fn LEDGER_BLE_receive(spi_buffer: *const u8);
    pub fn LEDGER_BLE_set_recv_buffer(buffer: *mut u8, buffer_length: u16);
}

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
