use super::*;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

// Own state, sharing the type-erasure helpers with the home screen: the two
// screens are never up at once, but keeping the pointers separate means neither
// can be left holding the other's.
static NVM_PTR: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static mut NVM_OPS: Option<NvmOps> = None;
static SWITCHES_PTR: AtomicPtr<nbgl_contentSwitch_t> = AtomicPtr::new(core::ptr::null_mut());
static SWITCHES_LEN: AtomicUsize = AtomicUsize::new(0);

/// Callback triggered by the NBGL API  when a setting switch is toggled.
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
    /// Switch descriptors handed to C; owned here so their number is not capped.
    switches: Vec<nbgl_contentSwitch_t>,
    content: nbgl_content_t,
    generic_contents: nbgl_genericContents_t,
}

impl SyncNBGL for NbglGenericSettings {}

impl Default for NbglGenericSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl NbglGenericSettings {
    pub fn new() -> NbglGenericSettings {
        NbglGenericSettings {
            title: CString::default(),
            init_page: 0,
            info: InfoHolder::default(),
            info_list: None,
            settings_title_subtitle: Vec::default(),
            switches: Vec::default(),
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

    /// # Panics
    /// Panics if there are more settings than the NVM array has bytes, each
    /// setting needing one byte of storage.
    // `tuneId` is touchscreen-only, so the struct update below is needed on
    // Nano and redundant elsewhere.
    #[allow(clippy::needless_update)]
    pub fn settings<const N: usize>(
        mut self,
        nvm_data: &mut AtomicStorage<[u8; N]>,
        settings_strings: &[[&str; 2]],
    ) -> NbglGenericSettings {
        if settings_strings.len() > N {
            panic!("More settings than the NVM array has bytes.");
        }

        self.settings_title_subtitle = settings_strings
            .iter()
            .map(|s| [CString::new(s[0]).unwrap(), CString::new(s[1]).unwrap()])
            .collect();

        NVM_PTR.store(
            nvm_data as *mut AtomicStorage<[u8; N]> as *mut (),
            Ordering::Relaxed,
        );
        unsafe {
            NVM_OPS = Some(NvmOps {
                read: nvm_read::<N>,
                toggle: nvm_toggle::<N>,
            });

            let nvm = NVM_PTR.load(Ordering::Relaxed);
            self.switches = self
                .settings_title_subtitle
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

            SWITCHES_PTR.store(self.switches.as_mut_ptr(), Ordering::Relaxed);
            SWITCHES_LEN.store(self.switches.len(), Ordering::Relaxed);
        }

        self.content = nbgl_content_t {
            content: nbgl_content_u {
                switchesList: nbgl_pageSwitchesList_s {
                    switches: self.switches.as_ptr(),
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

    fn show_internal(&mut self) -> SyncNbgl {
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

    #[cfg(not(feature = "io_new"))]
    pub fn show(&mut self) -> SyncNbgl {
        self.show_internal()
    }

    /// # Returns
    /// Returns `Ok(())` once the user exits the settings screen,
    /// or `Err(u8)` with the error code in case of an error.
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(&mut self, _comm: &mut crate::io::Comm<N>) -> Result<(), u8> {
        let ret = self.show_internal();
        match ret {
            SyncNbgl::UxSyncRetQuitted => Ok(()),
            _ => Err(u8::from(ret)),
        }
    }
}
