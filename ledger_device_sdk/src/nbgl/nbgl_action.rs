use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseChoice C API binding.
/// Draws a generic choice page, described in a centered info (with configurable icon),
/// thanks to a button and a footer at the bottom of the page.
pub struct NbglAction<'a> {
    message: CString,
    action_text: CString,
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl SyncNBGL for NbglAction<'_> {}

impl<'a> NbglAction<'a> {
    pub fn new() -> NbglAction<'a> {
        NbglAction {
            message: CString::default(),
            action_text: CString::default(),
            glyph: None,
        }
    }

    pub fn message(self, message: &'a str) -> NbglAction<'a> {
        NbglAction {
            message: CString::new(message).unwrap(),
            ..self
        }
    }

    pub fn action_text(self, action_text: &'a str) -> NbglAction<'a> {
        NbglAction {
            action_text: CString::new(action_text).unwrap(),
            ..self
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph<'a>) -> NbglAction<'a> {
        NbglAction {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(self, nbgl_com: &mut NBGLComm) -> SyncNbgl {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            unsafe {
                G_RET = SyncNbgl::UxSyncRetError.into();
                G_ENDED = false;
            }
            nbgl_useCaseAction(
                &icon as *const nbgl_icon_details_t,
                self.message.as_ptr() as *const c_char,
                self.action_text.as_ptr() as *const c_char,
                Some(continue_callback),
            );
            unsafe {
                while !G_ENDED {
                    let apdu_received = nbgl_com.comm.next_event_ahead(true)
                        == SyncNbgl::UxSyncRetApduReceived.into();
                    if apdu_received {
                        return SyncNbgl::UxSyncRetApduReceived;
                    }
                }
                G_RET.into()
            }
        }
    }
}
