//! A wrapper around the asynchronous NBGL [nbgl_useCaseChoice](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3584) C API binding.
//!
//! Draws a generic choice page, described in a centered info (with configurable icon),
//! thanks to a button and a footer at the bottom of the page.
use super::*;

const DEFAULT_CONFIRM_MESSAGE: &str = "Do you confirm this action?";
const DEFAULT_CONFIRM_SUBMESSAGE: &str = "This action is irreversible.";
const DEFAULT_CONFIRM_OK_TEXT: &str = "Confirm";
const DEFAULT_CONFIRM_KO_TEXT: &str = "Cancel";

const ACCEPT_IDX: usize = 0;
const REJECT_IDX: usize = 1;

struct ConfirmationStrings {
    message: CString,
    submessage: CString,
    ok_text: CString,
    ko_text: CString,
}

type Confirmations = [Option<ConfirmationStrings>; 2];

// Confirmations of the choice being shown, null when none is.
static mut G_CONFIRMATIONS: *const Confirmations = core::ptr::null();

/// Hands a choice's confirmations to its callback until dropped.
struct ActiveConfirmations;

impl ActiveConfirmations {
    fn set(confirmations: &Confirmations) -> Self {
        unsafe {
            G_CONFIRMATIONS = confirmations;
        }
        ActiveConfirmations
    }
}

impl Drop for ActiveConfirmations {
    fn drop(&mut self) {
        unsafe {
            G_CONFIRMATIONS = core::ptr::null();
        }
    }
}

unsafe extern "C" fn choice_confirm_callback(confirm: bool) {
    unsafe {
        let idx = if confirm { ACCEPT_IDX } else { REJECT_IDX };
        let Some(screen) = G_CONFIRMATIONS.as_ref().and_then(|c| c[idx].as_ref()) else {
            // No confirmation for this answer: it stands as is.
            choice_callback(confirm);
            return;
        };
        let callback: unsafe extern "C" fn() = if confirm {
            accept_confirmed_callback
        } else {
            reject_confirmed_callback
        };
        nbgl_useCaseConfirm(
            screen.message.as_ptr() as *const c_char,
            screen.submessage.as_ptr() as *const c_char,
            screen.ok_text.as_ptr() as *const c_char,
            screen.ko_text.as_ptr() as *const c_char,
            Some(callback),
        );
    }
}

unsafe extern "C" fn accept_confirmed_callback() {
    unsafe { choice_callback(true) }
}

unsafe extern "C" fn reject_confirmed_callback() {
    unsafe { choice_callback(false) }
}

/// A builder to create and show a choice flow.
pub struct NbglChoice<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    confirmations: Confirmations,
}

impl SyncNBGL for NbglChoice<'_> {}

