use super::*;

struct WarningDetailsType {
    dapp_provider_name: CString,
    report_url: CString,
    report_provider: CString,
    provider_message: CString,
}

/// A wrapper around the asynchronous NBGL nbgl_useCaseAdvanceReview C API binding.
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

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn review_title(self, review_title: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            review_title: CString::new(review_title).unwrap(),
            ..self
        }
    }

    pub fn review_subtitle(self, review_subtitle: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            review_subtitle: CString::new(review_subtitle).unwrap(),
            ..self
        }
    }

    pub fn finish_title(self, finish_title: &str) -> NbglAdvanceReview<'a> {
        NbglAdvanceReview {
            finish_title: CString::new(finish_title).unwrap(),
            ..self
        }
    }

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
