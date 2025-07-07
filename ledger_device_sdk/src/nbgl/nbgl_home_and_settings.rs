use super::*;

pub const SETTINGS_SIZE: usize = 10;
static mut NVM_REF: Option<&mut AtomicStorage<[u8; SETTINGS_SIZE]>> = None;
static mut SWITCH_ARRAY: [nbgl_contentSwitch_t; SETTINGS_SIZE] =
    [unsafe { const_zero!(nbgl_contentSwitch_t) }; SETTINGS_SIZE];

/// Callback triggered by the NBGL API when a setting switch is toggled.
unsafe extern "C" fn settings_callback(token: c_int, _index: u8, _page: c_int) {
    let idx = token - FIRST_USER_TOKEN as i32;
    if idx < 0 || idx >= SETTINGS_SIZE as i32 {
        panic!("Invalid token.");
    }

    let setting_idx: usize = idx as usize;

    match SWITCH_ARRAY[setting_idx].initState {
        OFF_STATE => SWITCH_ARRAY[setting_idx].initState = ON_STATE,
        ON_STATE => SWITCH_ARRAY[setting_idx].initState = OFF_STATE,
        _ => panic!("Invalid state."),
    }

    if let Some(data) = (*(&raw mut NVM_REF)).as_mut() {
        let mut switch_values: [u8; SETTINGS_SIZE] = *data.get_ref();
        if switch_values[setting_idx] == OFF_STATE {
            switch_values[setting_idx] = ON_STATE;
        } else {
            switch_values[setting_idx] = OFF_STATE;
        }
        data.update(&switch_values);
    }
}

/// Informations fields name to display in the dedicated
/// page of the home screen.
const INFO_FIELDS: [*const c_char; 2] = [
    "Version\0".as_ptr() as *const c_char,
    "Developer\0".as_ptr() as *const c_char,
];

pub enum PageIndex {
    Settings(u8),
    Home,
}

/// Used to display the home screen of the application, with an optional glyph,
/// information fields, and settings switches.
pub struct NbglHomeAndSettings {
    app_name: CString,
    tag_line: Option<CString>,
    info_contents: Vec<CString>,
    info_contents_ptr: Vec<*const c_char>,
    setting_contents: Vec<[CString; 2]>,
    nb_settings: u8,
    content: nbgl_content_t,
    generic_contents: nbgl_genericContents_t,
    info_list: nbgl_contentInfoList_t,
    icon: nbgl_icon_details_t,
    start_page: PageIndex,
}

impl SyncNBGL for NbglHomeAndSettings {}

unsafe extern "C" fn quit_cb() {
    exit_app(0);
}

