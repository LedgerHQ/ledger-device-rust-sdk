use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseReview C API binding.
/// Used to display transaction review screens.
pub struct NbglReview<'a> {
    title: CString,
    subtitle: CString,
    finish_title: CString,
    glyph: Option<&'a NbglGlyph<'a>>,
    tx_type: TransactionType,
    blind: bool,
    light: bool,
}

impl SyncNBGL for NbglReview<'_> {}

impl<'a> NbglReview<'a> {
    pub fn new() -> NbglReview<'a> {
        NbglReview {
            title: CString::default(),
            subtitle: CString::default(),
            finish_title: CString::default(),
            glyph: None,
            tx_type: TransactionType::Transaction,
            blind: false,
            light: false,
        }
    }

    pub fn tx_type(self, tx_type: TransactionType) -> NbglReview<'a> {
        NbglReview { tx_type, ..self }
    }

    pub fn blind(self) -> NbglReview<'a> {
        NbglReview {
            blind: true,
            ..self
        }
    }

    pub fn light(self) -> NbglReview<'a> {
        NbglReview {
            light: true,
            ..self
        }
    }

    pub fn titles(
        self,
        title: &'a str,
        subtitle: &'a str,
        finish_title: &'a str,
    ) -> NbglReview<'a> {
        NbglReview {
            title: CString::new(title).unwrap(),
            subtitle: CString::new(subtitle).unwrap(),
            finish_title: CString::new(finish_title).unwrap(),
            ..self
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglReview<'a> {
        NbglReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(&self, fields: &[Field]) -> bool {
        unsafe {
            let v: Vec<CField> = fields
                .iter()
                .map(|f| CField {
                    name: CString::new(f.name).unwrap(),
                    value: CString::new(f.value).unwrap(),
                })
                .collect();

            // Fill the tag_value_array with the fields converted to nbgl_contentTagValue_t
            let mut tag_value_array: Vec<nbgl_contentTagValue_t> = Vec::new();
            for field in v.iter() {
                let val = nbgl_contentTagValue_t {
                    item: field.name.as_ptr() as *const i8,
                    value: field.value.as_ptr() as *const i8,
                    ..Default::default()
                };
                tag_value_array.push(val);
            }

            // Create the tag_value_list with the tag_value_array.
            let tag_value_list = nbgl_contentTagValueList_t {
                pairs: tag_value_array.as_ptr() as *const nbgl_contentTagValue_t,
                nbPairs: fields.len() as u8,
                ..Default::default()
            };

            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            // Show the review on the device.
            self.ux_sync_init();
            match self.blind {
                true => {
                    nbgl_useCaseReviewBlindSigning(
                        self.tx_type.to_c_type(false),
                        &tag_value_list as *const nbgl_contentTagValueList_t,
                        &icon as *const nbgl_icon_details_t,
                        match self.title.is_empty() {
                            true => core::ptr::null(),
                            false => self.title.as_ptr() as *const c_char,
                        },
                        match self.subtitle.is_empty() {
                            true => core::ptr::null(),
                            false => self.subtitle.as_ptr() as *const c_char,
                        },
                        match self.finish_title.is_empty() {
                            true => core::ptr::null(),
                            false => self.finish_title.as_ptr() as *const c_char,
                        },
                        core::ptr::null(),
                        Some(choice_callback),
                    );
                }
                false => {
                    if self.light {
                        nbgl_useCaseReviewLight(
                            self.tx_type.to_c_type(false),
                            &tag_value_list as *const nbgl_contentTagValueList_t,
                            &icon as *const nbgl_icon_details_t,
                            match self.title.is_empty() {
                                true => core::ptr::null(),
                                false => self.title.as_ptr() as *const c_char,
                            },
                            match self.subtitle.is_empty() {
                                true => core::ptr::null(),
                                false => self.subtitle.as_ptr() as *const c_char,
                            },
                            match self.finish_title.is_empty() {
                                true => core::ptr::null(),
                                false => self.finish_title.as_ptr() as *const c_char,
                            },
                            Some(choice_callback),
                        );
                    } else {
                        nbgl_useCaseReview(
                            self.tx_type.to_c_type(false),
                            &tag_value_list as *const nbgl_contentTagValueList_t,
                            &icon as *const nbgl_icon_details_t,
                            match self.title.is_empty() {
                                true => core::ptr::null(),
                                false => self.title.as_ptr() as *const c_char,
                            },
                            match self.subtitle.is_empty() {
                                true => core::ptr::null(),
                                false => self.subtitle.as_ptr() as *const c_char,
                            },
                            match self.finish_title.is_empty() {
                                true => core::ptr::null(),
                                false => self.finish_title.as_ptr() as *const c_char,
                            },
                            Some(choice_callback),
                        );
                    }
                }
            }
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
