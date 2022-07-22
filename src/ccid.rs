extern "C" {
    pub static mut G_io_apdu_buffer: [u8; 260];
    pub fn io_usb_ccid_reply_bare(length: u16); 
}

pub fn send(buf: &[u8]) {
    unsafe {
        G_io_apdu_buffer[..buf.len()].copy_from_slice(buf);
        io_usb_ccid_reply_bare(buf.len() as u16);
    }
}
