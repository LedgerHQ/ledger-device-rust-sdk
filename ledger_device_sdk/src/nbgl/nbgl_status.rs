use super::*;

/// A wrapper around the synchronous NBGL ux_sync_status C API binding.
/// Draws a transient (3s) status page, either of success or failure, with the given message
pub struct NbglStatus {
    text: CString,
}

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
            ux_sync_status(self.text.as_ptr() as *const c_char, success);
        }
    }
}