impl<'a> Default for NbglChoice<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> NbglChoice<'a> {
    /// Creates a new choice flow builder.
    pub fn new() -> NbglChoice<'a> {
        NbglChoice {
            glyph: None,
            confirmations: [None, None],
        }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglChoice<'a> {
        NbglChoice {
            glyph: Some(glyph),
            ..self
        }
    }

    /// Configures the confirmation dialog to be shown when the user accepts or rejects the choice.
    ///
    /// The dialog belongs to this choice: it is shown only by this builder's `show*` methods.
    /// # Arguments
    /// * `message` - The main message to display in the confirmation dialog. If `None`, a default message is used.
    /// * `submessage` - An optional sub-message to display below the main message. If `None`, a default sub-message is used.
    /// * `ok_text` - The text to display on the confirmation button. If `None`, a default text is used.
    /// * `ko_text` - The text to display on the cancellation button. If `None`, a default text is used.
    /// * `if_accept` - The `if_accept` parameter determines whether the dialog is shown when the user accepts (`true`)
    ///   or rejects (`false`) the choice.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn ask_confirmation(
        mut self,
        message: Option<&str>,
        submessage: Option<&str>,
        ok_text: Option<&str>,
        ko_text: Option<&str>,
        if_accept: bool,
    ) -> NbglChoice<'a> {
        let screen = ConfirmationStrings {
            message: CString::new(message.unwrap_or(DEFAULT_CONFIRM_MESSAGE)).unwrap(),
            submessage: CString::new(submessage.unwrap_or(DEFAULT_CONFIRM_SUBMESSAGE)).unwrap(),
            ok_text: CString::new(ok_text.unwrap_or(DEFAULT_CONFIRM_OK_TEXT)).unwrap(),
            ko_text: CString::new(ko_text.unwrap_or(DEFAULT_CONFIRM_KO_TEXT)).unwrap(),
        };
        let idx = if if_accept { ACCEPT_IDX } else { REJECT_IDX };
        self.confirmations[idx] = Some(screen);
        self
    }

    fn show_internal(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
    ) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            let message = CString::new(message).unwrap();
            let sub_message = CString::new(sub_message).unwrap();
            let confirm_text = CString::new(confirm_text).unwrap();
            let cancel_text = CString::new(cancel_text).unwrap();

            let _confirmations = ActiveConfirmations::set(&self.confirmations);
            self.ux_sync_init();
            nbgl_useCaseChoice(
                &icon as *const nbgl_icon_details_t,
                match message.is_empty() {
                    true => core::ptr::null(),
                    false => message.as_ptr() as *const c_char,
                },
                match sub_message.is_empty() {
                    true => core::ptr::null(),
                    false => sub_message.as_ptr() as *const c_char,
                },
                match confirm_text.is_empty() {
                    true => core::ptr::null(),
                    false => confirm_text.as_ptr() as *const c_char,
                },
                match cancel_text.is_empty() {
                    true => core::ptr::null(),
                    false => cancel_text.as_ptr() as *const c_char,
                },
                Some(choice_confirm_callback),
            );
            let sync_ret = self.ux_sync_wait(false);

            // Return true if the user approved the transaction, false otherwise.
            matches!(sync_ret, SyncNbgl::UxSyncRetApproved)
        }
    }

    /// Shows the choice flow.
    /// # Arguments
    /// * `comm` - Mutable reference to Comm.
    /// * `message` - The main message to display in the center of the page.
    /// * `sub_message` - An optional sub-message to display below the main message.
    /// * `confirm_text` - The text to display on the confirmation button.
    /// * `cancel_text` - The text to display on the cancellation button.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
    ) -> bool {
        comm.lend_to_nbgl(|| self.show_internal(message, sub_message, confirm_text, cancel_text))
    }

    /// Shows the choice flow.
    /// # Arguments
    /// * `message` - The main message to display in the center of the page.
    /// * `sub_message` - An optional sub-message to display below the main message.
    /// * `confirm_text` - The text to display on the confirmation button.
    /// * `cancel_text` - The text to display on the cancellation button.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(not(feature = "io_new"))]
    pub fn show(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
    ) -> bool {
        self.show_internal(message, sub_message, confirm_text, cancel_text)
    }

    /// Shows the choice flow with a details page reachable from the screen.
    ///
    /// Same as [`Self::show`], plus a `details` page the user can open to read
    /// more before choosing. `details` is the same [`WarningDetails`] used by
    /// [`NbglWarning`].
    /// # Arguments
    /// * `comm` - Mutable reference to Comm.
    /// * `message` - The main message to display in the center of the page.
    /// * `sub_message` - An optional sub-message to display below the main message.
    /// * `confirm_text` - The text to display on the confirmation button.
    /// * `cancel_text` - The text to display on the cancellation button.
    /// * `details` - The page opened to read more.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(feature = "io_new")]
    #[allow(clippy::too_many_arguments)]
    pub fn show_with_details<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        comm.lend_to_nbgl(|| {
            self.show_with_details_internal(
                message,
                sub_message,
                confirm_text,
                cancel_text,
                details,
            )
        })
    }

    /// Shows the choice flow with a details page reachable from the screen.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(not(feature = "io_new"))]
    pub fn show_with_details(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        self.show_with_details_internal(message, sub_message, confirm_text, cancel_text, details)
    }

    fn show_with_details_internal(
        &self,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        if !details.fits() {
            return false;
        }
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            let message = CString::new(message).unwrap();
            let sub_message = CString::new(sub_message).unwrap();
            let confirm_text = CString::new(confirm_text).unwrap();
            let cancel_text = CString::new(cancel_text).unwrap();

            // Owns the strings and any sub-tree the details page points at.
            let c_details = CDetails::new(details);
            // C takes this by mutable pointer, so it needs its own storage.
            let mut details_c = c_details.as_c_type();

            let _confirmations = ActiveConfirmations::set(&self.confirmations);
            self.ux_sync_init();
            nbgl_useCaseChoiceWithDetails(
                &icon as *const nbgl_icon_details_t,
                opt_str_ptr(&message),
                opt_str_ptr(&sub_message),
                opt_str_ptr(&confirm_text),
                opt_str_ptr(&cancel_text),
                &mut details_c as *mut nbgl_warningDetails_t,
                Some(choice_confirm_callback),
            );
            matches!(self.ux_sync_wait(false), SyncNbgl::UxSyncRetApproved)
        }
    }

    /// Shows the choice flow with a header and a details page.
    ///
    /// Wraps `nbgl_useCaseAdvancedChoiceWithDetails`, which adds a header icon
    /// and title above the centered message.
    /// # Arguments
    /// * `comm` - Mutable reference to Comm.
    /// * `header_glyph` - Icon drawn in the header, if any.
    /// * `title` - Title drawn in the header.
    /// * `message` - The main message to display in the center of the page.
    /// * `sub_message` - An optional sub-message to display below the main message.
    /// * `confirm_text` - The text to display on the confirmation button.
    /// * `cancel_text` - The text to display on the cancellation button.
    /// * `details` - The page opened to read more.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(feature = "io_new")]
    #[allow(clippy::too_many_arguments)]
    pub fn show_advanced_with_details<const N: usize>(
        &self,
        comm: &mut crate::io::Comm<N>,
        header_glyph: Option<&NbglGlyph>,
        title: &str,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        comm.lend_to_nbgl(|| {
            self.show_advanced_internal(
                header_glyph,
                title,
                message,
                sub_message,
                confirm_text,
                cancel_text,
                details,
            )
        })
    }

    /// Shows the choice flow with a header and a details page.
    /// # Returns
    /// Returns `true` if the user confirmed the choice, `false` otherwise.
    #[cfg(not(feature = "io_new"))]
    #[allow(clippy::too_many_arguments)]
    pub fn show_advanced_with_details(
        &self,
        header_glyph: Option<&NbglGlyph>,
        title: &str,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        self.show_advanced_internal(
            header_glyph,
            title,
            message,
            sub_message,
            confirm_text,
            cancel_text,
            details,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn show_advanced_internal(
        &self,
        header_glyph: Option<&NbglGlyph>,
        title: &str,
        message: &str,
        sub_message: &str,
        confirm_text: &str,
        cancel_text: &str,
        details: &WarningDetails,
    ) -> bool {
        if !details.fits() {
            return false;
        }
        unsafe {
            let center_icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };
            let header_icon: Option<nbgl_icon_details_t> = header_glyph.map(|g| g.into());
            let title = CString::new(title).unwrap();
            let message = CString::new(message).unwrap();
            let sub_message = CString::new(sub_message).unwrap();
            let confirm_text = CString::new(confirm_text).unwrap();
            let cancel_text = CString::new(cancel_text).unwrap();

            let c_details = CDetails::new(details);
            let mut details_c = c_details.as_c_type();

            let _confirmations = ActiveConfirmations::set(&self.confirmations);
            self.ux_sync_init();
            nbgl_useCaseAdvancedChoiceWithDetails(
                &center_icon as *const nbgl_icon_details_t,
                match &header_icon {
                    Some(icon) => icon as *const nbgl_icon_details_t,
                    None => core::ptr::null(),
                },
                opt_str_ptr(&title),
                opt_str_ptr(&message),
                opt_str_ptr(&sub_message),
                opt_str_ptr(&confirm_text),
                opt_str_ptr(&cancel_text),
                &mut details_c as *mut nbgl_warningDetails_t,
                Some(choice_confirm_callback),
            );
            matches!(self.ux_sync_wait(false), SyncNbgl::UxSyncRetApproved)
        }
    }
}

