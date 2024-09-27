use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseReviewStreamingStart/Continue/Finish)
/// C API binding. Used to display streamed transaction review screens.
pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
}

impl SyncNBGL for NbglStreamingReview {}

impl NbglStreamingReview {
    pub fn new() -> NbglStreamingReview {
        NbglStreamingReview {
            icon: nbgl_icon_details_t::default(),
            tx_type: TransactionType::Transaction,
            blind: false,
        }
    }

    pub fn tx_type(self, tx_type: TransactionType) -> NbglStreamingReview {
        NbglStreamingReview { tx_type, ..self }
    }

    pub fn blind(self) -> NbglStreamingReview {
        NbglStreamingReview {
            blind: true,
            ..self
        }
    }

    pub fn glyph(self, glyph: &NbglGlyph) -> NbglStreamingReview {
        NbglStreamingReview {
            icon: glyph.into(),
            ..self
        }
    }

    pub fn start(&self, title: &str, subtitle: &str) -> bool {
        unsafe {
            let title = CString::new(title).unwrap();
            let subtitle = CString::new(subtitle).unwrap();

            self.ux_sync_init();
            match self.blind {
                true => {
                    nbgl_useCaseReviewStreamingBlindSigningStart(
                        self.tx_type.to_c_type(false),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        subtitle.as_ptr() as *const c_char,
                        Some(choice_callback),
                    );
                }
                false => {
                    nbgl_useCaseReviewStreamingStart(
                        self.tx_type.to_c_type(false),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        subtitle.as_ptr() as *const c_char,
                        Some(choice_callback),
                    );
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

    pub fn continue_review(&self, fields: &[Field]) -> bool {
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

            self.ux_sync_init();
            nbgl_useCaseReviewStreamingContinue(
                &tag_value_list as *const nbgl_contentTagValueList_t,
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

    pub fn finish(&self, finish_title: &str) -> bool {
        unsafe {
            let finish_title = CString::new(finish_title).unwrap();

            self.ux_sync_init();
            nbgl_useCaseReviewStreamingFinish(
                finish_title.as_ptr() as *const c_char,
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
