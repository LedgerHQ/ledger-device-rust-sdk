use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseChoice C API binding.
/// Draws a generic choice page, described in a centered info (with configurable icon),
/// thanks to a button and a footer at the bottom of the page.
pub struct NbglChoice<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl SyncNBGL for NbglChoice<'_> {}

// To support nbgl_useCaseChoiceWithDetails
pub enum WarningDetailsType {
    CenteredInfoWarning,
    QRCodeWarning,
    BarListWarning,
}

// To support nbgl_useCaseChoiceWithDetails
impl From<WarningDetailsType> for nbgl_warningDetailsType_t {
    fn from(wdt: WarningDetailsType) -> Self {
        match wdt {
            WarningDetailsType::CenteredInfoWarning => CENTERED_INFO_WARNING,
            WarningDetailsType::QRCodeWarning => QRCODE_WARNING,
            WarningDetailsType::BarListWarning => BAR_LIST_WARNING,
        }
    }
}

impl<'a> NbglChoice<'a> {
    pub fn new() -> NbglChoice<'a> {
        NbglChoice { glyph: None }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglChoice<'a> {
        NbglChoice {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn ask_confirmation(
        self,
        message: Option<&str>,
        submessage: Option<&str>,
        ok_text: Option<&str>,
        ko_text: Option<&str>,
        if_accept: bool,
    ) -> NbglChoice<'a> {
        let mut screen = ConfirmationStrings::default();
        match message {
            Some(m) => screen.message = CString::new(m).unwrap(),
            None => screen.message = CString::new(DEFAULT_CONFIRM_MESSAGE).unwrap(),
        }
        match submessage {
            Some(sm) => screen.submessage = CString::new(sm).unwrap(),
            None => screen.submessage = CString::new(DEFAULT_CONFIRM_SUBMESSAGE).unwrap(),
        }
        match ok_text {
            Some(ot) => screen.ok_text = CString::new(ot).unwrap(),
            None => screen.ok_text = CString::new(DEFAULT_CONFIRM_OK_TEXT).unwrap(),
        }
        match ko_text {
            Some(kt) => screen.ko_text = CString::new(kt).unwrap(),
            None => screen.ko_text = CString::new(DEFAULT_CONFIRM_KO_TEXT).unwrap(),
        }
        if if_accept {
            #[allow(static_mut_refs)]
            unsafe {
                G_CONFIRM_ASK_WHEN_TRUE = true;
                G_CONFIRM_SCREEN[G_CONFIRM_SCREEN_WHEN_TRUE_IDX] = Some(screen);
            }
            return self;
        } else {
            #[allow(static_mut_refs)]
            unsafe {
                G_CONFIRM_ASK_WHEN_FALSE = true;
                G_CONFIRM_SCREEN[G_CONFIRM_SCREEN_WHEN_FALSE_IDX] = Some(screen);
            }
            return self;
        }
    }

    pub fn show(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
    ) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            let message = CString::new(message).unwrap();
            let sub_message = CString::new(sub_message).unwrap();
            let confirm_text = CString::new(confirm_text).unwrap();
            let cancel_text = CString::new(cancel_text).unwrap();

            self.ux_sync_init();
            nbgl_useCaseChoice(
                &icon as *const nbgl_icon_details_t,
                match message.is_empty() {
                    true => core::ptr::null(),
                    false => message.as_ptr() as *const c_char,
                },
                match sub_message.is_empty() {
                    true => core::ptr::null(),
                    false => sub_message.as_ptr() as *const c_char,
                },
                match confirm_text.is_empty() {
                    true => core::ptr::null(),
                    false => confirm_text.as_ptr() as *const c_char,
                },
                match cancel_text.is_empty() {
                    true => core::ptr::null(),
                    false => cancel_text.as_ptr() as *const c_char,
                },
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
    }
}
