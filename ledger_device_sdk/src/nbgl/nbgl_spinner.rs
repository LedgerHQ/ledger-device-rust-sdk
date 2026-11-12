//! A wrapper around the NBGL [nbgl_useCaseSpinner](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L4510) C API binding.
//!
//! Draws a spinner page with the given parameters. The spinner will "turn" automatically every
//! 800 ms, provided the IO event loop is running to process TickerEvents.
use super::*;
extern crate alloc;
use alloc::ffi::CString;

/// Texts handed to the C spinner. NBGL keeps the pointer it is given, redraws
/// from it and compares it with the next text, so the texts are owned here
/// rather than by the [`NbglSpinner`] that showed them: the current and the
/// previous one stay alive until two newer texts replace them.
static mut TEXTS: [Option<CString>; 2] = [None, None];
static mut NEXT_SLOT: usize = 0;

/// Stores `text` for the C spinner and returns the pointer to hand to it.
fn store_text(text: &str) -> *const c_char {
    let text = CString::new(text).unwrap();
    unsafe {
        let slot = NEXT_SLOT;
        NEXT_SLOT = (slot + 1) % 2;
        let texts = &raw mut TEXTS;
        (*texts)[slot].insert(text).as_ptr() as *const c_char
    }
}

/// A builder to create and show a spinner page.
#[derive(Debug, Default)]
pub struct NbglSpinner {}

impl NbglSpinner {
    /// Creates a new spinner page builder.
    pub fn new() -> NbglSpinner {
        NbglSpinner {}
    }

    fn show_internal(&mut self, text: &str) {
        let text = store_text(text);
        unsafe {
            nbgl_useCaseSpinner(text);
        }
    }

    /// Shows the spinner with the current text.
    /// Every call make the spinner "turn" to the next text.
    /// # Arguments
    /// * `text` - The text to display below the spinner.
    /// # Returns
    /// This function does not return any value.
    /// The spinner will "turn" automatically every 800 ms, provided the IO event loop is running to process TickerEvents.
    ///
    /// The text is kept by the SDK while the spinner is displayed, so the
    /// `NbglSpinner` itself may be dropped once this returns.
    pub fn show(&mut self, text: &str) {
        self.show_internal(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use core::ffi::CStr;
    use testmacro::test_item as test;

    fn read(ptr: *const c_char) -> &'static [u8] {
        unsafe { CStr::from_ptr(ptr) }.to_bytes()
    }

    #[test]
    fn spinner_text_outlives_the_spinner() {
        let shown = {
            let _spinner = NbglSpinner::new();
            store_text("Signing")
        };
        assert_eq!(read(shown), b"Signing");
    }

    #[test]
    fn previous_spinner_text_stays_alive_for_the_next_update() {
        let previous = store_text("Please wait");
        let current = store_text("Signing");
        assert_eq!(read(previous), b"Please wait");
        assert_eq!(read(current), b"Signing");
    }
}
