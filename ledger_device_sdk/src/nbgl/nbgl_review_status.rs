//! A wrapper around the synchronous NBGL [nbgl_useCaseReviewStatus](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3528) C API binding.
//!
//! Draws a transient (3s) status page of the chosen type.
use super::*;

/// A builder to create and show a status review page.
pub struct NbglReviewStatus {
    status_type: StatusType,
}

impl SyncNBGL for NbglReviewStatus {}

impl<'a> NbglReviewStatus {
    /// Creates a new status review page builder.
    /// # Returns
    /// Returns a new instance of `NbglReviewStatus`.
    pub fn new() -> NbglReviewStatus {
        NbglReviewStatus {
            status_type: StatusType::Transaction,
        }
    }
    /// Sets the type of status to display.
    /// # Arguments
    /// * `status_type` - The type of status to display.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn status_type(self, status_type: StatusType) -> NbglReviewStatus {
        NbglReviewStatus { status_type }
    }

    /// Shows the status review page.
    /// # Arguments
    /// * `success` - If `true`, shows a success status; otherwise, shows a failure status.
    /// # Returns
    /// This function does not return any value.
    /// The status page is displayed for 3 seconds before automatically disappearing.
    pub fn show(&self, success: bool) {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseReviewStatus(self.status_type.to_message(success), Some(quit_callback));
            self.ux_sync_wait(false);
        }
    }
}
