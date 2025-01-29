use super::*;
extern crate alloc;
use alloc::ffi::CString;

/// A wrapper around the asynchronous NBGL nbgl_useCaseSpinner C API binding.
/// Draws a spinner page with the given parameters. The spinner will "turn" automatically every
/// 800 ms, provided the IO event loop is running to process TickerEvents.
#[derive(Debug, Default)]
pub struct NbglSpinner {
    text: [CString; 2],
    write_idx: usize,
    read_idx: usize,
}

impl NbglSpinner {
    pub fn new() -> NbglSpinner {
        NbglSpinner {
            text: [CString::default(), CString::default()],
            write_idx: 0,
            read_idx: 0,
        }
    }

    /// Shows the spinner with the current text.
    /// Every call make the spinner "turn" to the next text.
    pub fn show(&mut self, text: &str) {
        self.text[self.write_idx] = CString::new(text).unwrap();
        self.read_idx = self.write_idx;
        self.write_idx = (self.write_idx + 1) % 2;
        unsafe {
            nbgl_useCaseSpinner(self.text[self.read_idx].as_ptr() as *const c_char);
        }
    }
}
