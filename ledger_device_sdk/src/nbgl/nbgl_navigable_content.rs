//! A wrapper around the asynchronous NBGL [nbgl_useCaseNavigableContent] C API binding.
//!
//! Not supported

use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseNavigableContent C API binding.
pub struct NbglNavigableContent {
    title: CString,
    init_page: usize,
    nb_pages: u8,
}

impl SyncNBGL for NbglNavigableContent {}

static CHOICE1: &str = "Choice 1";
static CHOICE2: &str = "Choice 2";
static CHOICE3: &str = "Choice 3";
static CHOICE4: &str = "Choice 4";

unsafe extern "C" fn navigation_callback(_page: u8, content: *mut nbgl_pageContent_t) -> bool {
    (*content).type_ = CHOICES_LIST;
    (*content).__bindgen_anon_1.choicesList = nbgl_contentRadioChoice_t {
        __bindgen_anon_1: nbgl_contentRadioChoice_t__bindgen_ty_1 {
            names: &[
                CHOICE1.as_ptr() as *const i8,
                CHOICE2.as_ptr() as *const i8,
                CHOICE3.as_ptr() as *const i8,
                CHOICE4.as_ptr() as *const i8,
            ] as *const *const i8,
        },
        token: FIRST_USER_TOKEN as u8,
        nbChoices: 4,
        initChoice: 1,
        ..Default::default()
    };
    return true;
}

unsafe extern "C" fn controls_callback(_token: ::core::ffi::c_int, _index: u8) {
    G_ENDED = true;
}

impl NbglNavigableContent {
    pub fn new() -> NbglNavigableContent {
        NbglNavigableContent {
            title: CString::default(),
            init_page: 0,
            nb_pages: 0,
        }
    }

    pub fn title(self, title: &str) -> NbglNavigableContent {
        NbglNavigableContent {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    pub fn init_page(self, init_page: usize) -> NbglNavigableContent {
        NbglNavigableContent { init_page, ..self }
    }

    pub fn nb_pages(self, nb_pages: u8) -> NbglNavigableContent {
        NbglNavigableContent { nb_pages, ..self }
    }

    pub fn show(&self) {
        unsafe {
            self.ux_sync_init();
            nbgl_useCaseNavigableContent(
                match self.title.is_empty() {
                    true => core::ptr::null(),
                    false => self.title.as_ptr() as *const c_char,
                },
                self.init_page as u8,
                self.nb_pages,
                Some(quit_callback),
                Some(navigation_callback),
                Some(controls_callback),
            );
            self.ux_sync_wait(false);
        }
    }
}
