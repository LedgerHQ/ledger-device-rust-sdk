#![allow(clippy::upper_case_acronyms)]

use ledger_sdk_sys::*;

#[cfg(target_os = "nanox")]
use crate::ble;

#[repr(u8)]
pub enum Events {
    USBXFEREvent = SEPROXYHAL_TAG_USB_EP_XFER_EVENT as u8,
    USBEvent = SEPROXYHAL_TAG_USB_EVENT as u8,
    USBEventReset = SEPROXYHAL_TAG_USB_EVENT_RESET as u8,
    USBEventSOF = SEPROXYHAL_TAG_USB_EVENT_SOF as u8,
    USBEventSuspend = SEPROXYHAL_TAG_USB_EVENT_SUSPENDED as u8,
    USBEventResume = SEPROXYHAL_TAG_USB_EVENT_RESUMED as u8,
    CAPDUEvent = SEPROXYHAL_TAG_CAPDU_EVENT as u8,
    TickerEvent = SEPROXYHAL_TAG_TICKER_EVENT as u8,
    ButtonPush = SEPROXYHAL_TAG_BUTTON_PUSH_EVENT as u8,
    DisplayProcessed = SEPROXYHAL_TAG_DISPLAY_PROCESSED_EVENT as u8,
    BleReceive = SEPROXYHAL_TAG_BLE_RECV_EVENT as u8,
    Unknown = 0xff,
}
#[repr(u8)]
pub enum UsbEp {
    USBEpXFERSetup = SEPROXYHAL_TAG_USB_EP_XFER_SETUP as u8,
    USBEpXFERIn = SEPROXYHAL_TAG_USB_EP_XFER_IN as u8,
    USBEpXFEROut = SEPROXYHAL_TAG_USB_EP_XFER_OUT as u8,
    USBEpPrepare = SEPROXYHAL_TAG_USB_EP_PREPARE as u8,
    USBEpPrepareDirIn = SEPROXYHAL_TAG_USB_EP_PREPARE_DIR_IN as u8,
    Unknown,
}

impl From<u8> for Events {
    fn from(v: u8) -> Events {
        match v as u32 {
            SEPROXYHAL_TAG_USB_EP_XFER_EVENT => Events::USBXFEREvent,
            SEPROXYHAL_TAG_USB_EVENT => Events::USBEvent,
            SEPROXYHAL_TAG_USB_EVENT_RESET => Events::USBEventReset,
            SEPROXYHAL_TAG_USB_EVENT_SOF => Events::USBEventSOF,
            SEPROXYHAL_TAG_USB_EVENT_SUSPENDED => Events::USBEventSuspend,
            SEPROXYHAL_TAG_USB_EVENT_RESUMED => Events::USBEventResume,
            SEPROXYHAL_TAG_CAPDU_EVENT => Events::CAPDUEvent,
            SEPROXYHAL_TAG_TICKER_EVENT => Events::TickerEvent,
            SEPROXYHAL_TAG_BUTTON_PUSH_EVENT => Events::ButtonPush,
            SEPROXYHAL_TAG_DISPLAY_PROCESSED_EVENT => Events::DisplayProcessed,
            SEPROXYHAL_TAG_BLE_RECV_EVENT => Events::BleReceive,
            _ => Events::Unknown,
        }
    }
}

