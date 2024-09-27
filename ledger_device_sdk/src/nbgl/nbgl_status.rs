use super::*;

/// A wrapper around the asynchronous NBGL ux_sync_status C API binding.
/// Draws a transient (3s) status page, either of success or failure, with the given message
pub struct NbglStatus {
    text: CString,
}

impl SyncNBGL for NbglStatus {}

impl NbglStatus {
    pub fn new() -> NbglStatus {
        NbglStatus {
            text: CString::new("").unwrap(),
        }
    }

    pub fn text(self, text: &str) -> NbglStatus {
        NbglStatus {
            text: CString::new(text).unwrap(),
        }
    }

    pub fn show(&self, success: bool) {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseStatus(
                self.text.as_ptr() as *const c_char,
                success,
                Some(quit_callback),
            );
            self.ux_sync_wait(false);
        }
    }
}
