use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseReviewStart, 
/// nbgl_useCaseStaticReview, nbgl_useCaseStaticReviewLight C API binding.
/// Used to display transaction review screens.
pub struct NbglReviewExtended<'a> {
    review_title: CString,
    review_subtitle: CString,
    reject_text: CString,
    glyph_start: Option<&'a NbglGlyph<'a>>,
    text_end: CString,
    longpress_text: CString,
    glyph_end: Option<&'a NbglGlyph<'a>>,
    light_end: bool,
}

impl SyncNBGL for NbglReviewExtended<'_> {}

impl<'a> NbglReviewExtended<'a> {
    pub fn new() -> NbglReviewExtended<'a> {
        NbglReviewExtended {
            review_title: CString::default(),
            review_subtitle: CString::default(),
            reject_text: CString::default(),
            glyph_start: None,
            text_end: CString::default(),
            longpress_text: CString::default(),
            glyph_end: None,
            light_end: false,
        }
    }

    pub fn first_page(
        self,
        review_title: &'a str,
        review_subtitle: &'a str,
        reject_text: &'a str,
        glyph_start: &'a NbglGlyph<'a>,
    ) -> NbglReviewExtended<'a> {
        NbglReviewExtended {
            review_title: CString::new(review_title).unwrap(),
            review_subtitle: CString::new(review_subtitle).unwrap(),
            reject_text: CString::new(reject_text).unwrap(),
            glyph_start: Some(glyph_start),
            ..self
        }
    }

    pub fn last_page(
        self,
        text_end: &'a str,
        longpress_text: &'a str,
        glyph_end: &'a NbglGlyph<'a>,
        light: bool,
    ) -> NbglReviewExtended<'a> {
        NbglReviewExtended {
            text_end: CString::new(text_end).unwrap(),
            longpress_text: CString::new(longpress_text).unwrap(),
            glyph_end: Some(glyph_end),
            light_end: light,
            ..self
        }
    }

    pub fn start(&self) -> SyncNbgl {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph_start {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            self.ux_sync_init();
            nbgl_useCaseReviewStart(
                &icon as *const nbgl_icon_details_t,
                self.review_title.as_ptr() as *const c_char,
                self.review_subtitle.as_ptr() as *const c_char,
                self.reject_text.as_ptr() as *const c_char,
                Some(continue_callback),
                Some(rejected_callback),
            );
            self.ux_sync_wait(false)
        }
    }

    pub fn show(&self, fields: &[Field]) -> SyncNbgl {
        unsafe {
            let v: Vec<CField> = fields
                .iter()
                .map(|f| f.into())
                .collect();
            let mut tag_value_array: Vec<nbgl_contentTagValue_t> = Vec::new();
            for field in v.iter() {
                let val = nbgl_contentTagValue_t::from(field);
                tag_value_array.push(val);
            }
            let tag_value_list = nbgl_contentTagValueList_t {
                pairs: tag_value_array.as_ptr() as *const nbgl_contentTagValue_t,
                nbPairs: fields.len() as u8,
                ..Default::default()
            };

            let icon: nbgl_icon_details_t = match self.glyph_end {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let info_long_press = nbgl_pageInfoLongPress_t {
                text: self.text_end.as_ptr() as *const c_char,
                icon: &icon as *const nbgl_icon_details_t,
                longPressText: self.longpress_text.as_ptr() as *const c_char,
                longPressToken: 0,
                tuneId: TUNE_LOOK_AT_ME,
            };

            self.ux_sync_init();
            if !self.light_end {
                nbgl_useCaseStaticReview(
                    &tag_value_list as *const nbgl_contentTagValueList_t,
                    &info_long_press as *const nbgl_pageInfoLongPress_t,
                    self.text_end.as_ptr() as *const c_char,
                    Some(choice_callback),
                );
            } else {
                nbgl_useCaseStaticReviewLight(
                    &tag_value_list as *const nbgl_contentTagValueList_t,
                    &info_long_press as *const nbgl_pageInfoLongPress_t,
                    self.text_end.as_ptr() as *const c_char,
                    Some(choice_callback),
                );
            }
            self.ux_sync_wait(false)
        }
    }
}