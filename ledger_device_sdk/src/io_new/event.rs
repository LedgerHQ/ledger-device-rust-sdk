use super::{ApduError, ApduHeader, Comm};
use crate::seph::{self, PacketTypes};

#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use crate::buttons::ButtonEvent;
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use ledger_secure_sdk_sys::buttons::{get_button_event, ButtonsState};

#[cfg(any(
    target_os = "nanox",
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p"
))]
use crate::seph::ItcUxEvent;

use ledger_secure_sdk_sys::*;

// TODO: we might not need to split DecodedEvent from DecodedEventType. Simplify.
pub struct DecodedEvent<const N: usize> {
    event_type: DecodedEventType,
}

impl<const N: usize> DecodedEvent<N> {
    pub fn new(comm: &mut Comm<N>, len: usize) -> Self {
        let pt = comm.buf[0];
        use crate::seph::PacketTypes;

        let event_type = match PacketTypes::from(pt) {
            PacketTypes::PacketTypeSeph | PacketTypes::PacketTypeSeEvent => {
                // Copy out SEPH payload (like original) or reinterpret in place.
                // Optimization: we can borrow slice without copying because io_buffer stores
                // [packet_type][payload...].
                Self::decode_seph_event(comm, 1)
            }
            PacketTypes::PacketTypeRawApdu
            | PacketTypes::PacketTypeUsbHidApdu
            | PacketTypes::PacketTypeUsbWebusbApdu
            | PacketTypes::PacketTypeBleApdu => Self::decode_apdu(comm, pt, 1, len),

            _ => DecodedEventType::Ignored,
        };
        Self { event_type }
    }

    pub fn into_type(self) -> DecodedEventType {
        self.event_type
    }
    pub fn from_type(event_type: DecodedEventType) -> Self {
        Self { event_type }
    }

    fn decode_seph_event(comm: &mut Comm<N>, offset: usize) -> DecodedEventType {
        use crate::seph::Events;
        let seph_buffer = &comm.buf[offset..];
        let tag = seph_buffer[0];
        match Events::from(tag) {
            // BUTTON PUSH EVENT
            #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
            Events::ButtonPushEvent => {
                #[cfg(feature = "nano_nbgl")]
                unsafe {
                    ux_process_button_event(seph_buffer.as_ptr() as *mut u8); // the cast to mutable can be removed on more recent SDKs
                }
                let button_info = seph_buffer[3] >> 1;
                if let Some(btn_evt) = get_button_event(&mut comm.buttons, button_info) {
                    return DecodedEventType::Button(btn_evt);
                }
                DecodedEventType::Ignored
            }

            // SCREEN TOUCH EVENT
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            Events::ScreenTouchEvent => unsafe {
                ux_process_finger_event(seph_buffer.as_ptr() as *mut u8); // the cast to mutable can be removed on more recent SDKs
                return DecodedEventType::Touch;
            },

            // TICKER EVENT
            Events::TickerEvent => {
                #[cfg(any(
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p",
                    feature = "nano_nbgl"
                ))]
                unsafe {
                    ux_process_ticker_event();
                }
                DecodedEventType::Ticker
            }

            // ITC EVENT
            seph::Events::ItcEvent => {
                let _len = u16::from_be_bytes([seph_buffer[1], seph_buffer[2]]) as usize;
                #[cfg(any(
                    target_os = "nanox",
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p"
                ))]
                match ItcUxEvent::from(seph_buffer[3]) {
                    seph::ItcUxEvent::AskBlePairing => unsafe {
                        G_ux_params.ux_id = BOLOS_UX_ASYNCHMODAL_PAIRING_REQUEST;
                        G_ux_params.len = 20;
                        G_ux_params.u.pairing_request.type_ = seph_buffer[4];
                        G_ux_params.u.pairing_request.pairing_info_len = (_len - 2) as u32;
                        for i in 0..G_ux_params.u.pairing_request.pairing_info_len as usize {
                            G_ux_params.u.pairing_request.pairing_info[i as usize] =
                                seph_buffer[5 + i] as i8;
                        }
                        G_ux_params.u.pairing_request.pairing_info
                            [G_ux_params.u.pairing_request.pairing_info_len as usize] = 0;
                        os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);
                    },

                    seph::ItcUxEvent::BlePairingStatus => unsafe {
                        G_ux_params.ux_id = BOLOS_UX_ASYNCHMODAL_PAIRING_STATUS;
                        G_ux_params.len = 0;
                        G_ux_params.u.pairing_status.pairing_ok = seph_buffer[4];
                        os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);
                    },

                    seph::ItcUxEvent::Redisplay => {
                        #[cfg(any(
                            target_os = "stax",
                            target_os = "flex",
                            target_os = "apex_p",
                            feature = "nano_nbgl"
                        ))]
                        unsafe {
                            nbgl_objAllowDrawing(true);
                            nbgl_screenRedraw();
                            nbgl_refresh();
                        }
                    }
                    _ => {}
                }
                DecodedEventType::Ignored
            }
            // DEFAULT EVENT
            _ => {
                #[cfg(any(
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p",
                    feature = "nano_nbgl"
                ))]
                unsafe {
                    ux_process_default_event();
                }
                #[cfg(any(target_os = "nanox", target_os = "nanosplus"))]
                if !cfg!(feature = "nano_nbgl") {
                    crate::uxapp::UxEvent::Event.request();
                }
                DecodedEventType::Ignored
            }
        }
    }

    fn decode_apdu(
        comm: &mut Comm<N>,
        packet_type: u8,
        offset: usize,
        io_len: usize,
    ) -> DecodedEventType {
        use ApduError::*;

        comm.apdu_type = packet_type;

        let apdu_buffer = &comm.buf[offset..];

        if io_len < 5 {
            return DecodedEventType::ApduError(BadLen);
        }

        let rx_len = io_len - 1;

        let header = ApduHeader {
            cla: apdu_buffer[0],
            ins: apdu_buffer[1],
            p1: apdu_buffer[2],
            p2: apdu_buffer[3],
        };
        if rx_len == 4 {
            return DecodedEventType::new_apdu(header, 4, 0);
        }
        let first_len_byte = apdu_buffer[4];

        match (first_len_byte, rx_len) {
            (0, 5) => {
                // Non-conforming zero-data APDU (TODO: per the standard, this should actually be read as a 256-byte long APDU; but that's likely to break things as lots)
                DecodedEventType::new_apdu(header, 4, 0)
            }
            (0, 6) => DecodedEventType::ApduError(BadLen),
            (0, _) => {
                let len = u16::from_be_bytes([apdu_buffer[5], apdu_buffer[6]]) as usize;
                if rx_len != len + 7 {
                    return DecodedEventType::ApduError(BadLen);
                }
                DecodedEventType::new_apdu(header, 1 + 7, len)
            }
            (len, _) => {
                if rx_len != len as usize + 5 {
                    return DecodedEventType::ApduError(BadLen);
                }
                DecodedEventType::new_apdu(header, 1 + 5, len as usize)
            }
        }
    }
}

/// High-level decoded event (no side effects).
pub enum DecodedEventType {
    Apdu {
        header: ApduHeader,
        offset: usize,
        length: usize,
    },
    ApduError(ApduError),
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    Button(ButtonEvent),
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    Touch,
    Ticker,
    // Events for which no additional handling is required after decoding it
    Ignored,
}

impl DecodedEventType {
    fn new_apdu(header: ApduHeader, offset: usize, length: usize) -> Self {
        Self::Apdu {
            header,
            offset,
            length,
        }
    }
}
