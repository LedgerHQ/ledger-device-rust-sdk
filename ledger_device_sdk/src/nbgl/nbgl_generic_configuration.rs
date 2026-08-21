//! A wrapper around the NBGL [nbgl_useCaseGenericConfiguration](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c) C API binding.
//!
//! Draws a configuration screen from arbitrary content, paginated
//! automatically, with a header to leave. It is the general form of
//! [`NbglGenericSettings`], which is fixed to one switch list built from NVM,
//! and of `nbgl_useCaseGenericSettings` minus the app information page.
//!
//! Pages are described with the same [`NbglPageContent`] items as
//! [`NbglGenericReview`]:
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::{ChoicesList, NbglGenericConfiguration, NbglPageContent, SwitchesList};
//! let mut config = NbglGenericConfiguration::new()
//!     .title("Configuration")
//!     .add_content(NbglPageContent::SwitchesList(SwitchesList::new(&[
//!         ("Expert mode", "Show raw values", false),
//!     ])))
//!     .add_content(NbglPageContent::ChoicesList(ChoicesList::new(
//!         &["Slow", "Fast"],
//!         0,
//!     )));
//! config.show();
//! ```
//!
//! Unlike a review there is nothing to approve: the flow ends when the user
//! leaves through the header.
//!
//! Control touches reach the app through [`NbglGenericConfiguration::on_action`];
//! without it the interactive lists are drawn but report nothing. Persisting a
//! switch is still the app's job — [`NbglGenericSettings`] does that for the
//! simple NVM-backed case.

use super::*;

/// A builder for a configuration screen built from generic content.
pub struct NbglGenericConfiguration {
    title: CString,
    init_page: u8,
    content_list: Vec<NbglPageContent>,
    on_action: Option<fn(u8, u8)>,
}

impl SyncNBGL for NbglGenericConfiguration {}

impl Default for NbglGenericConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

impl NbglGenericConfiguration {
    /// Creates an empty configuration screen with no content.
    pub fn new() -> NbglGenericConfiguration {
        NbglGenericConfiguration {
            title: CString::default(),
            init_page: 0,
            content_list: Vec::new(),
            on_action: None,
        }
    }

    /// Sets the function run when a control inside one of the contents is
    /// touched — a switch, a choice, or a bar.
    ///
    /// It receives the token of the touched element and, for a choices list,
    /// the index chosen. Each content type assigns `FIRST_USER_TOKEN + i` to
    /// its `i`th element, except [`ChoicesList`], which uses one token and
    /// reports the selection in `index`.
    ///
    /// Without this the interactive lists are drawn but report nothing back.
    /// Note that this reports touches only: persisting a switch to NVM is the
    /// app's job, and [`NbglGenericSettings`] does it for the simple case.
    pub fn on_action(mut self, on_action: fn(u8, u8)) -> NbglGenericConfiguration {
        self.on_action = Some(on_action);
        self
    }

    /// Sets the text of the header.
    pub fn title(self, title: &str) -> NbglGenericConfiguration {
        NbglGenericConfiguration {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    /// Sets the page the screen opens on, counting from 0.
    pub fn init_page(self, init_page: u8) -> NbglGenericConfiguration {
        NbglGenericConfiguration { init_page, ..self }
    }

    /// Appends a content element.
    ///
    /// Elements are paginated by NBGL, so one element may span several pages
    /// depending on how many items it holds.
    pub fn add_content(mut self, content: NbglPageContent) -> NbglGenericConfiguration {
        self.content_list.push(content);
        self
    }

    /// Shows the configuration screen, returning when the user leaves through
    /// the header.
    ///
    /// # Panics
    /// Panics if no content was added, since NBGL would have nothing to draw.
    pub fn show(&mut self) {
        if self.content_list.is_empty() {
            panic!("No content added.");
        }

        unsafe {
            set_content_action(self.on_action);
            let on_action: nbgl_contentActionCallback_t = match self.on_action {
                Some(_) => Some(content_action_callback),
                None => None,
            };

            // Owns the C structs for as long as the call below borrows them.
            let c_content_list: Vec<nbgl_content_t> = self
                .content_list
                .iter()
                .map(|c| c.to_c_content(on_action))
                .collect();

            let contents = nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: c_content_list.as_ptr(),
                },
                nbContents: c_content_list.len() as u8,
            };

            self.ux_sync_init();
            nbgl_useCaseGenericConfiguration(
                match self.title.is_empty() {
                    true => core::ptr::null(),
                    false => self.title.as_ptr() as *const c_char,
                },
                self.init_page,
                &contents as *const nbgl_genericContents_t,
                Some(quit_callback),
            );
            self.ux_sync_wait(false);
        }
    }
}
