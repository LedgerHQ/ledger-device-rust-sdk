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

struct WarningDetailsType {
    dapp_provider_name: CString,
    report_url: CString,
    report_provider: CString,
    provider_message: CString,
}

/// A builder to create and show a streaming review flow.
pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
    skip: bool,
    warning_details_type: Option<WarningDetailsType>,
}

impl SyncNBGL for NbglStreamingReview {}

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
            warning_details_type: None,
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
        NbglStreamingReview {
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

    /// Starts the streaming review flow.
    /// # Arguments
    /// * `title` - The title to display at the top of the first page.
    /// * `subtitle` - An optional subtitle to display below the title on the first page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    pub fn start(&self, title: &str, subtitle: Option<&str>) -> bool {
        unsafe {
            let title = CString::new(title).unwrap();
            let subtitle = match subtitle {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };

            self.ux_sync_init();
            match self.blind {
                true => match &self.warning_details_type {
                    Some(w) => {
                        let warning_details = nbgl_warning_t {
                            predefinedSet: (1u32 << W3C_RISK_DETECTED_WARN),
                            dAppProvider: w.dapp_provider_name.as_ptr() as *const ::core::ffi::c_char,
                            reportUrl: w.report_url.as_ptr() as *const ::core::ffi::c_char,
                            reportProvider: w.report_provider.as_ptr() as *const ::core::ffi::c_char,
                            providerMessage: w.provider_message.as_ptr() as *const ::core::ffi::c_char,
                            ..Default::default()
                        };
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

    #[deprecated(note = "use next instead")]
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
                    item: field.name.as_ptr() as *const ::core::ffi::c_char,
                    value: field.value.as_ptr() as *const ::core::ffi::c_char,
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

    /// Proceeds to the next page in the streaming review flow with the provided fields.
    /// # Arguments
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    pub fn next(&self, fields: &[Field]) -> NbglStreamingReviewStatus {
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
                    item: field.name.as_ptr() as *const ::core::ffi::c_char,
                    value: field.value.as_ptr() as *const ::core::ffi::c_char,
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
            nbgl_useCaseReviewStreamingContinueExt(
                &tag_value_list as *const nbgl_contentTagValueList_t,
                Some(choice_callback),
                Some(skip_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                SyncNbgl::UxSyncRetApproved => {
                    return NbglStreamingReviewStatus::Next;
                }
                SyncNbgl::UxSyncRetSkipped => {
                    return NbglStreamingReviewStatus::Skipped;
                }
                _ => {
                    return NbglStreamingReviewStatus::Rejected;
                }
            }
        }
    }

    /// Finishes the streaming review flow by displaying the final confirmation page.
    /// # Arguments
    /// * `finish_title` - The title to display on the final confirmation page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
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
