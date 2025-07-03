#![allow(clippy::upper_case_acronyms)]

use ledger_secure_sdk_sys::*;

//#[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
//use crate::ble;

#[repr(u8)]
pub enum PacketTypes {
    PacketTypeNone = OS_IO_PACKET_TYPE_NONE as u8,
    PacketTypeSeph = OS_IO_PACKET_TYPE_SEPH as u8,
    PacketTypeSeEvent = OS_IO_PACKET_TYPE_SE_EVT as u8,

    PacketTypeRawApdu = OS_IO_PACKET_TYPE_RAW_APDU as u8,
    PacketTypeUsbHidApdu = OS_IO_PACKET_TYPE_USB_HID_APDU as u8,
    PacketTypeUsbWebusbApdu = OS_IO_PACKET_TYPE_USB_WEBUSB_APDU as u8,

    PacketTypeBleApdu = OS_IO_PACKET_TYPE_BLE_APDU as u8,
}

impl From<u8> for PacketTypes {
    fn from(v: u8) -> PacketTypes {
        match v as u8 {
            OS_IO_PACKET_TYPE_NONE => PacketTypes::PacketTypeNone,
            OS_IO_PACKET_TYPE_SEPH => PacketTypes::PacketTypeSeph,
            OS_IO_PACKET_TYPE_SE_EVT => PacketTypes::PacketTypeSeEvent,
            OS_IO_PACKET_TYPE_RAW_APDU => PacketTypes::PacketTypeRawApdu,
            OS_IO_PACKET_TYPE_USB_HID_APDU => PacketTypes::PacketTypeUsbHidApdu,
            OS_IO_PACKET_TYPE_USB_WEBUSB_APDU => PacketTypes::PacketTypeUsbWebusbApdu,
            OS_IO_PACKET_TYPE_BLE_APDU => PacketTypes::PacketTypeBleApdu,
            _ => PacketTypes::PacketTypeNone,
        }
    }
}

#[repr(u8)]
pub enum Events {
    TickerEvent = SEPROXYHAL_TAG_TICKER_EVENT as u8,
    ButtonPushEvent = SEPROXYHAL_TAG_BUTTON_PUSH_EVENT as u8,
    ScreenTouchEvent = SEPROXYHAL_TAG_FINGER_EVENT as u8,
    ItcEvent = SEPROXYHAL_TAG_ITC_EVENT as u8,
    Unknown = 0xff,
}

impl From<u8> for Events {
    fn from(v: u8) -> Events {
        match v as u32 {
            SEPROXYHAL_TAG_TICKER_EVENT => Events::TickerEvent,
            SEPROXYHAL_TAG_BUTTON_PUSH_EVENT => Events::ButtonPushEvent,
            SEPROXYHAL_TAG_FINGER_EVENT => Events::ScreenTouchEvent,
            SEPROXYHAL_TAG_ITC_EVENT => Events::ItcEvent,
            _ => Events::Unknown,
        }
    }
}

#[repr(u8)]
pub enum ItcUxEvent {
    AskBlePairing = ITC_UX_ASK_BLE_PAIRING as u8,
    BlePairingStatus = ITC_UX_BLE_PAIRING_STATUS as u8,
    Redisplay = ITC_UX_REDISPLAY as u8,
    Unknown = 0xff,
}

impl From<u8> for ItcUxEvent {
    fn from(v: u8) -> ItcUxEvent {
        match v as u8 {
            ITC_UX_ASK_BLE_PAIRING => ItcUxEvent::AskBlePairing,
            ITC_UX_BLE_PAIRING_STATUS => ItcUxEvent::BlePairingStatus,
            ITC_UX_REDISPLAY => ItcUxEvent::Redisplay,
            _ => ItcUxEvent::Unknown,
        }
    }
}

/// FFI bindings to USBD functions inlined here for clarity
/// and also because some of the generated ones are incorrectly
/// assuming mutable pointers when they are not
#[repr(C)]
#[derive(Copy, Clone)]
pub struct apdu_buffer_s {
    pub buf: *mut u8,
    pub len: u16,
}
impl Default for apdu_buffer_s {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}
pub type ApduBufferT = apdu_buffer_s;

pub fn handle_event(_apdu_buffer: &mut [u8], spi_buffer: &[u8]) {
    let _len = u16::from_be_bytes([spi_buffer[1], spi_buffer[2]]);
    match Events::from(spi_buffer[0]) {
        Events::TickerEvent => {
            #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
            unsafe {
                ux_process_ticker_event();
            }
        }
        _ => (),
    }
}
