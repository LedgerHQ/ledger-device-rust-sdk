use super::*;

/// A wrapper around the synchronous NBGL ux_sync_reviewStreaming (start, continue and finish)
/// C API binding. Used to display streamed transaction review screens.
pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
}

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

    pub fn start(&mut self, title: &str, subtitle: &str) -> bool {
        unsafe {
            let title = CString::new(title).unwrap();
            let subtitle = CString::new(subtitle).unwrap();

            if self.blind {
                if !show_blind_warning() {
                    return false;
                }
            }

            let sync_ret = ux_sync_reviewStreamingStart(
                self.tx_type.to_c_type(self.blind, false),
                &self.icon as *const nbgl_icon_details_t,
                title.as_ptr() as *const c_char,
                subtitle.as_ptr() as *const c_char,
            );

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                UX_SYNC_RET_APPROVED => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
    }

    pub fn continue_review(&mut self, fields: &[Field]) -> bool {
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

            let sync_ret = ux_sync_reviewStreamingContinue(
                &tag_value_list as *const nbgl_contentTagValueList_t,
            );

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                UX_SYNC_RET_APPROVED => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
    }

    pub fn finish(&mut self, finish_title: &str) -> bool {
        unsafe {
            let finish_title = CString::new(finish_title).unwrap();
            let sync_ret = ux_sync_reviewStreamingFinish(finish_title.as_ptr() as *const c_char);

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                UX_SYNC_RET_APPROVED => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }
    }
}
