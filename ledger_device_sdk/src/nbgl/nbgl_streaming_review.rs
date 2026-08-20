//! A wrapper around the asynchronous NBGL streaming review C API bindings
//! <ul>
//!     <li>nbgl_useCaseReviewStreamingStart</li>
//!     <li>nbgl_useCaseAdvancedReviewStreamingStart</li>
//!     <li>nbgl_useCaseReviewStreamingBlindSigningStart</li>
//!     <li>nbgl_useCaseReviewStreamingContinueExt</li>
//!     <li>nbgl_useCaseReviewStreamingFinish</li>
//! </ul>
//!
//! Used to display streamed transaction review screens.
use super::*;

/// A builder to create and show a streaming review flow.
pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
    skip: bool,
    /// Owns the C strings, icons and details tree behind the warning.
    warning: Option<CWarning>,
}

impl SyncNBGL for NbglStreamingReview {}

impl Default for NbglStreamingReview {
    fn default() -> Self {
        Self::new()
    }
}

/// Status returned by the `next` method.
pub enum NbglStreamingReviewStatus {
    Next,
    Rejected,
    Skipped,
}

impl NbglStreamingReview {
    /// Creates a new streaming review flow builder.
    /// # Returns
    /// Returns a new instance of `NbglStreamingReview`.
    pub fn new() -> NbglStreamingReview {
        NbglStreamingReview {
            icon: nbgl_icon_details_t::default(),
            tx_type: TransactionType::Transaction,
            blind: false,
            skip: false,
            warning: None,
        }
    }

