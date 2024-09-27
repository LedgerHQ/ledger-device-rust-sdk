use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseChoice C API binding.
/// Draws a generic choice page, described in a centered info (with configurable icon),
/// thanks to a button and a footer at the bottom of the page.
pub struct NbglChoice<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl SyncNBGL for NbglChoice<'_> {}

impl<'a> NbglChoice<'a> {
    pub fn new() -> NbglChoice<'a> {
        NbglChoice { glyph: None }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglChoice<'a> {
        NbglChoice {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
    ) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            let message = CString::new(message).unwrap();
            let sub_message = CString::new(sub_message).unwrap();
            let confirm_text = CString::new(confirm_text).unwrap();
            let cancel_text = CString::new(cancel_text).unwrap();

            self.ux_sync_init();
            nbgl_useCaseChoice(
                &icon as *const nbgl_icon_details_t,
                message.as_ptr() as *const c_char,
                sub_message.as_ptr() as *const c_char,
                confirm_text.as_ptr() as *const c_char,
                cancel_text.as_ptr() as *const c_char,
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
    }
}
