//! A wrapper around the asynchronous NBGL [nbgl_useCaseKeypad](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L4565) C API binding.
//!
//! Draws a keypad for user input, allowing for PIN entry and other numeric input.
use super::*;

/// A builder to create and show a keypad for user input.
pub struct NbglKeypad {
    title: CString,
    min_digits: u8,
    max_digits: u8,
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

unsafe extern "C" fn action_callback() {
    G_RET = SyncNbgl::UxSyncRetPinRejected.into();
    G_ENDED = true;
}

impl NbglKeypad {
    /// Creates a new keypad builder with default settings.
    /// By default, the title is "Enter PIN", minimum and maximum digits are set to 4,
    /// the keypad is shuffled, and input is hidden.
    /// # Returns
    /// Returns a new instance of `NbglKeypad`.
    pub fn new() -> NbglKeypad {
        NbglKeypad {
            title: CString::new("Enter PIN").unwrap(),
            min_digits: 4,
            max_digits: 4,
            shuffled: true,
            hide: true,
        }
    }

    /// Sets the title to display at the top of the keypad.
    /// # Arguments
    /// * `title` - The title to display at the top of the keypad.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn title(self, title: &str) -> NbglKeypad {
        NbglKeypad {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    /// Sets the minimum number of digits required for input.
    /// # Arguments
    /// * `min` - The minimum number of digits required for input.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn min_digits(self, min: u8) -> NbglKeypad {
        NbglKeypad {
            min_digits: min,
            ..self
        }
    }
    /// Sets the maximum number of digits allowed for input.
    /// # Arguments
    /// * `max` - The maximum number of digits allowed for input.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn max_digits(self, max: u8) -> NbglKeypad {
        NbglKeypad {
            max_digits: max,
            ..self
        }
    }

    /// Sets whether the keypad should be shuffled.
    /// # Arguments
    /// * `shuffle` - If `true`, the keypad will be shuffled; otherwise, it will be in a fixed order.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn shuffled(self, shuffle: bool) -> NbglKeypad {
        NbglKeypad {
            shuffled: shuffle,
            ..self
        }
    }

    /// Sets whether the input should be hidden (e.g., for PIN entry).
    /// # Arguments
    /// * `hide` - If `true`, the input will be hidden; otherwise, it will be visible.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn hide(self, hide: bool) -> NbglKeypad {
        NbglKeypad { hide, ..self }
    }

    fn ask_internal(self, pin: &[u8]) -> SyncNbgl {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseKeypad(
                self.title.as_ptr() as *const c_char,
                self.min_digits,
                self.max_digits,
                self.shuffled,
                self.hide,
                Some(pin_callback),
                Some(action_callback),
            );
            self.ux_sync_wait(false);
            // Compare with set pin code
            if pin == &PIN_BUFFER[..pin.len()] {
                return SyncNbgl::UxSyncRetPinValidated;
            } else {
                return SyncNbgl::UxSyncRetPinRejected;
            }
        }
    }

    /// Shows the keypad and waits for user input.
    /// # Arguments
    /// * `pin` - A slice containing the expected PIN for validation.
    /// # Returns
    /// Returns `true` if the entered PIN matches the expected PIN,
    /// otherwise returns `false`.
    #[cfg(feature = "io_new")]
    pub fn ask<const N: usize>(self, _comm: &mut crate::io::Comm<N>, pin: &[u8]) -> bool {
        self.ask_internal(pin) == SyncNbgl::UxSyncRetPinValidated
    }

    /// Shows the keypad and waits for user input.
    /// # Arguments
    /// * `pin` - A slice containing the expected PIN for validation.
    /// # Returns
    /// Returns `SyncNbgl::UxSyncRetPinValidated` if the entered PIN matches the expected PIN,
    /// otherwise returns `SyncNbgl::UxSyncRetPinRejected`.
    #[cfg(not(feature = "io_new"))]
    pub fn ask(self, pin: &[u8]) -> SyncNbgl {
        self.ask_internal(pin)
    }
}