impl From<u8> for UsbEp {
    fn from(v: u8) -> UsbEp {
        match v as u32 {
            SEPROXYHAL_TAG_USB_EP_XFER_SETUP => UsbEp::USBEpXFERSetup,
            SEPROXYHAL_TAG_USB_EP_XFER_IN => UsbEp::USBEpXFERIn,
            SEPROXYHAL_TAG_USB_EP_XFER_OUT => UsbEp::USBEpXFEROut,
            SEPROXYHAL_TAG_USB_EP_PREPARE => UsbEp::USBEpPrepare,
            SEPROXYHAL_TAG_USB_EP_PREPARE_DIR_IN => UsbEp::USBEpPrepareDirIn,
            _ => UsbEp::Unknown,
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
extern "C" {
    pub static mut USBD_Device: USBD_HandleTypeDef;
    pub fn USBD_LL_SetupStage(
        pdev: *mut USBD_HandleTypeDef,
        psetup: *const u8,
    ) -> USBD_StatusTypeDef;
    pub fn USBD_LL_DataOutStage(
        pdev: *mut USBD_HandleTypeDef,
        epnum: u8,
        pdata: *const u8,
        arg1: *mut ApduBufferT,
    ) -> USBD_StatusTypeDef;
    pub fn USBD_LL_DataInStage(
        pdev: *mut USBD_HandleTypeDef,
        epnum: u8,
        pdata: *const u8,
    ) -> USBD_StatusTypeDef;
    pub fn USBD_LL_Reset(pdev: *mut USBD_HandleTypeDef) -> USBD_StatusTypeDef;
    pub fn USBD_LL_SetSpeed(
        pdev: *mut USBD_HandleTypeDef,
        speed: USBD_SpeedTypeDef,
    ) -> USBD_StatusTypeDef;
    pub fn USBD_LL_Suspend(pdev: *mut USBD_HandleTypeDef) -> USBD_StatusTypeDef;
    pub fn USBD_LL_Resume(pdev: *mut USBD_HandleTypeDef) -> USBD_StatusTypeDef;
    pub fn USBD_LL_SOF(pdev: *mut USBD_HandleTypeDef) -> USBD_StatusTypeDef;
}

/// Below is a straightforward translation of the corresponding functions
/// in the C SDK, they could be improved
pub fn handle_usb_event(event: u8) {
    match Events::from(event) {
        Events::USBEventReset => {
            unsafe {
                USBD_LL_SetSpeed(&mut USBD_Device, 1 /*USBD_SPEED_FULL*/);
                USBD_LL_Reset(&mut USBD_Device);

                if G_io_app.apdu_media != IO_APDU_MEDIA_NONE {
                    return;
                }

                G_io_app.usb_ep_xfer_len = core::mem::zeroed();
                G_io_app.usb_ep_timeouts = core::mem::zeroed();
            }
        }
        Events::USBEventSOF => unsafe {
            USBD_LL_SOF(&mut USBD_Device);
        },
        Events::USBEventSuspend => unsafe {
            USBD_LL_Suspend(&mut USBD_Device);
        },
        Events::USBEventResume => unsafe {
            USBD_LL_Resume(&mut USBD_Device);
        },
        _ => (),
    }
}

pub fn handle_usb_ep_xfer_event(apdu_buffer: &mut [u8], buffer: &[u8]) {
    let endpoint = buffer[3] & 0x7f;
    match UsbEp::from(buffer[4]) {
        UsbEp::USBEpXFERSetup => unsafe {
            USBD_LL_SetupStage(&mut USBD_Device, &buffer[6]);
        },
        UsbEp::USBEpXFERIn => {
            if (endpoint as u32) < IO_USB_MAX_ENDPOINTS {
                unsafe {
                    G_io_app.usb_ep_timeouts[endpoint as usize].timeout = 0;
                    USBD_LL_DataInStage(&mut USBD_Device, endpoint, &buffer[6]);
                }
            }
        }
        UsbEp::USBEpXFEROut => {
            if (endpoint as u32) < IO_USB_MAX_ENDPOINTS {
                unsafe {
                    G_io_app.usb_ep_xfer_len[endpoint as usize] = buffer[5];
                    let mut apdu_buf = ApduBufferT {
                        buf: apdu_buffer.as_mut_ptr(),
                        len: 260,
                    };
                    USBD_LL_DataOutStage(&mut USBD_Device, endpoint, &buffer[6], &mut apdu_buf);
                }
            }
        }
        _ => (),
    }
}

pub fn handle_capdu_event(apdu_buffer: &mut [u8], buffer: &[u8]) {
    let io_app = unsafe { &mut G_io_app };
    if io_app.apdu_state == APDU_IDLE {
        let max = (apdu_buffer.len() - 3).min(buffer.len() - 3);
        let size = u16::from_be_bytes([buffer[1], buffer[2]]) as usize;

        io_app.apdu_media = IO_APDU_MEDIA_RAW;
        io_app.apdu_state = APDU_RAW;

        let len = size.min(max);

        io_app.apdu_length = len as u16;

        apdu_buffer[..len].copy_from_slice(&buffer[3..len + 3]);
    }
}

pub fn handle_event(apdu_buffer: &mut [u8], spi_buffer: &[u8]) {
    let len = u16::from_be_bytes([spi_buffer[1], spi_buffer[2]]);
    match Events::from(spi_buffer[0]) {
        Events::USBEvent => {
            if len == 1 {
                handle_usb_event(spi_buffer[3]);
            }
        }
        Events::USBXFEREvent => {
            if len >= 3 {
                handle_usb_ep_xfer_event(apdu_buffer, spi_buffer);
            }
        }
        #[cfg(target_os = "nanox")]
        Events::BleReceive => ble::receive(apdu_buffer, spi_buffer),
        Events::CAPDUEvent => handle_capdu_event(apdu_buffer, spi_buffer),
        Events::TickerEvent => { /* unsafe{ G_io_app.ms += 100; } */ }
        _ => (),
    }
}
