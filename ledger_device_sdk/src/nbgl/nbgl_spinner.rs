//! A wrapper around the NBGL [nbgl_useCaseSpinner](https://github.com/LedgerHQ/ledger-secure-sdk/blob/f7ba831fc72257d282060f9944644ef43b6b8e30/lib_nbgl/src/nbgl_use_case.c#L4427) C API binding.
//!
//! Draws a spinner page with the given parameters. The spinner will "turn" automatically every
//! 800 ms, provided the IO event loop is running to process TickerEvents.
use super::*;
extern crate alloc;
use alloc::ffi::CString;

/// A builder to create and show a spinner page.
#[derive(Debug, Default)]
pub struct NbglSpinner {
    text: [CString; 2],
    write_idx: usize,
    read_idx: usize,
}

impl NbglSpinner {
    /// Creates a new spinner page builder.
    pub fn new() -> NbglSpinner {
        NbglSpinner {
            text: [CString::default(), CString::default()],
            write_idx: 0,
            read_idx: 0,
        }
    }

    /// Shows the spinner with the current text.
    /// Every call make the spinner "turn" to the next text.
    /// # Arguments
    /// * `text` - The text to display below the spinner.
    /// # Returns
    /// This function does not return any value.
    /// The spinner will "turn" automatically every 800 ms, provided the IO event loop is running to process TickerEvents.
    pub fn show(&mut self, text: &str) {
        self.text[self.write_idx] = CString::new(text).unwrap();
        self.read_idx = self.write_idx;
        self.write_idx = (self.write_idx + 1) % 2;
        unsafe {
            nbgl_useCaseSpinner(self.text[self.read_idx].as_ptr() as *const c_char);
        }
    }
}