impl<'a> Default for NbglHomeAndSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> NbglHomeAndSettings {
    pub fn new() -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            app_name: CString::new("").unwrap(),
            tag_line: None,
            info_contents: Vec::default(),
            info_contents_ptr: Vec::default(),
            setting_contents: Vec::default(),
            nb_settings: 0,
            content: nbgl_content_t::default(),
            generic_contents: nbgl_genericContents_t::default(),
            info_list: nbgl_contentInfoList_t::default(),
            icon: nbgl_icon_details_t::default(),
            start_page: PageIndex::Home,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglHomeAndSettings {
        let icon = glyph.into();
        NbglHomeAndSettings { icon, ..self }
    }

    pub fn infos(
        self,
        app_name: &'a str,
        version: &'a str,
        author: &'a str,
    ) -> NbglHomeAndSettings {
        let v: Vec<CString> = vec![
            CString::new(version).unwrap(),
            CString::new(author).unwrap(),
        ];

        NbglHomeAndSettings {
            app_name: CString::new(app_name).unwrap(),
            info_contents: v,
            ..self
        }
    }

    pub fn tagline(self, tagline: &'a str) -> NbglHomeAndSettings {
        NbglHomeAndSettings {
            tag_line: Some(CString::new(tagline).unwrap()),
            ..self
        }
    }

    pub fn settings(
        self,
        nvm_data: &'a mut AtomicStorage<[u8; SETTINGS_SIZE]>,
        settings_strings: &[[&'a str; 2]],
    ) -> NbglHomeAndSettings {
        unsafe {
            NVM_REF = Some(transmute(nvm_data));
        }

        if settings_strings.len() > SETTINGS_SIZE {
            panic!("Too many settings.");
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

    pub fn set_start_page(&mut self, page: PageIndex) {
        self.start_page = page;
    }

    /// Show the home screen and settings page.
    /// This function will block until an APDU is received or the user quits the app.
    /// DEPRECATED as it constraints to refresh screen for every received APDU.
    /// Use `show_and_return` instead.
    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        unsafe {
            loop {
                self.info_contents_ptr = self
                    .info_contents
                    .iter()
                    .map(|s| s.as_ptr())
                    .collect::<Vec<_>>();

                self.info_list = nbgl_contentInfoList_t {
                    infoTypes: INFO_FIELDS.as_ptr(),
                    infoContents: self.info_contents_ptr[..].as_ptr(),
                    nbInfos: INFO_FIELDS.len() as u8,
                    infoExtensions: core::ptr::null(),
                    token: 0,
                    withExtensions: false,
                };

                for (i, setting) in self.setting_contents.iter().enumerate() {
                    SWITCH_ARRAY[i].text = setting[0].as_ptr();
                    SWITCH_ARRAY[i].subText = setting[1].as_ptr();
                    let state = if let Some(data) = (*(&raw mut NVM_REF)).as_mut() {
                        data.get_ref()[i]
                    } else {
                        OFF_STATE
                    };
                    SWITCH_ARRAY[i].initState = state;
                    SWITCH_ARRAY[i].token = (FIRST_USER_TOKEN + i as u32) as u8;
                    #[cfg(any(target_os = "stax", target_os = "flex"))]
                    {
                        SWITCH_ARRAY[i].tuneId = TuneIndex::TapCasual as u8;
                    }
                }

                self.content = nbgl_content_t {
                    content: nbgl_content_u {
                        switchesList: nbgl_pageSwitchesList_s {
                            switches: &raw const SWITCH_ARRAY as *const nbgl_contentSwitch_t,
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

                self.ux_sync_init();
                nbgl_useCaseHomeAndSettings(
                    self.app_name.as_ptr() as *const c_char,
                    &self.icon as *const nbgl_icon_details_t,
                    match self.tag_line {
                        None => core::ptr::null(),
                        Some(ref tag) => tag.as_ptr() as *const c_char,
                    },
                    match self.start_page {
                        PageIndex::Home => INIT_HOME_PAGE as u8,
                        PageIndex::Settings(idx) => idx,
                    },
                    match self.nb_settings {
                        0 => core::ptr::null(),
                        _ => &self.generic_contents as *const nbgl_genericContents_t,
                    },
                    &self.info_list as *const nbgl_contentInfoList_t,
                    core::ptr::null(),
                    Some(quit_callback),
                );
                match self.ux_sync_wait(true) {
                    SyncNbgl::UxSyncRetApduReceived => {
                        if let Some(comm) = (*(&raw mut COMM_REF)).as_mut() {
                            if let Some(value) = comm.check_event() {
                                return value;
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
    /// This function returns immediately after the screen is displayed.
    pub fn show_and_return(&mut self) {
        unsafe {
            self.info_contents_ptr = self
                .info_contents
                .iter()
                .map(|s| s.as_ptr())
                .collect::<Vec<_>>();

            self.info_list = nbgl_contentInfoList_t {
                infoTypes: INFO_FIELDS.as_ptr(),
                infoContents: self.info_contents_ptr[..].as_ptr(),
                nbInfos: INFO_FIELDS.len() as u8,
                infoExtensions: core::ptr::null(),
                token: 0,
                withExtensions: false,
            };

            for (i, setting) in self.setting_contents.iter().enumerate() {
                SWITCH_ARRAY[i].text = setting[0].as_ptr();
                SWITCH_ARRAY[i].subText = setting[1].as_ptr();
                let state = if let Some(data) = (*(&raw mut NVM_REF)).as_mut() {
                    data.get_ref()[i]
                } else {
                    OFF_STATE
                };
                SWITCH_ARRAY[i].initState = state;
                SWITCH_ARRAY[i].token = (FIRST_USER_TOKEN + i as u32) as u8;
                #[cfg(any(target_os = "stax", target_os = "flex"))]
                {
                    SWITCH_ARRAY[i].tuneId = TuneIndex::TapCasual as u8;
                }
            }

            self.content = nbgl_content_t {
                content: nbgl_content_u {
                    switchesList: nbgl_pageSwitchesList_s {
                        switches: &raw const SWITCH_ARRAY as *const nbgl_contentSwitch_t,
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

            nbgl_useCaseHomeAndSettings(
                self.app_name.as_ptr() as *const c_char,
                &self.icon as *const nbgl_icon_details_t,
                match self.tag_line {
                    None => core::ptr::null(),
                    Some(ref tag) => tag.as_ptr() as *const c_char,
                },
                match self.start_page {
                    PageIndex::Home => INIT_HOME_PAGE as u8,
                    PageIndex::Settings(idx) => idx,
                },
                match self.nb_settings {
                    0 => core::ptr::null(),
                    _ => &self.generic_contents as *const nbgl_genericContents_t,
                },
                &self.info_list as *const nbgl_contentInfoList_t,
                core::ptr::null(),
                Some(quit_cb),
            );
        }
    }
}
