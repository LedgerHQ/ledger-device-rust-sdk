#![allow(clippy::upper_case_acronyms)]

use ledger_secure_sdk_sys::*;

//#[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
//use crate::ble;

#[repr(u8)]
pub enum PacketTypes {
    PacketTypeNone = OS_IO_PACKET_TYPE_NONE,
    PacketTypeSeph = OS_IO_PACKET_TYPE_SEPH,
    PacketTypeSeEvent = OS_IO_PACKET_TYPE_SE_EVT,

    PacketTypeRawApdu = OS_IO_PACKET_TYPE_RAW_APDU,
    PacketTypeUsbHidApdu = OS_IO_PACKET_TYPE_USB_HID_APDU,
    PacketTypeUsbWebusbApdu = OS_IO_PACKET_TYPE_USB_WEBUSB_APDU,

    PacketTypeBleApdu = OS_IO_PACKET_TYPE_BLE_APDU,
}

impl From<u8> for PacketTypes {
    fn from(v: u8) -> PacketTypes {
        match v {
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
    AskBlePairing = ITC_UX_ASK_BLE_PAIRING,
    BlePairingStatus = ITC_UX_BLE_PAIRING_STATUS,
    Redisplay = ITC_UX_REDISPLAY,
    Unknown = 0xff,
}

impl From<u8> for ItcUxEvent {
    fn from(v: u8) -> ItcUxEvent {
        match v {
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
    if let Events::TickerEvent = Events::from(spi_buffer[0]) {
        #[cfg(any(
            target_os = "apex_p",
            target_os = "stax",
            target_os = "flex",
            feature = "nano_nbgl"
        ))]
        unsafe {
            ux_process_ticker_event();
        }
    }
}

mod canary {
    // This module provides stack overflow protection by initializing and checking
    // a canary value to detect if the stack has grown too much and is overlapping
    // with the .bss section. The canary is checked on every APDU I/O operation and
    // will panic if corruption is detected.
    // This might later be removed if such protection is provided in the C SDK.

    unsafe extern "C" {
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
