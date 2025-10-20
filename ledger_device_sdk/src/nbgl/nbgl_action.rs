//! A wrapper around the asynchronous NBGL [nbgl_useCaseAction](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3730) C API binding.
//!
//! Draws a page to represent an action, described in a centered info (with given icon),
//! thanks to a button at the bottom of the page.

use super::*;

/// A builder to create and show an action page.
pub struct NbglAction<'a> {
    message: CString,
    action_text: CString,
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl SyncNBGL for NbglAction<'_> {}

impl<'a> NbglAction<'a> {
    /// Creates a new action page builder.
    pub fn new() -> NbglAction<'a> {
        NbglAction {
            message: CString::default(),
            action_text: CString::default(),
            glyph: None,
        }
    }
    /// Sets the message to display in the center of the page.
    /// # Arguments
    /// * `message` - The main message to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn message(self, message: &'a str) -> NbglAction<'a> {
        NbglAction {
            message: CString::new(message).unwrap(),
            ..self
        }
    }

    /// Sets the text to display on the button at the bottom of the page.
    /// # Arguments
    /// * `action_text` - The text to display on the button at the bottom of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn action_text(self, action_text: &'a str) -> NbglAction<'a> {
        NbglAction {
            action_text: CString::new(action_text).unwrap(),
            ..self
        }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph<'a>) -> NbglAction<'a> {
        NbglAction {
            glyph: Some(glyph),
            ..self
        }
    }

    /// Shows the action page.
    pub fn show(self) -> SyncNbgl {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            self.ux_sync_init();
            nbgl_useCaseAction(
                &icon as *const nbgl_icon_details_t,
                self.message.as_ptr() as *const c_char,
                self.action_text.as_ptr() as *const c_char,
                Some(continue_callback),
            );
            self.ux_sync_wait(false)
        }
    }
}
