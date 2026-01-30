//! A wrapper around the NBGL [nbgl_useCaseStatus](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3497) C API binding.
//!
//! Draws a transient (3s) status page, either of success or failure, with the given message
use super::*;

/// A builder to create and show a status page.
pub struct NbglStatus {
    text: CString,
}

impl SyncNBGL for NbglStatus {}

impl NbglStatus {
    /// Creates a new status page builder.
    pub fn new() -> NbglStatus {
        NbglStatus {
            text: CString::default(),
        }
    }

    /// Sets the text to display below the status icon.
    /// # Arguments
    /// * `text` - The text to display below the status icon.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn text(self, text: &str) -> NbglStatus {
        NbglStatus {
            text: CString::new(text).unwrap(),
        }
    }

    fn show_internal(&self, success: bool) {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseStatus(
                match self.text.is_empty() {
                    true => core::ptr::null(),
                    false => self.text.as_ptr() as *const c_char,
                },
                success,
                Some(quit_callback),
            );
            self.ux_sync_wait(false);
        }
    }

    /// Shows the status page with the provided text.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `success` - If `true`, shows a success status; otherwise, shows a failure status.
    /// # Returns
    /// This function does not return any value.
    /// The status page is displayed for 3 seconds before automatically disappearing.
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(&self, _comm: &mut crate::io::Comm<N>, success: bool) {
        self.show_internal(success)
    }

    /// Shows the status page with the provided text.
    /// # Arguments
    /// * `success` - If `true`, shows a success status; otherwise, shows a failure status.
    /// # Returns
    /// This function does not return any value.
    /// The status page is displayed for 3 seconds before automatically disappearing.
    #[cfg(not(feature = "io_new"))]
    pub fn show(&self, success: bool) {
        self.show_internal(success)
    }
}
