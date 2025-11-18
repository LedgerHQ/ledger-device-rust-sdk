use super::*;

static mut NVM_REF: Option<&mut AtomicStorage<[u8; SETTINGS_SIZE]>> = None;
static mut SWITCH_ARRAY: [nbgl_contentSwitch_t; SETTINGS_SIZE] =
    [unsafe { const_zero!(nbgl_contentSwitch_t) }; SETTINGS_SIZE];

/// Callback triggered by the NBGL API  when a setting switch is toggled.
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

#[derive(Default)]
struct InfoHolder {
    fields: Vec<CString>,
    values: Vec<CString>,
    fields_ptr: Vec<*const i8>,
    values_ptr: Vec<*const i8>,
}

pub struct NbglGenericSettings {
    title: CString,
    init_page: usize,
    info: InfoHolder,
    info_list: Option<nbgl_contentInfoList_t>,
    settings_title_subtitle: Vec<[CString; 2]>,
    content: nbgl_content_t,
    generic_contents: nbgl_genericContents_t,
}

impl SyncNBGL for NbglGenericSettings {}

impl NbglGenericSettings {
    pub fn new() -> NbglGenericSettings {
        NbglGenericSettings {
            title: CString::default(),
            init_page: 0,
            info: InfoHolder::default(),
            info_list: None,
            settings_title_subtitle: Vec::default(),
            content: nbgl_content_t::default(),
            generic_contents: nbgl_genericContents_t::default(),
        }
    }

    pub fn title(self, title: &str) -> NbglGenericSettings {
        NbglGenericSettings {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    pub fn init_page(self, init_page: usize) -> NbglGenericSettings {
        NbglGenericSettings { init_page, ..self }
    }

    pub fn info(mut self, fields_values: &[(&str, &str)]) -> NbglGenericSettings {
        for (f, v) in fields_values.iter() {
            self.info.fields.push(CString::new(*f).unwrap());
            self.info.values.push(CString::new(*v).unwrap());
            self.info
                .fields_ptr
                .push(self.info.fields.last().unwrap().as_ptr() as *const i8);
            self.info
                .values_ptr
                .push(self.info.values.last().unwrap().as_ptr() as *const i8);
        }

        self.info_list = Some(nbgl_contentInfoList_t {
            infoTypes: self.info.fields_ptr[..].as_ptr() as *const *const ::core::ffi::c_char,
            infoContents: self.info.values_ptr[..].as_ptr() as *const *const ::core::ffi::c_char,
            nbInfos: fields_values.len() as u8,
            infoExtensions: core::ptr::null(),
            token: 0,
            withExtensions: false,
        });
        self
    }

    pub fn settings(
        mut self,
        nvm_data: &mut AtomicStorage<[u8; SETTINGS_SIZE]>,
        settings_strings: &[[&str; 2]],
    ) -> NbglGenericSettings {
        if settings_strings.len() > SETTINGS_SIZE {
            panic!("Too many settings.");
        }

        self.settings_title_subtitle = settings_strings
            .iter()
            .map(|s| [CString::new(s[0]).unwrap(), CString::new(s[1]).unwrap()])
            .collect();

        unsafe {
            NVM_REF = Some(transmute(nvm_data));
            for (i, setting) in self.settings_title_subtitle.iter().enumerate() {
                SWITCH_ARRAY[i].text = setting[0].as_ptr();
                SWITCH_ARRAY[i].subText = setting[1].as_ptr();
                let state = if let Some(data) = (*(&raw mut NVM_REF)).as_mut() {
                    data.get_ref()[i]
                } else {
                    OFF_STATE
                };
                SWITCH_ARRAY[i].initState = state;
                SWITCH_ARRAY[i].token = (FIRST_USER_TOKEN + i as u32) as u8;
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                {
                    SWITCH_ARRAY[i].tuneId = TuneIndex::TapCasual as u8;
                }
            }
        }

        self.content = nbgl_content_t {
            content: nbgl_content_u {
                switchesList: nbgl_pageSwitchesList_s {
                    switches: &raw const SWITCH_ARRAY as *const nbgl_contentSwitch_t,
                    nbSwitches: settings_strings.len() as u8,
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

        self
    }

    pub fn show(&mut self) -> SyncNbgl {
        self.ux_sync_init();
        unsafe {
            nbgl_useCaseGenericSettings(
                self.title.as_ptr() as *const c_char,
                self.init_page as u8,
                &self.generic_contents as *const nbgl_genericContents_t,
                match self.info_list {
                    Some(ref il) => il as *const nbgl_contentInfoList_t,
                    None => core::ptr::null(),
                },
                Some(quit_callback),
            )
        }
        self.ux_sync_wait(false)
    }
}
