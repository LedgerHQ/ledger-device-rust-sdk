//! A wrapper around the asynchronous NBGL [nbgl_useCaseReviewStart](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3563),
//! [nbgl_useCaseStaticReview](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3838), [nbgl_useCaseStaticReviewLight](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3894) C API binding.
//!
//! Used to display transaction review screens.
use super::*;

/// A builder to create and show an extended review flow.
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
    /// Creates a new extended review flow builder.
    /// # Returns
    /// Returns a new instance of `NbglReviewExtended`.
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

    /// Configures the first page of the review flow.
    /// # Arguments
    /// * `review_title` - The title to display at the top of the first page.
    /// * `review_subtitle` - The subtitle to display below the title on the first page.
    /// * `reject_text` - The text to display on the reject button on the first page.
    /// * `glyph_start` - The icon to display in the center of the first page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
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

    /// Configures the last page of the review flow.
    /// # Arguments
    /// * `text_end` - The text to display at the top of the last page.
    /// * `longpress_text` - The text to display when the user long-presses the button on the last page.
    /// * `glyph_end` - The icon to display in the center of the last page.
    /// * `light` - If `true`, the last page will be displayed in light mode; otherwise, it will be in standard mode.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
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

    fn start_internal(&self) -> SyncNbgl {
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

    /// Starts the review flow by displaying the first page.
    /// # Returns
    /// Returns `Ok(true)`` if the user accepts the review,
    /// `Ok(false)` if the user rejects it,
    /// or `Err(u8)` with the error code in case of an error.
    #[cfg(feature = "io_new")]
    pub fn start(&self) -> Result<bool, u8> {
        let ret = self.start_internal();
        match ret {
            SyncNbgl::UxSyncRetContinue => Ok(true),
            SyncNbgl::UxSyncRetRejected => Ok(false),
            _ => Err(u8::from(ret)),
        }
    }

    /// Starts the review flow by displaying the first page.
    /// # Returns
    /// Returns `SyncNbgl::UxSyncRetContinue` if the user accepts the review,
    /// `SyncNbgl::UxSyncRetRejected` if the user rejects it,
    /// or another `SyncNbgl` variant in case of an error.
    #[cfg(not(feature = "io_new"))]
    pub fn start(&self) -> SyncNbgl {
        self.start_internal()
    }

    /// Shows the extended review flow with the provided fields on the review pages (internal implementation).
    fn show_internal(&self, fields: &[Field]) -> SyncNbgl {
        unsafe {
            let v: Vec<CField> = fields.iter().map(|f| f.into()).collect();
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

    /// Shows the extended review flow with the provided fields on the review pages.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns `Ok(true)` if the user accepts the review,
    /// `Ok(false)` if the user rejects it,
    /// or `Err(u8)` with the error code in case of an error.
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        fields: &[Field],
    ) -> Result<bool, u8> {
        let ret = self.show_internal(fields);
        match ret {
            SyncNbgl::UxSyncRetApproved => Ok(true),
            SyncNbgl::UxSyncRetRejected => Ok(false),
            _ => Err(u8::from(ret)),
        }
    }

    /// Shows the extended review flow with the provided fields on the review pages.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns `SyncNbgl::UxSyncRetOK` if the user accepts the review,
    /// `SyncNbgl::UxSyncRetUserAborted` if the user rejects it,
    /// or another `SyncNbgl` variant in case of an error.
    #[cfg(not(feature = "io_new"))]
    pub fn show(&self, fields: &[Field]) -> SyncNbgl {
        self.show_internal(fields)
    }
}
