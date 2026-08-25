//! A wrapper around the asynchronous NBGL [nbgl_useCaseHomeAndSettings](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3454) C API binding.
//!
//! Draws the extended version of home page of an app (page on which we land when launching it
//! from dashboard) with automatic support of setting display.
//! It enables to use an action button
use super::*;
use crate::io::{Reply, StatusWords};
use crate::io_callbacks::{nbgl_fetch_apdu_header, nbgl_reply_status};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// The number of settings the examples store in NVM.
///
/// No longer a limit: [`NbglHomeAndSettings::settings`] takes an
/// `AtomicStorage<[u8; N]>` of whatever size the app declares, and the switch
/// descriptors are heap-allocated. An app may keep as many settings as its NVM
/// array has bytes.
pub const SETTINGS_SIZE: usize = 10;

/// The app's NVM settings storage, type-erased.
///
/// `settings` is generic over the array length, which a static cannot name, so
/// the pointer is erased and paired with functions monomorphised for it.
static NVM_PTR: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());

/// Reads and writes for the registered NVM storage, instantiated for its
/// concrete length.
#[derive(Copy, Clone)]
pub(crate) struct NvmOps {
    pub(crate) read: unsafe fn(*mut (), usize) -> u8,
    pub(crate) toggle: unsafe fn(*mut (), usize),
}

static mut NVM_OPS: Option<NvmOps> = None;

pub(crate) unsafe fn nvm_read<const N: usize>(ptr: *mut (), idx: usize) -> u8 {
    let data = unsafe { &*(ptr as *const AtomicStorage<[u8; N]>) };
    data.get_ref()[idx]
}

pub(crate) unsafe fn nvm_toggle<const N: usize>(ptr: *mut (), idx: usize) {
    let data = unsafe { &mut *(ptr as *mut AtomicStorage<[u8; N]>) };
    let mut values: [u8; N] = *data.get_ref();
    values[idx] = match values[idx] {
        OFF_STATE => ON_STATE,
        _ => OFF_STATE,
    };
    data.update(&values);
}

/// The switch descriptors handed to C, owned by the builder on the heap.
///
/// The callback reaches them through this pointer rather than a fixed-size
/// static, which is what used to cap the number of settings.
static SWITCHES_PTR: AtomicPtr<nbgl_contentSwitch_t> = AtomicPtr::new(core::ptr::null_mut());
static SWITCHES_LEN: AtomicUsize = AtomicUsize::new(0);

/// Callback triggered by the NBGL API when a setting switch is toggled.
unsafe extern "C" fn settings_callback(token: c_int, _index: u8, _page: c_int) {
    unsafe {
        let idx = token - FIRST_USER_TOKEN as i32;
        if idx < 0 || idx as usize >= SWITCHES_LEN.load(Ordering::Relaxed) {
            panic!("Invalid token.");
        }
        let setting_idx = idx as usize;

        let switches = SWITCHES_PTR.load(Ordering::Relaxed);
        if switches.is_null() {
            return;
        }
        let switch = &mut *switches.add(setting_idx);
        switch.initState = match switch.initState {
            OFF_STATE => ON_STATE,
            ON_STATE => OFF_STATE,
            _ => panic!("Invalid state."),
        };

        let ptr = NVM_PTR.load(Ordering::Relaxed);
        if let (false, Some(ops)) = (ptr.is_null(), NVM_OPS) {
            (ops.toggle)(ptr, setting_idx);
        }
    }
}

/// Initial page to display when showing the home and settings screen.
pub enum PageIndex {
    Settings(u8),
    Home,
}

/// App function to run when the home action button is touched.
///
/// `nbgl_callback_t` carries no user data, so the app's function is parked here
/// for [`action_callback`] to find.
static mut ACTION_CALLBACK: Option<fn()> = None;

