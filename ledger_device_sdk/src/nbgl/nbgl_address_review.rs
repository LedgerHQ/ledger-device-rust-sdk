use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseAddressReview C API binding.
/// Used to display address confirmation screens.
pub struct NbglAddressReview<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    review_title: CString,
    review_subtitle: CString,
    tag_value_list: Vec<CField>,
}

impl SyncNBGL for NbglAddressReview<'_> {}

impl<'a> NbglAddressReview<'a> {
    pub fn new() -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::default(),
            review_subtitle: CString::default(),
            glyph: None,
            tag_value_list: Vec::default(),
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressReview<'a> {
        NbglAddressReview {
            glyph: Some(glyph),
            ..self
        }
    }

    #[deprecated(note = "Use `review_title` instead")]
    pub fn verify_str(self, verify_str: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::new(verify_str).unwrap(),
            ..self
        }
    }

    pub fn review_title(self, review_title: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_title: CString::new(review_title).unwrap(),
            ..self
        }
    }

    pub fn review_subtitle(self, review_subtitle: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            review_subtitle: CString::new(review_subtitle).unwrap(),
            ..self
        }
    }

    pub fn set_tag_value_list(self, tag_value_list: &'a [Field<'a>]) -> NbglAddressReview<'a> {
        NbglAddressReview {
            tag_value_list: tag_value_list.iter().map(|f| f.into()).collect(),
            ..self
        }
    }

    pub fn show(&self, address: &str) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let address = CString::new(address).unwrap();

            let mut tag_value_array: Vec<nbgl_contentTagValue_t> = Vec::new();
            for field in self.tag_value_list.iter() {
                let val: nbgl_contentTagValue_t = field.into();
                tag_value_array.push(val);
            }

            let tag_value_list = nbgl_contentTagValueList_t {
                pairs: tag_value_array.as_ptr() as *const nbgl_contentTagValue_t,
                nbPairs: tag_value_array.len() as u8,
                ..Default::default()
            };

            self.ux_sync_init();
            nbgl_useCaseAddressReview(
                address.as_ptr(),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                match self.review_title.is_empty() {
                    true => core::ptr::null(),
                    false => self.review_title.as_ptr(),
                },
                match self.review_subtitle.is_empty() {
                    true => core::ptr::null(),
                    false => self.review_subtitle.as_ptr(),
                },
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the address, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => {
                    return true;
                }
                SyncNbgl::UxSyncRetRejected => {
                    return false;
                }
                _ => {
                    panic!("Unexpected return value from ux_sync_addressReview");
                }
            }
        }
    }
}
