//! A wrapper around the asynchronous NBGL [nbgl_useCaseAdvancedReview](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L4036) C API binding.
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

/// A builder to create and show an advanced review flow.
pub struct NbglAdvanceReview<'a> {
    operation_type: TransactionType,
    glyph: Option<&'a NbglGlyph<'a>>,
    review_title: CString,
    review_subtitle: CString,
    finish_title: CString,
    /// Owns the C strings, icons and details tree behind the warning.
    warning: Option<CWarning>,
    /// Owns the C strings and info list behind the tip box.
    tip_box: Option<CTipBox>,
    /// Extra `nbgl_operationType_t` bits set alongside the operation type.
    operation_flags: nbgl_operationType_t,
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
            operation_type,
            review_title: CString::default(),
            review_subtitle: CString::default(),
            finish_title: CString::default(),
            glyph: None,
            warning: None,
            tip_box: None,
            operation_flags: 0,
        }
    }

    /// Sets the flags accompanying the operation type.
    ///
    /// `Blind`, `Risky` and `NoThreat` are what make NBGL draw the warning
    /// button in the top-right of the first and last review pages. Setting any
    /// of them requires a warning that can fill that button — one with
    /// [`NbglWarning::predefined`] or [`NbglWarning::review_details`] — since
    /// NBGL reads it without checking for NULL. [`Self::show`] panics rather
    /// than let that reach C.
    ///
    /// The button appears on the last page only for a review whose final page
    /// is a long-press one.
    /// # Arguments
    /// * `flags` - The flags to set; see [`OperationFlag`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn operation_flags(self, flags: &[OperationFlag]) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            operation_flags: OperationFlag::mask(flags),
            ..self
        }
    }

    /// Sets the tip box shown on the review's first page.
    ///
    /// Only takes effect on a review whose warning set raises no tip box of its
    /// own. Any of the `W3c*` warnings, or `BlindSigning`, makes NBGL draw its
    /// own tip box instead and route the touch to the security report, ignoring
    /// this one entirely — both its text and its info list.
    ///
    /// This is why there is no equivalent on `NbglReview`: the only use case it
    /// wraps that accepts a tip box is the blind-signing one, which always
    /// raises `BlindSigning`.
    /// # Arguments
    /// * `tip_box` - The tip box; see [`TipBox`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn tip_box(self, tip_box: &TipBox) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            tip_box: Some(CTipBox::new(tip_box)),
            ..self
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
    ///
    /// This raises exactly one pre-defined warning,
    /// [`WarningType::W3cRiskDetected`]. Use [`Self::warning`] to raise any
    /// other combination, or to configure the pages manually.
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
        self.warning(&build_legacy_warning(
            dapp_provider,
            report_url,
            report_provider,
            provider_message,
        ))
    }

    /// Sets the warning shown before the review, and reachable from the
    /// top-right button during it.
    /// # Arguments
    /// * `warning` - The warning configuration; see [`NbglWarning`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn warning(self, warning: &NbglWarning) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            warning: Some(CWarning::new(warning)),
            ..self
        }
    }

    fn show_internal(&self, values: &[TagValue]) -> SyncNbgl {
        unsafe {
            // Owns the C strings and the extension structs the pairs point at;
            // must outlive the use-case call below.
            let c_values = CTagValueList::new(values);
            let tag_value_list = c_values.as_c_list();

            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            check_report_flags(
                self.operation_flags,
                self.warning
                    .as_ref()
                    .is_some_and(|w| w.raises_review_report()),
            );

            let warning_details = match &self.warning {
                Some(w) => w.as_c_type(),
                None => nbgl_warning_t::default(),
            };

            // Materialised here so the pointer passed below outlives the call.
            let tip_box = self.tip_box.as_ref().map(|t| t.as_c_type());

            self.ux_sync_init();
            nbgl_useCaseAdvancedReview(
                self.operation_type.to_c_type(self.operation_flags),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                self.review_title.as_ptr() as *const c_char,
                self.review_subtitle.as_ptr() as *const c_char,
                self.finish_title.as_ptr() as *const c_char,
                match &tip_box {
                    Some(tip_box) => tip_box as *const nbgl_tipBox_t,
                    None => core::ptr::null(),
                },
                &warning_details as *const nbgl_warning_t,
                Some(choice_callback),
            );

            self.ux_sync_wait(false)
        }
    }

    /// Shows the advanced review flow.
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
        self.show_ext(_comm, &to_tag_values(fields))
    }

    /// Shows the advanced review flow.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display.
    /// # Returns
    /// Returns a `SyncNbgl` instance to manage the synchronous NBGL flow.
    #[cfg(not(feature = "io_new"))]
    pub fn show(&self, fields: &[Field]) -> SyncNbgl {
        self.show_internal(&to_tag_values(fields))
    }

    /// Shows the advanced review flow with tag/value pairs that may carry a
    /// [`FieldExtension`].
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `values` - A slice of `TagValue` representing the pairs to display.
    /// # Returns
    /// Returns `Ok(true)` if the user accepts the review,
    /// `Ok(false)` if the user rejects it,
    /// or `Err(u8)` with the error code in case of an error.
    #[cfg(feature = "io_new")]
    pub fn show_ext<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        values: &[TagValue],
    ) -> Result<bool, u8> {
        let ret = self.show_internal(values);
        match ret {
            SyncNbgl::UxSyncRetApproved => Ok(true),
            SyncNbgl::UxSyncRetRejected => Ok(false),
            _ => Err(u8::from(ret)),
        }
    }

    /// Shows the advanced review flow with tag/value pairs that may carry a
    /// [`FieldExtension`].
    /// # Arguments
    /// * `values` - A slice of `TagValue` representing the pairs to display.
    /// # Returns
    /// Returns a `SyncNbgl` instance to manage the synchronous NBGL flow.
    #[cfg(not(feature = "io_new"))]
    pub fn show_ext(&self, values: &[TagValue]) -> SyncNbgl {
        self.show_internal(values)
    }
}
