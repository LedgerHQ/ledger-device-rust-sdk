//! A wrapper around the asynchronous NBGL [nbgl_useCaseReview](https://github.com/LedgerHQ/ledger-secure-sdk/blob/f7ba831fc72257d282060f9944644ef43b6b8e30/lib_nbgl/src/nbgl_use_case.c#L3874), [nbgl_useCaseReviewBlindSigning](https://github.com/LedgerHQ/ledger-secure-sdk/blob/f7ba831fc72257d282060f9944644ef43b6b8e30/lib_nbgl/src/nbgl_use_case.c#L3915) and [nbgl_useCaseReviewLight](https://github.com/LedgerHQ/ledger-secure-sdk/blob/f7ba831fc72257d282060f9944644ef43b6b8e30/lib_nbgl/src/nbgl_use_case.c#L4043) C API bindings.
//!
//! Used to display transaction review screens.
use super::*;

/// A builder to create and show a review flow.
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
    /// Creates a new review flow builder.
    /// # Returns
    /// Returns a new instance of `NbglReview`.
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

    /// Sets the type of transaction being reviewed.
    /// # Arguments
    /// * `tx_type` - The type of transaction being reviewed.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn tx_type(self, tx_type: TransactionType) -> NbglReview<'a> {
        NbglReview { tx_type, ..self }
    }

    /// Enables blind signing mode for the review.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn blind(self) -> NbglReview<'a> {
        NbglReview {
            blind: true,
            ..self
        }
    }

    /// Enables light mode for the review.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn light(self) -> NbglReview<'a> {
        NbglReview {
            light: true,
            ..self
        }
    }

    /// Sets the titles to display at the top and bottom of the page.
    /// # Arguments
    /// * `title` - The title to display at the top of the page.
    /// * `subtitle` - The subtitle to display below the title at the top of the page.
    /// * `finish_title` - The title to display at the bottom of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
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

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglReview<'a> {
        NbglReview {
            glyph: Some(glyph),
            ..self
        }
    }

    /// Shows the review flow with the provided fields on the review pages.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
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
