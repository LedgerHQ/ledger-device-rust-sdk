use super::{Comm, StatusWords};
use ledger_secure_sdk_sys::*;

#[cfg(feature = "stack_usage")]
use crate::io_legacy::BOLOS_INS_STACK_CONSUMPTION;
use crate::io_legacy::{
    BOLOS_INS_GET_VERSION, BOLOS_INS_QUIT, BOLOS_INS_SET_PKI_CERT, PkiLoadCertificateError,
    SyscallError,
};

/// Handle internal BOLOS APDUs (CLA = 0xB0).
///
/// `offset` and `length` locate the APDU data in `comm.buf`, as decoded.
// `p2` is only consumed by the `stack_usage` INS handler below.
#[cfg_attr(not(feature = "stack_usage"), allow(unused_variables))]
pub(crate) fn handle_bolos_apdu<const N: usize>(
    comm: &mut Comm<N>,
    ins: u8,
    p1: u8,
    p2: u8,
    offset: usize,
    length: usize,
) {
    match ins {
        // Get Information INS: retrieve App name and version
        BOLOS_INS_GET_VERSION => {
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
        BOLOS_INS_QUIT => {
            let _ = comm.begin_response().send(StatusWords::Ok);
            crate::exit_app(0);
        }
        BOLOS_INS_SET_PKI_CERT => {
            let Some(certificate) = apdu_data(&mut comm.buf, offset, length) else {
                let _ = comm.begin_response().send(StatusWords::BadLen);
                return;
            };
            let mut public_key = cx_ecfp_384_public_key_t::default();
            let err = unsafe {
                os_pki_load_certificate(
                    p1,
                    certificate.as_mut_ptr(),
                    certificate.len(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut public_key,
                )
            };
            if err != 0 {
                let _ = comm
                    .begin_response()
                    .send(SyscallError::from(PkiLoadCertificateError::from(err)));
            } else {
                let _ = comm.begin_response().send(StatusWords::Ok);
            }
        }
        #[cfg(feature = "stack_usage")]
        BOLOS_INS_STACK_CONSUMPTION => {
            crate::testing::handle_stack_consumption_apdu_new(p1, p2, comm);
        }
        // Unknown INS within BOLOS namespace
        _ => {
            let _ = comm.begin_response().send(StatusWords::BadIns);
        }
    }
}

/// The APDU data at `offset`, `length` in `buf`, if there is any and it lies
/// within `buf`.
fn apdu_data(buf: &mut [u8], offset: usize, length: usize) -> Option<&mut [u8]> {
    offset
        .checked_add(length)
        .and_then(|end| buf.get_mut(offset..end))
        .filter(|data| !data.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    // Offsets as the decoder hands them: the data of a short APDU starts at 6,
    // after the packet type, the header and Lc.
    #[test]
    fn apdu_data_is_the_decoded_data() {
        let mut buf = [0xffu8; 64];
        assert_eq!(apdu_data(&mut buf, 6, 3).map(|d| d.len()), Some(3));
        assert_eq!(apdu_data(&mut buf, 6, 58).map(|d| d.len()), Some(58));
    }

    #[test]
    fn apdu_data_rejects_missing_data() {
        // Header-only, whatever the bytes left in the buffer.
        let mut buf = [0xffu8; 273];
        assert_eq!(apdu_data(&mut buf, 4, 0).is_none(), true);
    }

    #[test]
    fn apdu_data_stays_within_small_buffers() {
        let mut buf = [0xffu8; 64];
        assert_eq!(apdu_data(&mut buf, 6, 59).is_none(), true);
        assert_eq!(apdu_data(&mut buf, 6, 255).is_none(), true);
        assert_eq!(apdu_data(&mut buf, usize::MAX, 2).is_none(), true);
    }
}
