//! A wrapper around the asynchronous NBGL [nbgl_useCaseAdvancedReview](https://github.com/LedgerHQ/ledger-secure-sdk/blob/f7ba831fc72257d282060f9944644ef43b6b8e30/lib_nbgl/src/nbgl_use_case.c#L3957) C API binding.
//!
//! Draws a flow of pages of a review requiring if necessary a warning page before the review.
//! Moreover, the first and last pages of review display a top-right button, that displays more
//! information about the warnings
//!
//! Navigation operates with either swipe or navigation
//! keys at bottom right. The last page contains a long-press button with the given finishTitle and
//! the given icon.
//! All tag/value pairs are provided in the API and the number of pages is automatically
//! computed, the last page being a long press one
use super::*;

struct WarningDetailsType {
    dapp_provider_name: CString,
    report_url: CString,
    report_provider: CString,
    provider_message: CString,
}

/// A builder to create and show an advanced review flow.
pub struct NbglAdvanceReview<'a> {
    operation_type: TransactionType,
    glyph: Option<&'a NbglGlyph<'a>>,
    review_title: CString,
    review_subtitle: CString,
    finish_title: CString,
    warning_details_type: Option<WarningDetailsType>,
}

impl SyncNBGL for NbglAdvanceReview<'_> {}

impl<'a> NbglAdvanceReview<'a> {
    /// Creates a new advanced review flow builder.
    /// # Arguments
    /// * `operation_type` - The type of operation being reviewed.
    /// # Returns
    /// Returns a new instance of `NbglAdvanceReview`.
    pub fn new(operation_type: TransactionType) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            operation_type: operation_type,
            review_title: CString::default(),
            review_subtitle: CString::default(),
            finish_title: CString::default(),
            glyph: None,
            warning_details_type: None,
        }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            glyph: Some(glyph),
            ..self
        }
    }

    /// Sets the title to display at the top of the page.
    /// # Arguments
    /// * `review_title` - The title to display at the top of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn review_title(self, review_title: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            review_title: CString::new(review_title).unwrap(),
            ..self
        }
    }

    /// Sets the subtitle to display below the title at the top of the page.
    /// # Arguments
    /// * `review_subtitle` - The subtitle to display below the title at the top of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn review_subtitle(self, review_subtitle: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            review_subtitle: CString::new(review_subtitle).unwrap(),
            ..self
        }
    }

    /// Sets the title to display on the long-press button at the bottom of the last page.
    /// # Arguments
    /// * `finish_title` - The title to display on the long-press button.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn finish_title(self, finish_title: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            finish_title: CString::new(finish_title).unwrap(),
            ..self
        }
    }

    /// Sets the warning details to display when the user taps on the warning icon.
    /// All parameters are optional and can be set to `None` if not needed.
    /// # Arguments
    /// * `dapp_provider` - The name of the dApp provider.
    /// * `report_url` - The URL to report the issue.
    /// * `report_provider` - The name of the report provider.
    /// * `provider_message` - A message from the provider.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn warning_details(
        self,
        dapp_provider: Option<&str>,
        report_url: Option<&str>,
        report_provider: Option<&str>,
        provider_message: Option<&str>,
    ) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            warning_details_type: Some(WarningDetailsType {
                dapp_provider_name: match dapp_provider {
                    Some(s) => CString::new(s).unwrap(),
                    None => CString::default(),
                },
                report_url: match report_url {
                    Some(s) => CString::new(s).unwrap(),
                    None => CString::default(),
                },
                report_provider: match report_provider {
                    Some(s) => CString::new(s).unwrap(),
                    None => CString::default(),
                },
                provider_message: match provider_message {
                    Some(s) => CString::new(s).unwrap(),
                    None => CString::default(),
                },
            }),
            ..self
        }
    }

    /// Shows the advanced review flow.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns a `SyncNbgl` instance to manage the synchronous NBGL flow.
    pub fn show(&self, fields: &[Field]) -> SyncNbgl {
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

            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let warning_details = match &self.warning_details_type {
                Some(w) => nbgl_warning_t {
                    predefinedSet: (1u32 << W3C_RISK_DETECTED_WARN),
                    dAppProvider: w.dapp_provider_name.as_ptr() as *const i8,
                    reportUrl: w.report_url.as_ptr() as *const i8,
                    reportProvider: w.report_provider.as_ptr() as *const i8,
                    providerMessage: w.provider_message.as_ptr() as *const i8,
                    ..Default::default()
                },
                None => nbgl_warning_t::default(),
            };

            self.ux_sync_init();
            nbgl_useCaseAdvancedReview(
                self.operation_type.to_c_type(false),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                self.review_title.as_ptr() as *const c_char,
                self.review_subtitle.as_ptr() as *const c_char,
                self.finish_title.as_ptr() as *const c_char,
                core::ptr::null(),
                &warning_details as *const nbgl_warning_t,
                Some(choice_callback),
            );

            self.ux_sync_wait(false)
        }
    }
}
