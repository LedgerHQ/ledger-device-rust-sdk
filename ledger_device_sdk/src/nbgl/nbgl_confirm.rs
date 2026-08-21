//! A wrapper around the NBGL [nbgl_useCaseConfirm](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3909) C API binding.
//!
//! Draws a modal asking the user to confirm an action, with a black button and
//! a footer to dismiss.
//!
//! Two things follow from it being a *modal*, and shape the API:
//!
//! * It is drawn over whatever is already on screen, and dismissing it reveals
//!   that again. Showing one with nothing underneath leaves a blank screen.
//! * The C callback runs only when the button is touched. Dismissing through
//!   the footer reports nothing at all, so there is no way to wait for "the
//!   user decided" — hence [`NbglConfirm::show_and_return`] does not block and
//!   only tells you about confirmation.
//!
//! To ask for confirmation as part of accepting or rejecting a review, use
//! [`NbglChoice::ask_confirmation`] instead, which handles both outcomes.
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::NbglConfirm;
//! fn on_confirm() {
//!     // the user touched the button
//! }
//!
//! NbglConfirm::new()
//!     .message("Delete this account?")
//!     .sub_message("This cannot be undone.")
//!     .texts("Delete", "Cancel")
//!     .show_and_return(on_confirm);
//! ```

use super::*;

/// App function to run when the confirmation button is touched.
///
/// `nbgl_callback_t` carries no user data, so the app's function is parked here
/// for [`confirm_callback`] to find.
static mut CONFIRM_CALLBACK: Option<fn()> = None;

/// Callback registered with NBGL, forwarding to the app's own function.
unsafe extern "C" fn confirm_callback() {
    if let Some(callback) = unsafe { CONFIRM_CALLBACK } {
        callback();
    }
}

/// A builder to create and show a confirmation modal.
#[derive(Default)]
pub struct NbglConfirm {
    message: CString,
    sub_message: CString,
    confirm_text: CString,
    cancel_text: CString,
}

impl NbglConfirm {
    /// Creates a new confirmation modal builder.
    pub fn new() -> NbglConfirm {
        NbglConfirm::default()
    }

    /// Sets the main message, shown in large case.
    pub fn message(self, message: &str) -> NbglConfirm {
        NbglConfirm {
            message: CString::new(message).unwrap(),
            ..self
        }
    }

    /// Sets the secondary message, shown under the main one.
    pub fn sub_message(self, sub_message: &str) -> NbglConfirm {
        NbglConfirm {
            sub_message: CString::new(sub_message).unwrap(),
            ..self
        }
    }

    /// Sets the button and footer labels.
    /// # Arguments
    /// * `confirm_text` - Label of the button that confirms.
    /// * `cancel_text` - Label of the footer that dismisses.
    pub fn texts(self, confirm_text: &str, cancel_text: &str) -> NbglConfirm {
        NbglConfirm {
            confirm_text: CString::new(confirm_text).unwrap(),
            cancel_text: CString::new(cancel_text).unwrap(),
            ..self
        }
    }

    /// Draws the modal and returns immediately.
    ///
    /// `on_confirm` runs if the user touches the button. Dismissing through the
    /// footer runs nothing: the C API reports only confirmation, so an app that
    /// needs to know about dismissal has to infer it from what happens next.
    ///
    /// `on_confirm` runs inside NBGL's event dispatch, so it should return
    /// promptly.
    pub fn show_and_return(&self, on_confirm: fn()) {
        unsafe {
            CONFIRM_CALLBACK = Some(on_confirm);
            nbgl_useCaseConfirm(
                match self.message.is_empty() {
                    true => core::ptr::null(),
                    false => self.message.as_ptr() as *const c_char,
                },
                match self.sub_message.is_empty() {
                    true => core::ptr::null(),
                    false => self.sub_message.as_ptr() as *const c_char,
                },
                match self.confirm_text.is_empty() {
                    true => core::ptr::null(),
                    false => self.confirm_text.as_ptr() as *const c_char,
                },
                match self.cancel_text.is_empty() {
                    true => core::ptr::null(),
                    false => self.cancel_text.as_ptr() as *const c_char,
                },
                Some(confirm_callback),
            );
        }
    }
}
