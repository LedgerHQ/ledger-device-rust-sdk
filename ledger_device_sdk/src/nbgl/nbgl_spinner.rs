use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseSpinner C API binding.
/// Draws a spinner page with the given parameters. The spinner will "turn" automatically every
/// 800 ms, provided the IO event loop is running to process TickerEvents.
pub struct NbglSpinner {
    text: CString,
}

impl NbglSpinner {
    pub fn new() -> NbglSpinner {
        NbglSpinner {
            text: CString::new("").unwrap(),
        }
    }

    pub fn text(self, text: &str) -> NbglSpinner {
        NbglSpinner {
            text: CString::new(text).unwrap(),
        }
    }

    pub fn show(&self) {
        unsafe {
            nbgl_useCaseSpinner(self.text.as_ptr() as *const c_char);
        }
    }
}
