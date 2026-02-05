use super::{Comm, StatusWords};
use ledger_secure_sdk_sys::*;

use crate::io_legacy::{
    PkiLoadCertificateError, SyscallError, BOLOS_INS_GET_VERSION, BOLOS_INS_QUIT,
    BOLOS_INS_SET_PKI_CERT,
};

/// Handle internal BOLOS APDUs (CLA = 0xB0, P1 = 0x00, P2 = 0x00).
pub(crate) fn handle_bolos_apdu<const N: usize>(comm: &mut Comm<N>, ins: u8) {
    match ins {
        // Get Information INS: retrieve App name and version
        BOLOS_INS_GET_VERSION => {
            const APP_NAME: &[u8] = option_env!("APP_NAME").unwrap_or("Unknown").as_bytes();
            const APP_VERSION: &[u8] = option_env!("APP_VERSION").unwrap_or("Unknown").as_bytes();

            let mut response = comm.begin_response();

            if APP_NAME.len() + APP_VERSION.len() + 5 > N {
                let _ = response.send(StatusWords::Panic); // this should never happen
                return;
            }

            let _ = response.append(&[0x01]);

            // Name length and value
            let _ = response.append(&[APP_NAME.len() as u8]);
            let _ = response.append(APP_NAME);

            // Version length and value
            let _ = response.append(&[APP_VERSION.len() as u8]);
            let _ = response.append(APP_VERSION);

            // ---- Flags ----
            let flags_byte = unsafe { os_flags() } as u8;
            // flags length (always 1 currently) then flags byte
            let _ = response.append(&[1]);
            let _ = response.append(&[flags_byte]);
            let _ = response.send(StatusWords::Ok);
        }
        // Quit Application INS
        BOLOS_INS_QUIT => {
            let _ = comm.begin_response().send(StatusWords::Ok);
            crate::exit_app(0);
        }
        BOLOS_INS_SET_PKI_CERT => unsafe {
            let public_key = cx_ecfp_384_public_key_t::default();
            let err = os_pki_load_certificate(
                comm.buf[3],                // P1
                comm.buf[6..].as_mut_ptr(), // Data
                comm.buf[5] as usize,       // Length
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &public_key as *const cx_ecfp_384_public_key_t as *mut cx_ecfp_384_public_key_t,
            );
            if err != 0 {
                let _ = comm
                    .begin_response()
                    .send(SyscallError::from(PkiLoadCertificateError::from(err)));
            } else {
                let _ = comm.begin_response().send(StatusWords::Ok);
            }
        },
        // Unknown INS within BOLOS namespace
        _ => {
            let _ = comm.begin_response().send(StatusWords::BadIns);
        }
    }
}
