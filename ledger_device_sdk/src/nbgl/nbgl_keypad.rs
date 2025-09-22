use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseChoice C API binding.
/// Draws a generic choice page, described in a centered info (with configurable icon),
/// thanks to a button and a footer at the bottom of the page.
pub struct NbglKeypad {
    title: CString,
    min_digits: u8,
    max_digits: u8,
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    back_token: u8,
    shuffled: bool,
    hide: bool,
}

impl SyncNBGL for NbglKeypad {}

static mut PIN_BUFFER: [u8; 16] = [0x00; 16];

unsafe extern "C" fn pin_callback(pin: *const u8, pin_len: u8) {
    for i in 0..pin_len {
        PIN_BUFFER[i as usize] = *pin.add(i.into());
    }   
    G_ENDED = true;
}

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
unsafe extern "C" fn action_callback(_token: ::core::ffi::c_int, _index: u8) {    
    G_RET = SyncNbgl::UxSyncRetPinRejected.into();
    G_ENDED = true;
}
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
unsafe extern "C" fn action_callback() {
    G_RET = SyncNbgl::UxSyncRetPinRejected.into();
    G_ENDED = true;
}

impl NbglKeypad {
    pub fn new() -> NbglKeypad {
        NbglKeypad {
            title: CString::new("Enter PIN").unwrap(),
            min_digits: 4,
            max_digits: 4,
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            back_token: 0,
            shuffled: true,
            hide: true,
        }
    }

    pub fn title(self, title: &str) -> NbglKeypad {
        NbglKeypad {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    pub fn min_digits(self, min: u8) -> NbglKeypad {
        NbglKeypad { min_digits: min, ..self }
    }

    pub fn max_digits(self, max: u8) -> NbglKeypad {
        NbglKeypad { max_digits: max, ..self }
    }

    pub fn hide(self, hide: bool) -> NbglKeypad {
        NbglKeypad { hide, ..self }
    }

    pub fn ask(self, pin: &[u8]) -> SyncNbgl {
        unsafe {
            self.ux_sync_init();
            match self.hide {
                true => {
                    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                    nbgl_useCaseKeypadPIN(
                        self.title.as_ptr() as *const c_char,
                        self.min_digits,
                        self.max_digits,
                        self.back_token,
                        self.shuffled,
                        TUNE_LOOK_AT_ME,
                        Some(pin_callback),
                        Some(action_callback),
                    );
                    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
                    nbgl_useCaseKeypadPIN(
                        self.title.as_ptr() as *const c_char,
                        self.min_digits,
                        self.max_digits,
                        self.shuffled,
                        Some(pin_callback),
                        Some(action_callback),
                    );

                }
                false => {
                    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                    nbgl_useCaseKeypadDigits(
                        self.title.as_ptr() as *const c_char,
                        self.min_digits,
                        self.max_digits,
                        self.back_token,
                        self.shuffled,
                        TUNE_LOOK_AT_ME,
                        Some(pin_callback),
                        Some(action_callback),
                    );
                    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
                    nbgl_useCaseKeypadDigits(
                        self.title.as_ptr() as *const c_char,
                        self.min_digits,
                        self.max_digits,
                        self.shuffled,
                        Some(pin_callback),
                        Some(action_callback),
                    );
                }
            }
            self.ux_sync_wait(false);
            // Compare with set pin code
            if pin == &PIN_BUFFER[..pin.len()] {
                return SyncNbgl::UxSyncRetPinValidated;
            } else {
                return SyncNbgl::UxSyncRetPinRejected;
            }
        }
    }
}