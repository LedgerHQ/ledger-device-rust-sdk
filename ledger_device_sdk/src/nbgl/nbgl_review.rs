use super::*;

/// A wrapper around the synchronous NBGL ux_sync_review C API binding.
/// Used to display transaction review screens.
pub struct NbglReview<'a> {
    title: CString,
    subtitle: CString,
    finish_title: CString,
    glyph: Option<&'a NbglGlyph<'a>>,
    tx_type: TransactionType,
    blind: bool,
}

impl<'a> NbglReview<'a> {
    pub fn new() -> NbglReview<'a> {
        NbglReview {
            title: CString::new("").unwrap(),
            subtitle: CString::new("").unwrap(),
            finish_title: CString::new("").unwrap(),
            glyph: None,
            tx_type: TransactionType::Transaction,
            blind: false,
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

    pub fn show(&mut self, fields: &[Field]) -> bool {
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

            if self.blind {
                if !show_blind_warning() {
                    return false;
                }
            }

            // Show the review on the device.
            let sync_ret = ux_sync_review(
                self.tx_type.to_c_type(self.blind, false),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                self.title.as_ptr() as *const c_char,
                self.subtitle.as_ptr() as *const c_char,
                self.finish_title.as_ptr() as *const c_char,
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
}