use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseAddressReview C API binding.
/// Used to display address confirmation screens.
pub struct NbglAddressReview<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    verify_str: CString,
}

impl SyncNBGL for NbglAddressReview<'_> {}

impl<'a> NbglAddressReview<'a> {
    pub fn new() -> NbglAddressReview<'a> {
        NbglAddressReview {
            verify_str: CString::default(),
            glyph: None,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressReview<'a> {
        NbglAddressReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn verify_str(self, verify_str: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            verify_str: CString::new(verify_str).unwrap(),
            ..self
        }
    }

    pub fn show(&self, address: &str) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let address = CString::new(address).unwrap();

            self.ux_sync_init();
            nbgl_useCaseAddressReview(
                address.as_ptr(),
                core::ptr::null(),
                &icon as *const nbgl_icon_details_t,
                match self.verify_str.is_empty() {
                    true => core::ptr::null(),
                    false => self.verify_str.as_ptr(),
                },
                core::ptr::null(),
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the address, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => {
                    return true;
                }
                SyncNbgl::UxSyncRetRejected => {
                    return false;
                }
                _ => {
                    panic!("Unexpected return value from ux_sync_addressReview");
                }
            }
        }
    }
}
