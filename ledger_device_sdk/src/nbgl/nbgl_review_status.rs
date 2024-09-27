use super::*;

/// A wrapper around the synchronous NBGL ux_sync_reviewStatus C API binding.
/// Draws a transient (3s) status page of the chosen type.
pub struct NbglReviewStatus {
    status_type: StatusType,
}

impl SyncNBGL for NbglReviewStatus {}

impl<'a> NbglReviewStatus {
    pub fn new() -> NbglReviewStatus {
        NbglReviewStatus {
            status_type: StatusType::Transaction,
        }
    }

    pub fn status_type(self, status_type: StatusType) -> NbglReviewStatus {
        NbglReviewStatus { status_type }
    }

    pub fn show(&self, success: bool) {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseReviewStatus(self.status_type.to_message(success), Some(quit_callback));
            self.ux_sync_wait(false);
        }
    }
}