/// Callback registered with NBGL for the home action button, forwarding to the
/// app's own function.
unsafe extern "C" fn action_callback() {
    if let Some(callback) = unsafe { ACTION_CALLBACK } {
        callback();
    }
}

/// Style of the home screen action button, mapped to `nbgl_homeActionStyle_t`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum HomeActionStyle {
    /// Black button, for the app's main action.
    #[default]
    Strong,
    /// White button, for extended features.
    Soft,
}

impl HomeActionStyle {
    fn to_c_type(self) -> nbgl_homeActionStyle_t {
        match self {
            HomeActionStyle::Strong => STRONG_HOME_ACTION,
            HomeActionStyle::Soft => SOFT_HOME_ACTION,
        }
    }
}

/// An action button on the home screen, mapped to `nbgl_homeAction_t`.
///
/// `callback` runs from NBGL's event dispatch when the button is touched; what
/// it does is entirely up to the app.
pub struct HomeAction<'a> {
    text: &'a str,
    icon: Option<&'a NbglGlyph<'a>>,
    style: HomeActionStyle,
    callback: fn(),
}

impl<'a> HomeAction<'a> {
    /// Creates a strong (black) action button labelled `text`, running
    /// `callback` when touched.
    pub fn new(text: &'a str, callback: fn()) -> HomeAction<'a> {
        HomeAction {
            text,
            icon: None,
            style: HomeActionStyle::Strong,
            callback,
        }
    }

    /// Icon drawn in the button.
    pub fn icon(self, icon: &'a NbglGlyph<'a>) -> HomeAction<'a> {
        HomeAction {
            icon: Some(icon),
            ..self
        }
    }

    /// Button style.
    pub fn style(self, style: HomeActionStyle) -> HomeAction<'a> {
        HomeAction { style, ..self }
    }
}

/// Owns the C string and icon behind an `nbgl_homeAction_t`.
struct CHomeAction {
    text: CString,
    icon: Option<nbgl_icon_details_t>,
    style: nbgl_homeActionStyle_t,
}

impl CHomeAction {
    fn new(action: &HomeAction) -> CHomeAction {
        unsafe {
            ACTION_CALLBACK = Some(action.callback);
        }
        CHomeAction {
            text: CString::new(action.text).unwrap(),
            icon: action.icon.map(|g| g.into()),
            style: action.style.to_c_type(),
        }
    }

    fn as_c_type(&self) -> nbgl_homeAction_t {
        nbgl_homeAction_t {
            text: self.text.as_ptr(),
            icon: match &self.icon {
                Some(icon) => icon as *const nbgl_icon_details_t,
                None => core::ptr::null(),
            },
            callback: Some(action_callback),
            style: self.style,
        }
    }
}

/// A builder to create and show a home and settings page.
pub struct NbglHomeAndSettings {
    app_name: CString,
    tag_line: Option<CString>,
    info_types: Vec<CString>,
    info_contents: Vec<CString>,
    info_types_ptr: Vec<*const c_char>,
    info_contents_ptr: Vec<*const c_char>,
    setting_contents: Vec<[CString; 2]>,
    /// Switch descriptors handed to C; owned here so their number is not capped.
    switches: Vec<nbgl_contentSwitch_t>,
    nb_settings: u8,
    content: nbgl_content_t,
    generic_contents: nbgl_genericContents_t,
    info_list: nbgl_contentInfoList_t,
    icon: nbgl_icon_details_t,
    start_page: PageIndex,
    /// Owns the C string and icon behind the action button.
    action: Option<CHomeAction>,
}

impl SyncNBGL for NbglHomeAndSettings {}

unsafe extern "C" fn quit_cb() {
    exit_app(0);
}