    /// Sets the transaction type for the streaming review flow.
    /// # Arguments
    /// * `tx_type` - The transaction type to set.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn tx_type(self, tx_type: TransactionType) -> NbglStreamingReview {
        NbglStreamingReview { tx_type, ..self }
    }

    /// Enables blind signing mode for the streaming review flow.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn blind(self) -> NbglStreamingReview {
        NbglStreamingReview {
            blind: true,
            ..self
        }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &NbglGlyph) -> NbglStreamingReview {
        NbglStreamingReview {
            icon: glyph.into(),
            ..self
        }
    }

    /// Makes the review skippable, adding a "Skip" button to the UI.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn skippable(self) -> NbglStreamingReview {
        NbglStreamingReview { skip: true, ..self }
    }

    /// Configures the warning details to display in case of a risky transaction.
    ///
    /// This raises exactly one pre-defined warning,
    /// [`WarningType::W3cRiskDetected`]. Use [`Self::warning`] to raise any
    /// other combination, or to configure the pages manually.
    /// # Arguments
    /// * `dapp_provider` - The name of the dApp provider.
    /// * `report_url` - The URL where the user can report the issue.
    /// * `report_provider` - The name of the entity to which the issue can be reported.
    /// * `provider_message` - A message from the provider regarding the warning.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn warning_details(
        self,
        dapp_provider: Option<&str>,
        report_url: Option<&str>,
        report_provider: Option<&str>,
        provider_message: Option<&str>,
    ) -> NbglStreamingReview {
        self.warning(&build_legacy_warning(
            dapp_provider,
            report_url,
            report_provider,
            provider_message,
        ))
    }

    /// Sets the warning shown before the review, and reachable from the
    /// top-right button during it.
    ///
    /// Setting a warning selects `nbgl_useCaseAdvancedReviewStreamingStart`
    /// over the plain blind-signing start, so it takes effect only in
    /// combination with [`Self::blind`].
    /// # Arguments
    /// * `warning` - The warning configuration; see [`NbglWarning`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn warning(self, warning: &NbglWarning) -> NbglStreamingReview {
        NbglStreamingReview {
            warning: Some(CWarning::new(warning)),
            ..self
        }
    }

    fn start_internal(&self, title: &str, subtitle: Option<&str>) -> bool {
        unsafe {
            let title = CString::new(title).unwrap();
            let subtitle = match subtitle {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };

            self.ux_sync_init();
            match self.blind {
                true => match &self.warning {
                    Some(w) => {
                        let warning_details = w.as_c_type();
                        nbgl_useCaseAdvancedReviewStreamingStart(
                            self.tx_type.to_c_type(self.skip),
                            &self.icon as *const nbgl_icon_details_t,
                            title.as_ptr() as *const c_char,
                            match subtitle.is_empty() {
                                true => core::ptr::null(),
                                false => subtitle.as_ptr() as *const c_char,
                            },
                            &warning_details as *const nbgl_warning_t,
                            Some(choice_callback),
                        );
                    }
                    None => {
                        nbgl_useCaseReviewStreamingBlindSigningStart(
                            self.tx_type.to_c_type(self.skip),
                            &self.icon as *const nbgl_icon_details_t,
                            title.as_ptr() as *const c_char,
                            match subtitle.is_empty() {
                                true => core::ptr::null(),
                                false => subtitle.as_ptr() as *const c_char,
                            },
                            Some(choice_callback),
                        );
                    }
                },
                false => {
                    nbgl_useCaseReviewStreamingStart(
                        self.tx_type.to_c_type(self.skip),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        match subtitle.is_empty() {
                            true => core::ptr::null(),
                            false => subtitle.as_ptr() as *const c_char,
                        },
                        Some(choice_callback),
                    );
                }
            }
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            matches!(sync_ret, SyncNbgl::UxSyncRetApproved)
        }
    }

    fn continue_review_internal(&self, fields: &[Field]) -> bool {
        if nb_items(fields.len()).is_none() {
            return false;
        }
        unsafe {
            // Owns the C strings the pairs point at; must outlive the call below.
            let c_values = CTagValueList::from_fields(fields);
            let tag_value_list = c_values.as_c_list();

            self.ux_sync_init();
            nbgl_useCaseReviewStreamingContinue(
                &tag_value_list as *const nbgl_contentTagValueList_t,
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            matches!(sync_ret, SyncNbgl::UxSyncRetApproved)
        }
    }

    fn next_internal(&self, values: &[TagValue]) -> NbglStreamingReviewStatus {
        if nb_items(values.len()).is_none() {
            return NbglStreamingReviewStatus::Rejected;
        }
        unsafe {
            // Owns the C strings and the extension structs the pairs point at;
            // must outlive the use-case call below.
            let c_values = CTagValueList::new(values);
            let tag_value_list = c_values.as_c_list();

            self.ux_sync_init();
            nbgl_useCaseReviewStreamingContinueExt(
                &tag_value_list as *const nbgl_contentTagValueList_t,
                Some(choice_callback),
                Some(skip_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => NbglStreamingReviewStatus::Next,
                SyncNbgl::UxSyncRetSkipped => NbglStreamingReviewStatus::Skipped,
                _ => NbglStreamingReviewStatus::Rejected,
            }
        }
    }

    fn finish_internal(&self, finish_title: &str) -> bool {
        unsafe {
            let finish_title = CString::new(finish_title).unwrap();

            self.ux_sync_init();
            nbgl_useCaseReviewStreamingFinish(
                finish_title.as_ptr() as *const c_char,
                Some(choice_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            matches!(sync_ret, SyncNbgl::UxSyncRetApproved)
        }
    }

    /// Starts the streaming review flow.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `title` - The title to display at the top of the first page.
    /// * `subtitle` - An optional subtitle to display below the title on the first page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(feature = "io_new")]
    pub fn start<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        title: &str,
        subtitle: Option<&str>,
    ) -> bool {
        self.start_internal(title, subtitle)
    }

    /// Starts the streaming review flow.
    /// # Arguments
    /// * `title` - The title to display at the top of the first page.
    /// * `subtitle` - An optional subtitle to display below the title on the first page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(not(feature = "io_new"))]
    pub fn start(&self, title: &str, subtitle: Option<&str>) -> bool {
        self.start_internal(title, subtitle)
    }

    #[cfg(feature = "io_new")]
    #[deprecated(note = "use next instead")]
    pub fn continue_review<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        fields: &[Field],
    ) -> bool {
        self.continue_review_internal(fields)
    }

    #[cfg(not(feature = "io_new"))]
    #[deprecated(note = "use next instead")]
    pub fn continue_review(&self, fields: &[Field]) -> bool {
        self.continue_review_internal(fields)
    }

    /// Proceeds to the next page in the streaming review flow with the provided fields.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(feature = "io_new")]
    pub fn next<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        fields: &[Field],
    ) -> NbglStreamingReviewStatus {
        self.next_internal(&to_tag_values(fields))
    }

    /// Proceeds to the next page in the streaming review flow with tag/value
    /// pairs that may carry a [`FieldExtension`].
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `values` - A slice of `TagValue` representing the pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(feature = "io_new")]
    pub fn next_ext<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        values: &[TagValue],
    ) -> NbglStreamingReviewStatus {
        self.next_internal(values)
    }

    /// Proceeds to the next page in the streaming review flow with the provided fields.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(not(feature = "io_new"))]
    pub fn next(&self, fields: &[Field]) -> NbglStreamingReviewStatus {
        self.next_internal(&to_tag_values(fields))
    }

    /// Proceeds to the next page in the streaming review flow with tag/value
    /// pairs that may carry a [`FieldExtension`].
    /// # Arguments
    /// * `values` - A slice of `TagValue` representing the pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(not(feature = "io_new"))]
    pub fn next_ext(&self, values: &[TagValue]) -> NbglStreamingReviewStatus {
        self.next_internal(values)
    }

    /// Finishes the streaming review flow by displaying the final confirmation page.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    /// * `finish_title` - The title to display on the final confirmation page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(feature = "io_new")]
    pub fn finish<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        finish_title: &str,
    ) -> bool {
        self.finish_internal(finish_title)
    }

    /// Finishes the streaming review flow by displaying the final confirmation page.
    /// # Arguments
    /// * `finish_title` - The title to display on the final confirmation page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(not(feature = "io_new"))]
    pub fn finish(&self, finish_title: &str) -> bool {
        self.finish_internal(finish_title)
    }
}
