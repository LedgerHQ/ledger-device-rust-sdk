use super::{Comm, StatusWords, DEFAULT_BUF_SIZE};
use ledger_secure_sdk_sys::*;

/// Handle internal BOLOS APDUs (CLA = 0xB0, P1 = 0x00, P2 = 0x00).
pub(crate) fn handle_bolos_apdu<const N: usize>(comm: &mut Comm<N>, ins: u8) {
    match ins {
        // Get Information INS: retrieve App name and version
        0x01 => {
            let mut response = comm.begin_response();
            let _ = response.append(&[0x01]);
            const MAX_TAG_LENGTH: u8 = 32; // maximum length for the buffer containing app name/version.
            let mut tag_buf = [0u8; MAX_TAG_LENGTH as usize];

            // ---- App name ----
            let name_len = unsafe {
                os_registry_get_current_app_tag(
                    BOLOS_TAG_APPNAME,
                    tag_buf.as_mut_ptr(),
                    MAX_TAG_LENGTH as u32,
                )
            };

            if name_len > MAX_TAG_LENGTH.into() {
                let _ = response.send(StatusWords::Panic); // this should never happen
                return;
            }

            let _ = response.append(&[name_len as u8]);
            let _ = response.append(&tag_buf[..name_len as usize]);

            // ---- App version ----
            let ver_len = unsafe {
                os_registry_get_current_app_tag(
                    BOLOS_TAG_APPVERSION,
                    tag_buf.as_mut_ptr(),
                    MAX_TAG_LENGTH as u32,
                )
            };

            if ver_len > MAX_TAG_LENGTH.into() {
                let _ = response.send(StatusWords::Panic); // this should never happen
                return;
            }

            let _ = response.append(&[ver_len as u8]);
            let _ = response.append(&tag_buf[..ver_len as usize]);

            // ---- Flags ----
            let flags_byte = unsafe { os_flags() } as u8;
            // flags length (always 1 currently) then flags byte
            let _ = response.append(&[1]);
            let _ = response.append(&[flags_byte]);
            let _ = response.send(StatusWords::Ok);
        }
        // Quit Application INS
        0xA7 | 0xa7 => {
            let _ = comm.begin_response().send(StatusWords::Ok);
            crate::exit_app(0);
        }
        // Unknown INS within BOLOS namespace
        _ => {
            let _ = comm.begin_response().send(StatusWords::BadIns);
        }
    }
}
