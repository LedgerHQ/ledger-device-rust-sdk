use crate::{
    io_seph_is_status_sent, io_seph_recv, io_seph_send, SEPROXYHAL_TAG_GENERAL_STATUS,
    SEPROXYHAL_TAG_RAPDU, SEPROXYHAL_TAG_SCREEN_DISPLAY_STATUS,
};

/// Directly send buffer over the SPI channel to the MCU
pub fn seph_send(buffer: &[u8]) {
    unsafe { io_seph_send(buffer.as_ptr(), buffer.len() as u16) };
}

/// Receive the next APDU into 'buffer'
pub fn seph_recv(buffer: &mut [u8], flags: u32) -> u16 {
    unsafe { io_seph_recv(buffer.as_mut_ptr(), buffer.len() as u16, flags) }
}

/// Wrapper for 'io_seph_is_status_sent'
pub fn is_status_sent() -> bool {
    let status = unsafe { io_seph_is_status_sent() };
    status == 1
}

/// Inform the MCU that the previous event was processed
pub fn send_general_status() {
    // XXX: Not sure we need this line to 'avoid troubles' like
    // in the original SDK
    //   if io_seproxyhal_spi_is_status_sent() {
    //     return;
    //   }
    if !is_status_sent() {
        // The two last bytes are supposed to be
        // SEPROXYHAL_TAG_GENERAL_STATUS_LAST_COMMAND, which is 0u16
        let status = [SEPROXYHAL_TAG_GENERAL_STATUS as u8, 0, 2, 0, 0];
        seph_send(&status);
    }
}

#[repr(u8)]
pub enum SephTags {
    ScreenDisplayStatus = SEPROXYHAL_TAG_SCREEN_DISPLAY_STATUS as u8,
    GeneralStatus = SEPROXYHAL_TAG_GENERAL_STATUS as u8,
    RawAPDU = SEPROXYHAL_TAG_RAPDU as u8,
    Unknown,
}

impl From<u8> for SephTags {
    fn from(v: u8) -> SephTags {
        match v as u32 {
            SEPROXYHAL_TAG_SCREEN_DISPLAY_STATUS => SephTags::ScreenDisplayStatus,
            SEPROXYHAL_TAG_GENERAL_STATUS => SephTags::GeneralStatus,
            _ => SephTags::Unknown,
        }
    }
}
