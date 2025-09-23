use super::*;

/// A wrapper around the asynchronous NBGL nbgl_useCaseReviewStreamingStart/Continue/Finish)
/// C API binding. Used to display streamed transaction review screens.

struct WarningDetailsType {
    dapp_provider_name: CString,
    report_url: CString,
    report_provider: CString,
    provider_message: CString,
}

pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
    skip: bool,
    warning_details_type: Option<WarningDetailsType>,
}

impl SyncNBGL for NbglStreamingReview {}

pub enum NbglStreamingReviewStatus {
    Next,
    Rejected,
    Skipped,
}

impl NbglStreamingReview {
    pub fn new() -> NbglStreamingReview {
        NbglStreamingReview {
            icon: nbgl_icon_details_t::default(),
            tx_type: TransactionType::Transaction,
            blind: false,
            skip: false,
            warning_details_type: None,
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

    pub fn skippable(self) -> NbglStreamingReview {
        NbglStreamingReview { skip: true, ..self }
    }

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
                            dAppProvider: w.dapp_provider_name.as_ptr() as *const i8,
                            reportUrl: w.report_url.as_ptr() as *const i8,
                            reportProvider: w.report_provider.as_ptr() as *const i8,
                            providerMessage: w.provider_message.as_ptr() as *const i8,
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
