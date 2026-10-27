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
//! # On Nano
//!
//! Nano NBGL has no modal layer. The confirmation replaces the state of the
//! screen it is drawn over: dismissing it repaints that screen, but the screen
//! no longer works, and the app is not told. Use [`NbglChoice`] there instead,
//! which reports both outcomes, and draw the previous screen again afterwards
//! (see the `nbgl_home_and_settings` example).
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
use alloc::rc::Rc;
use core::ffi::CStr;

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

/// The modal currently displayed, whose texts NBGL points to.
///
/// Only replaced once the next one has been handed to C.
static mut DISPLAYED: Option<NbglConfirm> = None;

/// A builder to create and show a confirmation modal.
///
/// Its strings are reference-counted so that displaying it shares them rather
/// than copying them. Setters always replace them, never mutate them.
#[derive(Default, Clone)]
pub struct NbglConfirm {
    message: Rc<CStr>,
    sub_message: Rc<CStr>,
    confirm_text: Rc<CStr>,
    cancel_text: Rc<CStr>,
}

impl NbglConfirm {
    /// Creates a new confirmation modal builder.
    pub fn new() -> NbglConfirm {
        NbglConfirm::default()
    }

    /// Sets the main message, shown in large case.
    pub fn message(self, message: &str) -> NbglConfirm {
        NbglConfirm {
            message: CString::new(message).unwrap().into(),
            ..self
        }
    }

    /// Sets the secondary message, shown under the main one.
    pub fn sub_message(self, sub_message: &str) -> NbglConfirm {
        NbglConfirm {
            sub_message: CString::new(sub_message).unwrap().into(),
            ..self
        }
    }

    /// Sets the button and footer labels.
    /// # Arguments
    /// * `confirm_text` - Label of the button that confirms.
    /// * `cancel_text` - Label of the footer that dismisses.
    pub fn texts(self, confirm_text: &str, cancel_text: &str) -> NbglConfirm {
        NbglConfirm {
            confirm_text: CString::new(confirm_text).unwrap().into(),
            cancel_text: CString::new(cancel_text).unwrap().into(),
            ..self
        }
    }

    /// Draws the modal and returns immediately.
    ///
    /// `on_confirm` runs if the user touches the button. Dismissing through the
    /// footer runs nothing: the C API reports only confirmation, so an app that
    /// needs to know about dismissal has to infer it from what happens next.
    ///
    /// `on_confirm` runs inside NBGL's event dispatch, while `Comm` is still
    /// handling the touch: it must not use `Comm` nor block on another screen.
    /// Record what is due and act on it once `Comm::next_event` has returned
    /// (see the `nbgl_home_and_settings` example).
    ///
    /// The SDK keeps its own reference to the texts while the modal is displayed,
    /// so the builder may be dropped afterwards.
    ///
    /// On Nano it is not a modal; see the [module documentation](self).
    pub fn show_and_return(&self, on_confirm: fn()) {
        // Shares the strings, whose heap buffers C points to; they stay put
        // when `shown` is moved into `DISPLAYED` below.
        let shown = self.clone();
        unsafe {
            CONFIRM_CALLBACK = Some(on_confirm);
            nbgl_useCaseConfirm(
                match shown.message.is_empty() {
                    true => core::ptr::null(),
                    false => shown.message.as_ptr() as *const c_char,
                },
                match shown.sub_message.is_empty() {
                    true => core::ptr::null(),
                    false => shown.sub_message.as_ptr() as *const c_char,
                },
                match shown.confirm_text.is_empty() {
                    true => core::ptr::null(),
                    false => shown.confirm_text.as_ptr() as *const c_char,
                },
                match shown.cancel_text.is_empty() {
                    true => core::ptr::null(),
                    false => shown.cancel_text.as_ptr() as *const c_char,
                },
                Some(confirm_callback),
            );
            // C now points into `shown`, so the previous copy can be released.
            DISPLAYED = Some(shown);
        }
    }
}
