use super::*;

/// A wrapper around the asynchronous NBGL ux_sync_status C API binding.
/// Draws a transient (3s) status page, either of success or failure, with the given message
pub struct NbglConfirm {}

impl SyncNBGL for NbglConfirm {}

impl NbglConfirm {
    pub fn new() -> NbglConfirm {
        NbglConfirm {}
    }

    pub fn show(&self, message: &str, submessage: Option<&str>, confirm_text: &str, cancel_text: &str) -> bool {
        unsafe {
            let message_cstr = CString::new(message).unwrap();
            let submessage_cstr = match submessage {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };
            let confirm_text_cstr = CString::new(confirm_text).unwrap();
            let cancel_text_cstr = CString::new(cancel_text).unwrap();  

            self.ux_sync_init();
            nbgl_useCaseConfirm(
                message_cstr.as_ptr() as *const c_char,
                submessage_cstr.as_ptr() as *const c_char,
                confirm_text_cstr.as_ptr() as *const c_char,
                cancel_text_cstr.as_ptr() as *const c_char,
                Some(continue_callback),
            );
            let ret = self.ux_sync_wait(false);

            if ret == SyncNbgl::UxSyncRetContinue {
                true
            } else {
                false
            }
        }
    }
}

