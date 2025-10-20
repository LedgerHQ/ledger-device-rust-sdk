//! A wrapper around the asynchronous NBGL [nbgl_useCaseChoice](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3584) C API binding.
//!
//! Draws a generic choice page, described in a centered info (with configurable icon),
//! thanks to a button and a footer at the bottom of the page.
use super::*;

/// A builder to create and show a choice flow.
pub struct NbglChoice<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl SyncNBGL for NbglChoice<'_> {}

impl<'a> NbglChoice<'a> {
    /// Creates a new choice flow builder.
    pub fn new() -> NbglChoice<'a> {
        NbglChoice { glyph: None }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglChoice<'a> {
        NbglChoice {
            glyph: Some(glyph),
            ..self
        }
    }

    /// Configures the confirmation dialog to be shown when the user accepts or rejects the choice.
    /// # Arguments
    /// * `message` - The main message to display in the confirmation dialog. If `None`, a default message is used.
    /// * `submessage` - An optional sub-message to display below the main message. If `None`, a default sub-message is used.
    /// * `ok_text` - The text to display on the confirmation button. If `None`, a default text is used.
    /// * `ko_text` - The text to display on the cancellation button. If `None`, a default text is used.
    /// * `if_accept` - The `if_accept` parameter determines whether the dialog is shown when the user accepts (`true`)
    /// or rejects (`false`) the choice.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn ask_confirmation(
        self,
        message: Option<&str>,
        submessage: Option<&str>,
        ok_text: Option<&str>,
        ko_text: Option<&str>,
        if_accept: bool,
    ) -> NbglChoice<'a> {
        let mut screen = ConfirmationStrings::default();
        match message {
            Some(m) => screen.message = CString::new(m).unwrap(),
            None => screen.message = CString::new(DEFAULT_CONFIRM_MESSAGE).unwrap(),
        }
        match submessage {
            Some(sm) => screen.submessage = CString::new(sm).unwrap(),
            None => screen.submessage = CString::new(DEFAULT_CONFIRM_SUBMESSAGE).unwrap(),
        }
        match ok_text {
            Some(ot) => screen.ok_text = CString::new(ot).unwrap(),
            None => screen.ok_text = CString::new(DEFAULT_CONFIRM_OK_TEXT).unwrap(),
        }
        match ko_text {
            Some(kt) => screen.ko_text = CString::new(kt).unwrap(),
            None => screen.ko_text = CString::new(DEFAULT_CONFIRM_KO_TEXT).unwrap(),
        }
        if if_accept {
            #[allow(static_mut_refs)]
            unsafe {
                G_CONFIRM_ASK_WHEN_TRUE = true;
                G_CONFIRM_SCREEN[G_CONFIRM_SCREEN_WHEN_TRUE_IDX] = Some(screen);
            }
            return self;
        } else {
            #[allow(static_mut_refs)]
            unsafe {
                G_CONFIRM_ASK_WHEN_FALSE = true;
                G_CONFIRM_SCREEN[G_CONFIRM_SCREEN_WHEN_FALSE_IDX] = Some(screen);
            }
            return self;
        }
    }

    /// Shows the choice flow.
    ///
    /// # Arguments
    ///
    /// * `message` - The main message to display in the center of the page.
    /// * `sub_message` - An optional sub-message to display below the main message.
    /// * `confirm_text` - The text to display on the confirmation button.
    /// * `cancel_text` - The text to display on the cancellation button.
    ///
    /// # Returns
    ///
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
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
                match message.is_empty() {
                    true => core::ptr::null(),
                    false => message.as_ptr() as *const c_char,
                },
                match sub_message.is_empty() {
                    true => core::ptr::null(),
                    false => sub_message.as_ptr() as *const c_char,
                },
                match confirm_text.is_empty() {
                    true => core::ptr::null(),
                    false => confirm_text.as_ptr() as *const c_char,
                },
                match cancel_text.is_empty() {
                    true => core::ptr::null(),
                    false => cancel_text.as_ptr() as *const c_char,
                },
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
