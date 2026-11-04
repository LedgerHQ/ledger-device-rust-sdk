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
use alloc::boxed::Box;

/// A builder to create and show a streaming review flow.
pub struct NbglStreamingReview {
    icon: nbgl_icon_details_t,
    tx_type: TransactionType,
    blind: bool,
    skip: bool,
    /// Owns the C strings, icons and details tree behind the warning.
    warning: Option<CWarning>,
    /// The warning handed to `nbgl_useCaseAdvancedReviewStreamingStart`, built
    /// from `warning` and `blind`. NBGL keeps this pointer until the review
    /// ends, so it is boxed to stay at the same address for as long as the
    /// builder lives, even if the builder moves.
    c_warning: Option<Box<nbgl_warning_t>>,
    /// Extra `nbgl_operationType_t` bits set alongside the transaction type.
    operation_flags: nbgl_operationType_t,
    /// App handler for a touched value icon.
    on_value_icon: Option<fn(u8)>,
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
            c_warning: None,
            operation_flags: 0,
            on_value_icon: None,
        }
    }

    /// Sets the function run when a value icon is touched.
    ///
    /// It receives the index of the pair whose icon was touched. Without it the
    /// icons are drawn but report nothing.
    ///
    /// Only meaningful for pairs carrying [`TagValue::value_icon`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn on_value_icon(self, on_value_icon: fn(u8)) -> NbglStreamingReview {
        NbglStreamingReview {
            on_value_icon: Some(on_value_icon),
            ..self
        }
    }

    /// Sets the flags accompanying the transaction type.
    ///
    /// `Blind`, `Risky` and `NoThreat` are what make NBGL draw the warning
    /// button in the top-right of the first and last review pages. Setting any
    /// of them requires a warning that can fill that button — see
    /// [`Self::warning`] — since NBGL reads it without checking for NULL.
    /// [`Self::skip`] already sets `Skippable`, so it need not be repeated here.
    /// # Arguments
    /// * `flags` - The flags to set; see [`OperationFlag`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn operation_flags(self, flags: &[OperationFlag]) -> NbglStreamingReview {
        NbglStreamingReview {
            operation_flags: OperationFlag::mask(flags),
            ..self
        }
    }

    /// The operation type handed to C: the transaction type, the app's flags,
    /// and `Skippable` when `skip` was set.
    ///
    /// # Panics
    /// Panics if a flag that raises the review's warning button is set without
    /// a warning able to fill it, which NBGL would dereference as NULL.
    fn c_operation_type(&self) -> nbgl_operationType_t {
        check_report_flags(
            self.operation_flags,
            self.warning
                .as_ref()
                .is_some_and(|w| w.raises_review_report()),
        );
        let mut flags = self.operation_flags;
        if self.skip {
            flags |= OperationFlag::Skippable.bit();
        }
        self.tx_type.to_c_type(flags)
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
        .with_c_warning()
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &NbglGlyph<'static>) -> NbglStreamingReview {
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
    /// Setting a warning selects `nbgl_useCaseAdvancedReviewStreamingStart`,
    /// with or without [`Self::blind`]. In blind mode the blind-signing
    /// warning is added to the ones configured here.
    /// # Arguments
    /// * `warning` - The warning configuration; see [`NbglWarning`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn warning(self, warning: &NbglWarning) -> NbglStreamingReview {
        NbglStreamingReview {
            warning: Some(CWarning::new(warning)),
            ..self
        }
        .with_c_warning()
    }

    /// Rebuilds the warning handed to `nbgl_useCaseAdvancedReviewStreamingStart`,
    /// if one is set; blind mode adds the blind-signing warning to it.
    fn with_c_warning(self) -> NbglStreamingReview {
        let c_warning =
            with_blind_signing(self.warning.as_ref().map(CWarning::as_c_type), self.blind)
                .map(Box::new);
        NbglStreamingReview { c_warning, ..self }
    }

    fn start_internal(&self, title: &str, subtitle: Option<&str>) -> bool {
        if !self.warning.as_ref().is_none_or(CWarning::fits) {
            return false;
        }
        unsafe {
            let title = CString::new(title).unwrap();
            let subtitle = match subtitle {
                Some(s) => CString::new(s).unwrap(),
                None => CString::default(),
            };
            let subtitle_ptr = match subtitle.is_empty() {
                true => core::ptr::null(),
                false => subtitle.as_ptr() as *const c_char,
            };

            self.ux_sync_init();
            match self.c_warning.as_deref() {
                Some(warning) => {
                    nbgl_useCaseAdvancedReviewStreamingStart(
                        self.c_operation_type(),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        subtitle_ptr,
                        warning as *const nbgl_warning_t,
                        Some(choice_callback),
                    );
                }
                None if self.blind => {
                    nbgl_useCaseReviewStreamingBlindSigningStart(
                        self.c_operation_type(),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        subtitle_ptr,
                        Some(choice_callback),
                    );
                }
                None => {
                    nbgl_useCaseReviewStreamingStart(
                        self.c_operation_type(),
                        &self.icon as *const nbgl_icon_details_t,
                        title.as_ptr() as *const c_char,
                        subtitle_ptr,
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
        if !tag_values_fit(values) {
            return NbglStreamingReviewStatus::Rejected;
        }
        unsafe {
            // Owns the C strings and the extension structs the pairs point at;
            // must outlive the use-case call below.
            set_value_icon_handler(self.on_value_icon);
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
    /// * `comm` - Mutable reference to Comm.
    /// * `title` - The title to display at the top of the first page.
    /// * `subtitle` - An optional subtitle to display below the title on the first page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(feature = "io_new")]
    pub fn start<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        title: &str,
        subtitle: Option<&str>,
    ) -> bool {
        comm.lend_to_nbgl(|| self.start_internal(title, subtitle))
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
        comm: &mut crate::io::Comm<N>,
        fields: &[Field],
    ) -> bool {
        comm.lend_to_nbgl(|| self.continue_review_internal(fields))
    }

    #[cfg(not(feature = "io_new"))]
    #[deprecated(note = "use next instead")]
    pub fn continue_review(&self, fields: &[Field]) -> bool {
        self.continue_review_internal(fields)
    }

    /// Proceeds to the next page in the streaming review flow with the provided fields.
    /// # Arguments
    /// * `comm` - Mutable reference to Comm.
    /// * `fields` - A slice of `Field` representing the tag/value pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(feature = "io_new")]
    pub fn next<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        fields: &[Field],
    ) -> NbglStreamingReviewStatus {
        comm.lend_to_nbgl(|| self.next_internal(&to_tag_values(fields)))
    }

    /// Proceeds to the next page in the streaming review flow with tag/value
    /// pairs that may carry a [`FieldExtension`].
    /// # Arguments
    /// * `comm` - Mutable reference to Comm.
    /// * `values` - A slice of `TagValue` representing the pairs to display on the next page.
    /// # Returns
    /// Returns an `NbglStreamingReviewStatus` indicating whether the user proceeded to the next
    /// page, skipped the review, or rejected it.
    #[cfg(feature = "io_new")]
    pub fn next_ext<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        values: &[TagValue],
    ) -> NbglStreamingReviewStatus {
        comm.lend_to_nbgl(|| self.next_internal(values))
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
    /// * `comm` - Mutable reference to Comm.
    /// * `finish_title` - The title to display on the final confirmation page.
    /// # Returns
    /// Returns `true` if the user approved the transaction, `false` otherwise.
    #[cfg(feature = "io_new")]
    pub fn finish<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        finish_title: &str,
    ) -> bool {
        comm.lend_to_nbgl(|| self.finish_internal(finish_title))
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

/// `warning` with the blind-signing warning added in blind mode.
fn with_blind_signing(warning: Option<nbgl_warning_t>, blind: bool) -> Option<nbgl_warning_t> {
    warning.map(|mut warning| {
        if blind {
            warning.predefinedSet |= WarningType::BlindSigning.bit();
        }
        warning
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const RISK: u32 = 1 << W3C_RISK_DETECTED_WARN;
    const BLIND: u32 = 1 << BLIND_SIGNING_WARN;
    const DAPP: &core::ffi::CStr = c"dApp";

    // Built by hand: an `NbglStreamingReview` or `NbglWarning` would link the
    // whole warning builder into the test binary, which does not fit in flash
    // on every target.
    fn risk_warning() -> nbgl_warning_t {
        nbgl_warning_t {
            predefinedSet: RISK,
            dAppProvider: DAPP.as_ptr(),
            ..Default::default()
        }
    }

    // No warning: the plain or blind-signing start, chosen by `blind`.
    #[test]
    fn no_warning_uses_plain_or_blind_start() {
        assert_eq!(with_blind_signing(None, false).is_none(), true);
        assert_eq!(with_blind_signing(None, true).is_none(), true);
    }

    #[test]
    fn warning_is_shown_outside_blind_mode() {
        let warning = with_blind_signing(Some(risk_warning()), false);
        assert_eq!(warning.map(|w| w.predefinedSet), Some(RISK));
        assert_eq!(
            warning.is_some_and(|w| w.dAppProvider == DAPP.as_ptr()),
            true
        );
    }

    // NBGL keeps the start warning's address for the whole review: it must not
    // change when the builder moves between `start` and `next`/`finish`.
    #[test]
    fn start_warning_keeps_its_address_when_the_review_moves() {
        let review = NbglStreamingReview::new().blind();
        // Blind mode without a warning uses the plain blind-signing start.
        assert_eq!(review.c_warning.is_none(), true);

        let review = NbglStreamingReview {
            c_warning: with_blind_signing(Some(risk_warning()), review.blind).map(Box::new),
            ..review
        };
        let before = review
            .c_warning
            .as_deref()
            .map(|w| w as *const nbgl_warning_t);
        let moved = [review];
        let after = moved[0]
            .c_warning
            .as_deref()
            .map(|w| w as *const nbgl_warning_t);
        assert_eq!(before.is_some() && before == after, true);
        assert_eq!(
            moved[0].c_warning.as_deref().map(|w| w.predefinedSet),
            Some(RISK | BLIND)
        );
    }

    #[test]
    fn blind_mode_adds_blind_signing_warning() {
        let warning = with_blind_signing(Some(risk_warning()), true);
        assert_eq!(warning.map(|w| w.predefinedSet), Some(RISK | BLIND));
        assert_eq!(
            warning.is_some_and(|w| w.dAppProvider == DAPP.as_ptr()),
            true
        );
    }
}