impl Default for NbglHomeAndSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl NbglHomeAndSettings {
    /// Creates a new home and settings page builder.
    /// # Returns
    /// Returns a new instance of `NbglHomeAndSettings`.
    pub fn new() -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            app_name: CString::new("").unwrap(),
            tag_line: None,
            info_types: Vec::default(),
            info_contents: Vec::default(),
            info_types_ptr: Vec::default(),
            info_contents_ptr: Vec::default(),
            setting_contents: Vec::default(),
            switches: Vec::default(),
            nb_settings: 0,
            content: nbgl_content_t::default(),
            generic_contents: nbgl_genericContents_t::default(),
            info_list: nbgl_contentInfoList_t::default(),
            icon: nbgl_icon_details_t::default(),
            start_page: PageIndex::Home,
            action: None,
        }
    }

    /// Adds an action button to the home screen.
    /// # Arguments
    /// * `action` - The button and the function to run when it is touched;
    ///   see [`HomeAction`].
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn action(self, action: &HomeAction) -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            action: Some(CHomeAction::new(action)),
            ..self
        }
    }

    /// Sets the icon to display in the center of the page.
    /// # Arguments
    /// * `glyph` - The icon to display in the center of the page.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn glyph(self, glyph: &NbglGlyph) -> NbglHomeAndSettings {
        let icon = glyph.into();
        NbglHomeAndSettings { icon, ..self }
    }

    /// Sets the name of the application, displayed on the home screen and used
    /// as the title of the settings pages.
    /// # Arguments
    /// * `app_name` - The name of the application.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn app_name(self, app_name: &str) -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            app_name: CString::new(app_name).unwrap(),
            ..self
        }
    }

    /// Sets an arbitrary list of informations to display in the dedicated
    /// page of the home screen.
    ///
    /// Replaces any list previously set, either by [`Self::infos`] or by an
    /// earlier call to this method.
    /// # Arguments
    /// * `infos` - Slice of `(type, content)` pairs, where `type` is the label
    ///   shown in bold and `content` the value below it. For instance
    ///   `[("Version", "1.0.0"), ("Developer", "Ledger"), ("Copyright", "(c) 2026 Ledger")]`.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn info_list(mut self, infos: &[(&str, &str)]) -> NbglHomeAndSettings {
        self.info_types = infos
            .iter()
            .map(|(t, _)| CString::new(*t).unwrap())
            .collect();
        self.info_contents = infos
            .iter()
            .map(|(_, c)| CString::new(*c).unwrap())
            .collect();
        self
    }

    /// Sets the application informations to display in the dedicated
    /// page of the home screen.
    ///
    /// Shortcut for the usual `Version` / `Developer` pair. Use
    /// [`Self::app_name`] together with [`Self::info_list`] to display an
    /// arbitrary set of fields instead.
    /// # Arguments
    /// * `app_name` - The name of the application.
    /// * `version` - The version of the application.
    /// * `author` - The author of the application.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn infos(self, app_name: &str, version: &str, author: &str) -> NbglHomeAndSettings {
        self.app_name(app_name)
            .info_list(&[("Version", version), ("Developer", author)])
    }

    /// Sets the tagline to display below the application name on the home screen.
    /// # Arguments
    /// * `tagline` - The tagline to display below the application name on the home screen.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn tagline(self, tagline: &str) -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            tag_line: Some(CString::new(tagline).unwrap()),
            ..self
        }
    }

    /// Sets the settings to display in the settings page.
    /// # Arguments
    /// * `nvm_data` - A mutable reference to an `AtomicStorage` containing the settings data.
    /// * `settings_strings` - A slice of tuples containing the setting name and description.
    /// # Panics
    /// Panics if there are more settings than the NVM array has bytes, each
    /// setting needing one byte of storage.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn settings<const N: usize>(
        self,
        nvm_data: &mut AtomicStorage<[u8; N]>,
        settings_strings: &[[&str; 2]],
    ) -> NbglHomeAndSettings {
        if settings_strings.len() > N {
            panic!("More settings than the NVM array has bytes.");
        }

        NVM_PTR.store(
            nvm_data as *mut AtomicStorage<[u8; N]> as *mut (),
            Ordering::Relaxed,
        );
        // Records reads and writes instantiated for this N, the length being
        // invisible to the statics above.
        unsafe {
            NVM_OPS = Some(NvmOps {
                read: nvm_read::<N>,
                toggle: nvm_toggle::<N>,
            });
        }

        let v: Vec<[CString; 2]> = settings_strings
            .iter()
            .map(|s| [CString::new(s[0]).unwrap(), CString::new(s[1]).unwrap()])
            .collect();

        NbglHomeAndSettings {
            nb_settings: settings_strings.len() as u8,
            setting_contents: v,
            ..self
        }
    }

    /// Sets the initial page to display when showing the home and settings screen.
    /// # Arguments
    /// * `page` - The initial page to display.
    /// # Returns
    /// Returns the builder itself to allow method chaining.
    pub fn set_start_page(&mut self, page: PageIndex) {
        self.start_page = page;
    }

    // `tuneId` is touchscreen-only, so the struct update below is needed on
    // Nano and redundant elsewhere.
    #[allow(clippy::needless_update)]
    /// Builds the C structures handed over to `nbgl_useCaseHomeAndSettings` from
    /// the builder state. The C API only borrows them, so they are kept alive in
    /// `self` and refreshed before every call.
    fn prepare(&mut self) {
        unsafe {
            self.info_types_ptr = self.info_types.iter().map(|s| s.as_ptr()).collect();
            self.info_contents_ptr = self.info_contents.iter().map(|s| s.as_ptr()).collect();

            self.info_list = nbgl_contentInfoList_t {
                infoTypes: self.info_types_ptr.as_ptr(),
                infoContents: self.info_contents_ptr.as_ptr(),
                nbInfos: self.info_types_ptr.len() as u8,
                infoExtensions: core::ptr::null(),
                token: 0,
                withExtensions: false,
            };

            let nvm = NVM_PTR.load(Ordering::Relaxed);
            self.switches = self
                .setting_contents
                .iter()
                .enumerate()
                .map(|(i, setting)| {
                    let state = match (nvm.is_null(), NVM_OPS) {
                        (false, Some(ops)) => (ops.read)(nvm, i),
                        _ => OFF_STATE,
                    };
                    nbgl_contentSwitch_t {
                        text: setting[0].as_ptr(),
                        subText: setting[1].as_ptr(),
                        initState: state,
                        token: (FIRST_USER_TOKEN + i as u32) as u8,
                        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                        tuneId: TuneIndex::TapCasual as u8,
                        ..Default::default()
                    }
                })
                .collect();

            // The callback toggles state through these, so they must point at
            // the vector before the screen can be touched.
            SWITCHES_PTR.store(self.switches.as_mut_ptr(), Ordering::Relaxed);
            SWITCHES_LEN.store(self.switches.len(), Ordering::Relaxed);

            self.content = nbgl_content_t {
                content: nbgl_content_u {
                    switchesList: nbgl_pageSwitchesList_s {
                        switches: self.switches.as_ptr(),
                        nbSwitches: self.nb_settings,
                    },
                },
                contentActionCallback: Some(settings_callback),
                type_: SWITCHES_LIST,
            };

            self.generic_contents = nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: &self.content as *const nbgl_content_t,
                },
                nbContents: 1,
            };
        }
    }

    /// Pointer to the info list, or NULL when no information field was set
    /// (the C API accepts a NULL `infosList`).
    fn info_list_ptr(&self) -> *const nbgl_contentInfoList_t {
        match self.info_list.nbInfos {
            0 => core::ptr::null(),
            _ => &self.info_list as *const nbgl_contentInfoList_t,
        }
    }

    /// Pointer to the settings contents, or NULL when no setting was set.
    fn settings_ptr(&self) -> *const nbgl_genericContents_t {
        match self.nb_settings {
            0 => core::ptr::null(),
            _ => &self.generic_contents as *const nbgl_genericContents_t,
        }
    }

    /// Pointer to the tagline, or NULL when none was set.
    fn tagline_ptr(&self) -> *const c_char {
        match self.tag_line {
            None => core::ptr::null(),
            Some(ref tag) => tag.as_ptr() as *const c_char,
        }
    }

    /// The action button struct, materialised so the call sites have something
    /// with a stable address to point at.
    fn action_c_type(&self) -> Option<nbgl_homeAction_t> {
        self.action.as_ref().map(|a| a.as_c_type())
    }

    /// Index of the page to start on.
    fn start_page_index(&self) -> u8 {
        match self.start_page {
            PageIndex::Home => INIT_HOME_PAGE as u8,
            PageIndex::Settings(idx) => idx,
        }
    }

    /// Show the home screen and settings page (internal implementation).
    fn show_internal<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        unsafe {
            loop {
                self.prepare();
                let action = self.action_c_type();

                self.ux_sync_init();
                nbgl_useCaseHomeAndSettings(
                    self.app_name.as_ptr() as *const c_char,
                    &self.icon as *const nbgl_icon_details_t,
                    self.tagline_ptr(),
                    self.start_page_index(),
                    self.settings_ptr(),
                    self.info_list_ptr(),
                    match &action {
                        Some(action) => action as *const nbgl_homeAction_t,
                        None => core::ptr::null(),
                    },
                    Some(quit_callback),
                );
                match self.ux_sync_wait(true) {
                    SyncNbgl::UxSyncRetApduReceived => {
                        if let Some(hdr) = nbgl_fetch_apdu_header() {
                            // Reconstruct minimal Event::Command using APDU header only.
                            // The generic parameter T: TryFrom<ApduHeader> will parse header.
                            match T::try_from(hdr) {
                                Ok(ins) => {
                                    return Event::Command(ins);
                                }
                                _ => {
                                    // In case of parse error we emulate a BadIns reply.
                                    nbgl_reply_status(Reply(StatusWords::BadIns as u16));
                                }
                            }
                        }
                    }
                    SyncNbgl::UxSyncRetQuitted => {
                        exit_app(0);
                    }
                    _ => {
                        panic!("Unexpected return value from ux_sync_homeAndSettings");
                    }
                }
            }
        }
    }

    /// Show the home screen and settings page.
    /// This function will block until an APDU is received or the user quits the app.
    /// # Arguments
    /// * `_comm` - Mutable reference to Comm.
    #[cfg(feature = "io_new")]
    #[deprecated(
        since = "1.37.0",
        note = "blocking on an APDU forces the home screen to be refreshed for every received APDU; use `show_and_return` instead"
    )]
    pub fn show<T: TryFrom<ApduHeader>, const N: usize>(
        &mut self,
        _comm: &mut crate::io::Comm<N>,
    ) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        self.show_internal()
    }

    /// Show the home screen and settings page.
    /// This function will block until an APDU is received or the user quits the app.
    #[cfg(not(feature = "io_new"))]
    #[deprecated(
        since = "1.37.0",
        note = "blocking on an APDU forces the home screen to be refreshed for every received APDU; use `show_and_return` instead"
    )]
    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        self.show_internal()
    }

    /// Show the home screen and settings page.
    /// This function returns immediately after the screen is displayed.
    pub fn show_and_return(&mut self) {
        self.prepare();
        let action = self.action_c_type();

        unsafe {
            nbgl_useCaseHomeAndSettings(
                self.app_name.as_ptr() as *const c_char,
                &self.icon as *const nbgl_icon_details_t,
                self.tagline_ptr(),
                self.start_page_index(),
                self.settings_ptr(),
                self.info_list_ptr(),
                match &action {
                    Some(action) => action as *const nbgl_homeAction_t,
                    None => core::ptr::null(),
                },
                Some(quit_cb),
            );
        }
    }
}