/// NULL for an empty string, matching how the other wrappers treat "unset".
fn opt_str_ptr(s: &CString) -> *const c_char {
    match s.is_empty() {
        true => core::ptr::null(),
        false => s.as_ptr() as *const c_char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    // Runs `callback` as NBGL would and returns what the flow ends with.
    fn answer(callback: unsafe extern "C" fn(bool), confirm: bool) -> (u8, bool) {
        unsafe {
            G_RET = SyncNbgl::UxSyncRetError.into();
            G_ENDED = false;
            callback(confirm);
            (G_RET, G_ENDED)
        }
    }

    const APPROVED: (u8, bool) = (0x00, true);
    const REJECTED: (u8, bool) = (0x01, true);

    fn both_confirmations() -> NbglChoice<'static> {
        NbglChoice::new()
            .ask_confirmation(Some("Accept?"), None, None, None, true)
            .ask_confirmation(Some("Reject?"), None, None, None, false)
    }

    #[test]
    fn review_answers_ignore_choice_confirmations() {
        let choice = both_confirmations();
        assert_eq!(answer(choice_callback, true), APPROVED);
        assert_eq!(answer(choice_callback, false), REJECTED);

        let _confirmations = ActiveConfirmations::set(&choice.confirmations);
        assert_eq!(answer(choice_callback, true), APPROVED);
        assert_eq!(answer(choice_callback, false), REJECTED);
    }

    #[test]
    fn answer_without_confirmation_stands() {
        let accept_only = NbglChoice::new().ask_confirmation(None, None, None, None, true);
        let _confirmations = ActiveConfirmations::set(&accept_only.confirmations);
        assert_eq!(answer(choice_confirm_callback, false), REJECTED);

        let reject_only = NbglChoice::new().ask_confirmation(None, None, None, None, false);
        let _confirmations = ActiveConfirmations::set(&reject_only.confirmations);
        assert_eq!(answer(choice_confirm_callback, true), APPROVED);
    }

    #[test]
    fn confirmations_are_cleared_after_show() {
        let choice = both_confirmations();
        drop(ActiveConfirmations::set(&choice.confirmations));
        assert_eq!(unsafe { G_CONFIRMATIONS.is_null() }, true);
        assert_eq!(answer(choice_confirm_callback, true), APPROVED);
        assert_eq!(answer(choice_confirm_callback, false), REJECTED);
    }

    #[test]
    fn details_with_too_many_bars_are_refused() {
        const BAR: WarningBar = WarningBar {
            text: "Risk",
            sub_text: None,
            icon: None,
            details: None,
        };
        static BARS: [WarningBar; 256] = [BAR; 256];
        let details = WarningDetails::BarList {
            title: "Risks",
            bars: &BARS,
        };
        let choice = NbglChoice::new();
        let shown = choice.show_with_details_internal("Sign?", "", "Yes", "No", &details);
        assert_eq!(shown, false);
        let shown = choice.show_advanced_internal(None, "Sign", "Sign?", "", "Yes", "No", &details);
        assert_eq!(shown, false);
    }

    #[test]
    fn glyph_keeps_confirmations() {
        static GLYPH: NbglGlyph =
            NbglGlyph::from_include(include_gif::include_gif!("icons/badge_check.gif", NBGL));
        let choice = both_confirmations().glyph(&GLYPH);
        assert_eq!(choice.confirmations.iter().all(Option::is_some), true);
    }
}
