use crate::io::{ApduHeader, Comm, Event, Reply};
use crate::nvm::*;
use const_zero::const_zero;
extern crate alloc;
use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::c_char;
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::vec::Vec;

#[no_mangle]
pub static mut G_ux_params: bolos_ux_params_t = unsafe { const_zero!(bolos_ux_params_t) };

static mut COMM_REF: Option<&mut Comm> = None;
const SETTINGS_SIZE: usize = 10;
static mut NVM_REF: Option<&mut AtomicStorage<[u8; SETTINGS_SIZE]>> = None;
static mut SWITCH_ARRAY: [nbgl_contentSwitch_t; SETTINGS_SIZE] =
    [unsafe { const_zero!(nbgl_contentSwitch_t) }; SETTINGS_SIZE];

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

struct CField {
    pub name: CString,
    pub value: CString,
}

pub struct NbglGlyph<'a> {
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub is_file: bool,
    pub bitmap: &'a [u8],
}

impl<'a> NbglGlyph<'a> {
    pub const fn new(
        bitmap: &'a [u8],
        width: u16,
        height: u16,
        bpp: u8,
        is_file: bool,
    ) -> NbglGlyph<'a> {
        NbglGlyph {
            width,
            height,
            bpp,
            is_file,
            bitmap,
        }
    }
    pub const fn from_include(packed: (&'a [u8], u16, u16, u8, bool)) -> NbglGlyph<'a> {
        NbglGlyph {
            width: packed.1,
            height: packed.2,
            bpp: packed.3,
            is_file: packed.4,
            bitmap: packed.0,
        }
    }
}

impl<'a> Into<nbgl_icon_details_t> for &NbglGlyph<'a> {
    fn into(self) -> nbgl_icon_details_t {
        let bpp = match self.bpp {
            1 => NBGL_BPP_1,
            2 => NBGL_BPP_2,
            4 => NBGL_BPP_4,
            _ => panic!("Invalid bpp"),
        };
        nbgl_icon_details_t {
            width: self.width,
            height: self.height,
            bpp,
            isFile: self.is_file,
            bitmap: self.bitmap.as_ptr() as *const u8,
        }
    }
}

/// Initialize the global COMM_REF variable with the provided Comm instance.
/// This function should be called from the main function of the application.
/// The COMM_REF variable is used by the NBGL API to detect touch events and
/// APDU reception.
pub fn init_comm(comm: &mut Comm) {
    unsafe {
        COMM_REF = Some(transmute(comm));
    }
}

/// IO function used in the synchronous NBGL C library to process
/// events (touch, buttons, etc.) or detect if an APDU was received.
/// It returns true if an APDU was received, false otherwise.
#[no_mangle]
pub extern "C" fn io_recv_and_process_event() -> bool {
    unsafe {
        if let Some(comm) = COMM_REF.as_mut() {
            let apdu_received = comm.next_event_ahead::<ApduHeader>();
            if apdu_received {
                return true;
            }
        }
    }
    false
}

/// Callback triggered by the NBGL API when a setting switch is toggled.
unsafe extern "C" fn settings_callback(
    token: ::core::ffi::c_int,
    _index: u8,
    _page: ::core::ffi::c_int,
) {
    let idx = token - FIRST_USER_TOKEN as i32;
    if idx < 0 || idx >= SETTINGS_SIZE as i32 {
        panic!("Invalid token.");
    }

    if let Some(data) = NVM_REF.as_mut() {
        let setting_idx: usize = idx as usize;
        let mut switch_values: [u8; SETTINGS_SIZE] = data.get_ref().clone();
        switch_values[setting_idx] = !switch_values[setting_idx];
        data.update(&switch_values);
        SWITCH_ARRAY[setting_idx].initState = switch_values[setting_idx] as nbgl_state_t;
    }
}

/// Informations fields name to display in the dedicated
/// page of the home screen.
const INFO_FIELDS: [*const c_char; 2] = [
    "Version\0".as_ptr() as *const c_char,
    "Developer\0".as_ptr() as *const c_char,
];

/// A wrapper around the synchronous NBGL ux_sync_homeAndSettings C API binding.
/// Used to display the home screen of the application, with an optional glyph,
/// information fields, and settings switches.  
pub struct NbglHomeAndSettings<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    // app_name, version, author
    info_contents: Vec<CString>,
    setting_contents: Vec<[CString; 2]>,
    nb_settings: u8,
}

impl<'a> NbglHomeAndSettings<'a> {
    pub fn new() -> NbglHomeAndSettings<'a> {
        NbglHomeAndSettings {
            glyph: None,
            info_contents: Vec::default(),
            setting_contents: Vec::default(),
            nb_settings: 0,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglHomeAndSettings<'a> {
        NbglHomeAndSettings {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn infos(
        self,
        app_name: &'a str,
        version: &'a str,
        author: &'a str,
    ) -> NbglHomeAndSettings<'a> {
        let mut v: Vec<CString> = Vec::new();
        v.push(CString::new(app_name).unwrap());
        v.push(CString::new(version).unwrap());
        v.push(CString::new(author).unwrap());
        NbglHomeAndSettings {
            info_contents: v,
            ..self
        }
    }

    pub fn settings(
        self,
        nvm_data: &'a mut AtomicStorage<[u8; SETTINGS_SIZE]>,
        settings_strings: &[[&'a str; 2]],
    ) -> NbglHomeAndSettings<'a> {
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

    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        unsafe {
            loop {
                let info_contents: Vec<*const c_char> = self
                    .info_contents
                    .iter()
                    .map(|s| s.as_ptr())
                    .collect::<Vec<_>>();

                let info_list: nbgl_contentInfoList_t = nbgl_contentInfoList_t {
                    infoTypes: INFO_FIELDS.as_ptr() as *const *const c_char,
                    infoContents: info_contents[1..].as_ptr() as *const *const c_char,
                    nbInfos: INFO_FIELDS.len() as u8,
                };

                let icon: nbgl_icon_details_t = match self.glyph {
                    Some(g) => g.into(),
                    None => nbgl_icon_details_t::default(),
                };

                for (i, setting) in self.setting_contents.iter().enumerate() {
                    SWITCH_ARRAY[i].text = setting[0].as_ptr();
                    SWITCH_ARRAY[i].subText = setting[1].as_ptr();
                    SWITCH_ARRAY[i].initState =
                        NVM_REF.as_mut().unwrap().get_ref()[i] as nbgl_state_t;
                    SWITCH_ARRAY[i].token = (FIRST_USER_TOKEN + i as u32) as u8;
                    SWITCH_ARRAY[i].tuneId = TuneIndex::TapCasual as u8;
                }

                let content: nbgl_content_t = nbgl_content_t {
                    content: nbgl_content_u {
                        switchesList: nbgl_pageSwitchesList_s {
                            switches: &SWITCH_ARRAY as *const nbgl_contentSwitch_t,
                            nbSwitches: self.nb_settings,
                        },
                    },
                    contentActionCallback: Some(settings_callback),
                    type_: SWITCHES_LIST,
                };

                let generic_contents: nbgl_genericContents_t = nbgl_genericContents_t {
                    callbackCallNeeded: false,
                    __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                        contentsList: &content as *const nbgl_content_t,
                    },
                    nbContents: if self.nb_settings > 0 { 1 } else { 0 },
                };

                match ledger_secure_sdk_sys::ux_sync_homeAndSettings(
                    info_contents[0],
                    &icon as *const nbgl_icon_details_t,
                    core::ptr::null(),
                    INIT_HOME_PAGE as u8,
                    &generic_contents as *const nbgl_genericContents_t,
                    &info_list as *const nbgl_contentInfoList_t,
                    core::ptr::null(),
                ) {
                    ledger_secure_sdk_sys::UX_SYNC_RET_APDU_RECEIVED => {
                        if let Some(comm) = COMM_REF.as_mut() {
                            if let Some(value) = comm.check_event() {
                                return value;
                            }
                        }
                    }
                    _ => {
                        panic!("Unexpected return value from ux_sync_homeAndSettings");
                    }
                }
            }
        }
    }
}

/// A wrapper around the synchronous NBGL ux_sync_review C API binding.
/// Used to display transaction review screens.
pub struct NbglReview<'a> {
    title: CString,
    subtitle: CString,
    finish_title: CString,
    glyph: Option<&'a NbglGlyph<'a>>,
}

impl<'a> NbglReview<'a> {
    pub fn new() -> NbglReview<'a> {
        NbglReview {
            title: CString::new("").unwrap(),
            subtitle: CString::new("").unwrap(),
            finish_title: CString::new("").unwrap(),
            glyph: None,
        }
    }

    pub fn titles(
        self,
        title: &'a str,
        subtitle: &'a str,
        finish_title: &'a str,
    ) -> NbglReview<'a> {
        NbglReview {
            title: CString::new(title).unwrap(),
            subtitle: CString::new(subtitle).unwrap(),
            finish_title: CString::new(finish_title).unwrap(),
            ..self
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglReview<'a> {
        NbglReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(&mut self, fields: &[Field]) -> bool {
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
                let mut val = nbgl_contentTagValue_t::default();
                val.item = field.name.as_ptr() as *const i8;
                val.value = field.value.as_ptr() as *const i8;
                tag_value_array.push(val);
            }

            // Create the tag_value_list with the tag_value_array.
            let mut tag_value_list = nbgl_contentTagValueList_t::default();
            tag_value_list.pairs = tag_value_array.as_ptr() as *const nbgl_contentTagValue_t;
            tag_value_list.nbPairs = fields.len() as u8;

            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            // Show the review on the device.
            let sync_ret = ledger_secure_sdk_sys::ux_sync_review(
                TYPE_TRANSACTION.into(),
                &tag_value_list as *const nbgl_contentTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                self.title.as_ptr() as *const c_char,
                self.subtitle.as_ptr() as *const c_char,
                self.finish_title.as_ptr() as *const c_char,
            );

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_TRANSACTION_SIGNED);
                    return true;
                }
                _ => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_TRANSACTION_REJECTED);
                    return false;
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum CenteredInfoStyle {
    LargeCaseInfo = 0,
    LargeCaseBoldInfo,
    NormalInfo,
    PluginInfo,
}

impl From<CenteredInfoStyle> for nbgl_contentCenteredInfoStyle_t {
    fn from(style: CenteredInfoStyle) -> nbgl_contentCenteredInfoStyle_t {
        match style {
            CenteredInfoStyle::LargeCaseInfo => LARGE_CASE_INFO,
            CenteredInfoStyle::LargeCaseBoldInfo => LARGE_CASE_BOLD_INFO,
            CenteredInfoStyle::NormalInfo => NORMAL_INFO,
            CenteredInfoStyle::PluginInfo => PLUGIN_INFO,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// centered info screen that will be displayed on the device
/// when using the NbglGenericReview struct.
pub struct CenteredInfo {
    text1: CString,
    text2: CString,
    text3: CString,
    icon: Option<Box<nbgl_icon_details_t>>,
    on_top: bool,
    style: CenteredInfoStyle,
    offset_y: i16,
}

impl CenteredInfo {
    pub fn new(
        text1: &str,
        text2: &str,
        text3: &str,
        icon: Option<&NbglGlyph>,
        on_top: bool,
        style: CenteredInfoStyle,
        offset_y: i16,
    ) -> CenteredInfo {
        let icon_boxed: Option<Box<nbgl_icon_details_t>> = match icon {
            Some(glyph) => Some(Box::new(glyph.into())),
            None => None,
        };
        CenteredInfo {
            text1: CString::new(text1).unwrap(),
            text2: CString::new(text2).unwrap(),
            text3: CString::new(text3).unwrap(),
            icon: icon_boxed,
            on_top: on_top,
            style: style,
            offset_y: offset_y,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// "long press" button to confirm some information that will be displayed
/// on the device when using the NbglGenericReview struct.
pub struct InfoLongPress {
    text: CString,
    icon: Option<Box<nbgl_icon_details_t>>,
    long_press_text: CString,
    tune_id: TuneIndex,
}

impl InfoLongPress {
    pub fn new(
        text: &str,
        icon: Option<&NbglGlyph>,
        long_press_text: &str,
        tune_id: TuneIndex,
    ) -> InfoLongPress {
        let icon_boxed: Option<Box<nbgl_icon_details_t>> = match icon {
            Some(glyph) => Some(Box::new(glyph.into())),
            None => None,
        };
        InfoLongPress {
            text: CString::new(text).unwrap(),
            icon: icon_boxed,
            long_press_text: CString::new(long_press_text).unwrap(),
            tune_id: tune_id,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// button to confirm some information that will be displayed
/// on the device when using the NbglGenericReview struct.
pub struct InfoButton {
    text: CString,
    icon: Option<Box<nbgl_icon_details_t>>,
    button_text: CString,
    tune_id: TuneIndex,
}

impl InfoButton {
    pub fn new(
        text: &str,
        icon: Option<&NbglGlyph>,
        button_text: &str,
        tune_id: TuneIndex,
    ) -> InfoButton {
        let icon_boxed: Option<Box<nbgl_icon_details_t>> = match icon {
            Some(glyph) => Some(Box::new(glyph.into())),
            None => None,
        };
        InfoButton {
            text: CString::new(text).unwrap(),
            icon: icon_boxed,
            button_text: CString::new(button_text).unwrap(),
            tune_id: tune_id,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// tag/value list screen that will be displayed on the device when
/// using the NbglGenericReview struct.
pub struct TagValueList {
    pairs: Vec<nbgl_contentTagValue_t>,
    items: Vec<CString>,
    values: Vec<CString>,
    nb_max_lines_for_value: u8,
    small_case_for_value: bool,
    wrapping: bool,
}

impl TagValueList {
    pub fn new(
        pairs: &[Field],
        nb_max_lines_for_value: u8,
        small_case_for_value: bool,
        wrapping: bool,
    ) -> TagValueList {
        let mut c_field_strings: Vec<nbgl_contentTagValue_t> = Vec::new();
        let mut c_field_names: Vec<CString> = Vec::new();
        let mut c_field_values: Vec<CString> = Vec::new();
        for field in pairs {
            let name = CString::new(field.name).unwrap();
            let value = CString::new(field.value).unwrap();
            c_field_strings.push(nbgl_contentTagValue_t {
                item: name.as_ptr() as *const c_char,
                value: value.as_ptr() as *const c_char,
                valueIcon: core::ptr::null() as *const nbgl_icon_details_t,
                _bitfield_align_1: [0; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                __bindgen_padding_0: [0; 3usize],
            });
            c_field_names.push(name);
            c_field_values.push(value);
        }
        TagValueList {
            pairs: c_field_strings,
            items: c_field_names,
            values: c_field_values,
            nb_max_lines_for_value,
            small_case_for_value,
            wrapping,
        }
    }
}

impl From<&TagValueList> for nbgl_contentTagValueList_t {
    fn from(list: &TagValueList) -> nbgl_contentTagValueList_t {
        nbgl_contentTagValueList_t {
            pairs: list.pairs.as_ptr() as *const nbgl_contentTagValue_t,
            callback: None,
            nbPairs: list.pairs.len() as u8,
            startIndex: 0,
            nbMaxLinesForValue: list.nb_max_lines_for_value,
            token: FIRST_USER_TOKEN as u8,
            smallCaseForValue: list.small_case_for_value,
            wrapping: list.wrapping,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// list of tag-value pairs and confirmation button that will be displayed
/// on the device when using the NbglGenericReview struct.
pub struct TagValueConfirm {
    tag_value_list: nbgl_contentTagValueList_t,
    tune_id: TuneIndex,
    confirmation_text: CString,
    cancel_text: CString,
}

impl TagValueConfirm {
    pub fn new(
        tag_value_list: &TagValueList,
        tune_id: TuneIndex,
        confirmation_text: &str,
        cancel_text: &str,
    ) -> TagValueConfirm {
        let confirmation_text_cstring = CString::new(confirmation_text).unwrap();
        let cancel_text_cstring = CString::new(cancel_text).unwrap();
        TagValueConfirm {
            tag_value_list: tag_value_list.into(),
            tune_id: tune_id,
            confirmation_text: confirmation_text_cstring,
            cancel_text: cancel_text_cstring,
        }
    }
}

/// Structure exposed by the NBGL Rust API to the user to create a
/// list of information fields that will be displayed on the device
/// when using the NbglGenericReview struct.
pub struct InfosList {
    info_types_cstrings: Vec<CString>,
    info_contents_cstrings: Vec<CString>,
    info_types_ptr: Vec<*const c_char>,
    info_contents_ptr: Vec<*const c_char>,
}

impl InfosList {
    pub fn new(infos: &[Field]) -> InfosList {
        let info_types_cstrings: Vec<CString> = infos
            .iter()
            .map(|field| CString::new(field.name).unwrap())
            .collect();
        let info_contents_cstrings: Vec<CString> = infos
            .iter()
            .map(|field| CString::new(field.value).unwrap())
            .collect();
        let info_types_ptr: Vec<*const c_char> =
            info_types_cstrings.iter().map(|s| s.as_ptr()).collect();
        let info_contents_ptr: Vec<*const c_char> =
            info_contents_cstrings.iter().map(|s| s.as_ptr()).collect();
        InfosList {
            info_types_cstrings: info_types_cstrings,
            info_contents_cstrings: info_contents_cstrings,
            info_types_ptr: info_types_ptr,
            info_contents_ptr: info_contents_ptr,
        }
    }
}

/// Represents the different types of content that can be displayed
/// on the device when using the NbglGenericReview add_content method.
pub enum NbglPageContent {
    CenteredInfo(CenteredInfo),
    InfoLongPress(InfoLongPress),
    InfoButton(InfoButton),
    TagValueList(TagValueList),
    TagValueConfirm(TagValueConfirm),
    InfosList(InfosList),
}

impl From<&NbglPageContent>
    for (
        nbgl_content_u,
        nbgl_contentType_t,
        nbgl_contentActionCallback_t,
    )
{
    fn from(
        content: &NbglPageContent,
    ) -> (
        nbgl_content_u,
        nbgl_contentType_t,
        nbgl_contentActionCallback_t,
    ) {
        match content {
            NbglPageContent::CenteredInfo(data) => (
                nbgl_content_u {
                    centeredInfo: nbgl_contentCenteredInfo_t {
                        text1: data.text1.as_ptr() as *const c_char,
                        text2: data.text2.as_ptr() as *const c_char,
                        text3: data.text3.as_ptr() as *const c_char,
                        icon: data.icon.as_ref().map_or(core::ptr::null(), |icon| {
                            icon.as_ref() as *const nbgl_icon_details_t
                        }),
                        onTop: data.on_top,
                        style: data.style.into(),
                        offsetY: data.offset_y,
                    },
                },
                CENTERED_INFO,
                None,
            ),
            NbglPageContent::TagValueList(data) => (
                nbgl_content_u {
                    tagValueList: nbgl_contentTagValueList_t {
                        pairs: data.pairs.as_ptr() as *const nbgl_contentTagValue_t,
                        callback: None,
                        nbPairs: data.pairs.len() as u8,
                        startIndex: 0,
                        nbMaxLinesForValue: data.nb_max_lines_for_value,
                        token: FIRST_USER_TOKEN as u8,
                        smallCaseForValue: data.small_case_for_value,
                        wrapping: data.wrapping,
                    },
                },
                TAG_VALUE_LIST,
                None,
            ),
            NbglPageContent::TagValueConfirm(data) => (
                nbgl_content_u {
                    tagValueConfirm: nbgl_contentTagValueConfirm_t {
                        tagValueList: data.tag_value_list,
                        detailsButtonIcon: core::ptr::null(),
                        detailsButtonText: core::ptr::null(),
                        detailsButtonToken: (FIRST_USER_TOKEN + 2) as u8,
                        tuneId: data.tune_id as u8,
                        confirmationText: data.confirmation_text.as_ptr() as *const c_char,
                        cancelText: data.cancel_text.as_ptr() as *const c_char,
                        confirmationToken: FIRST_USER_TOKEN as u8,
                        cancelToken: (FIRST_USER_TOKEN + 1) as u8,
                    },
                },
                TAG_VALUE_CONFIRM,
                Some(generic_content_action_callback),
            ),
            NbglPageContent::InfoLongPress(data) => (
                nbgl_content_u {
                    infoLongPress: nbgl_contentInfoLongPress_t {
                        text: data.text.as_ptr() as *const c_char,
                        icon: data.icon.as_ref().map_or(core::ptr::null(), |icon| {
                            icon.as_ref() as *const nbgl_icon_details_t
                        }),
                        longPressText: data.long_press_text.as_ptr() as *const c_char,
                        longPressToken: FIRST_USER_TOKEN as u8,
                        tuneId: data.tune_id as u8,
                    },
                },
                INFO_LONG_PRESS,
                Some(generic_content_action_callback),
            ),
            NbglPageContent::InfoButton(data) => (
                nbgl_content_u {
                    infoButton: nbgl_contentInfoButton_t {
                        text: data.text.as_ptr() as *const c_char,
                        icon: data.icon.as_ref().map_or(core::ptr::null(), |icon| {
                            icon.as_ref() as *const nbgl_icon_details_t
                        }),
                        buttonText: data.button_text.as_ptr() as *const c_char,
                        buttonToken: FIRST_USER_TOKEN as u8,
                        tuneId: data.tune_id as u8,
                    },
                },
                INFO_BUTTON,
                Some(generic_content_action_callback),
            ),
            NbglPageContent::InfosList(data) => (
                nbgl_content_u {
                    infosList: nbgl_contentInfoList_t {
                        infoTypes: data.info_types_ptr.as_ptr() as *const *const c_char,
                        infoContents: data.info_contents_ptr.as_ptr() as *const *const c_char,
                        nbInfos: data.info_types_cstrings.len() as u8,
                    },
                },
                INFOS_LIST,
                None,
            ),
        }
    }
}

pub struct NbglGenericReview {
    content_list: Vec<NbglPageContent>,
}

impl NbglGenericReview {
    pub fn new() -> NbglGenericReview {
        NbglGenericReview {
            content_list: Vec::new(),
        }
    }

    pub fn add_content(mut self, content: NbglPageContent) -> NbglGenericReview {
        self.content_list.push(content);
        self
    }

    fn to_c_content_list(&self) -> Vec<nbgl_content_t> {
        self.content_list
            .iter()
            .map(|content| {
                let (c_struct, content_type, action_callback) = content.into();
                nbgl_content_t {
                    content: c_struct,
                    contentActionCallback: action_callback,
                    type_: content_type,
                }
            })
            .collect()
    }

    pub fn show(&mut self, reject_button_str: &str, succeed_str: &str, rejected_str: &str) -> bool {
        unsafe {
            let c_content_list: Vec<nbgl_content_t> = self.to_c_content_list();

            let content_struct = nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: c_content_list.as_ptr() as *const nbgl_content_t,
                },
                nbContents: self.content_list.len() as u8,
            };

            let reject_button_cstring = CString::new(reject_button_str).unwrap();
            let succeed_cstring = CString::new(succeed_str).unwrap();
            let rejected_cstring = CString::new(rejected_str).unwrap();

            let sync_ret = ux_sync_genericReview(
                &content_struct as *const nbgl_genericContents_t,
                reject_button_cstring.as_ptr() as *const c_char,
            );

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    ledger_secure_sdk_sys::ux_sync_status(
                        succeed_cstring.as_ptr() as *const c_char,
                        true,
                    );
                    return true;
                }
                _ => {
                    ledger_secure_sdk_sys::ux_sync_status(
                        rejected_cstring.as_ptr() as *const c_char,
                        false,
                    );
                    return false;
                }
            }
        }
    }
}

/// A wrapper around the synchronous NBGL ux_sync_addressReview C API binding.
/// Used to display address confirmation screens.
pub struct NbglAddressReview<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    verify_str: CString,
}

impl<'a> NbglAddressReview<'a> {
    pub fn new() -> NbglAddressReview<'a> {
        NbglAddressReview {
            verify_str: CString::new("").unwrap(),
            glyph: None,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressReview<'a> {
        NbglAddressReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn verify_str(self, verify_str: &str) -> NbglAddressReview<'a> {
        NbglAddressReview {
            verify_str: CString::new(verify_str).unwrap(),
            ..self
        }
    }

    pub fn show(&mut self, address: &str) -> bool {
        unsafe {
            let icon: nbgl_icon_details_t = match self.glyph {
                Some(g) => g.into(),
                None => nbgl_icon_details_t::default(),
            };

            let address = CString::new(address).unwrap();

            // Show the address confirmation on the device.
            let sync_ret = ux_sync_addressReview(
                address.as_ptr(),
                core::ptr::null(),
                &icon as *const nbgl_icon_details_t,
                self.verify_str.as_ptr(),
                core::ptr::null(),
            );

            // Return true if the user approved the address, false otherwise.
            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_ADDRESS_VERIFIED);
                    return true;
                }
                ledger_secure_sdk_sys::UX_SYNC_RET_REJECTED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_ADDRESS_REJECTED);
                    return false;
                }
                _ => {
                    panic!("Unexpected return value from ux_sync_addressReview");
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum TuneIndex {
    Reserved,
    Boot,
    Charging,
    LedgerMoment,
    Error,
    Neutral,
    Lock,
    Success,
    LookAtMe,
    TapCasual,
    TapNext,
}

impl TryFrom<u8> for TuneIndex {
    type Error = ();
    fn try_from(index: u8) -> Result<TuneIndex, ()> {
        Ok(match index {
            TUNE_RESERVED => TuneIndex::Reserved,
            TUNE_BOOT => TuneIndex::Boot,
            TUNE_CHARGING => TuneIndex::Charging,
            TUNE_LEDGER_MOMENT => TuneIndex::LedgerMoment,
            TUNE_ERROR => TuneIndex::Error,
            TUNE_NEUTRAL => TuneIndex::Neutral,
            TUNE_LOCK => TuneIndex::Lock,
            TUNE_SUCCESS => TuneIndex::Success,
            TUNE_LOOK_AT_ME => TuneIndex::LookAtMe,
            TUNE_TAP_CASUAL => TuneIndex::TapCasual,
            TUNE_TAP_NEXT => TuneIndex::TapNext,
            _ => return Err(()),
        })
    }
}

// this is a mock that does nothing yet, but should become a direct translation
// of the C original. This was done to avoid compiling `os_io_seproxyhal.c` which
// includes many other things
#[no_mangle]
extern "C" fn io_seproxyhal_play_tune(tune_index: u8) {
    let index = TuneIndex::try_from(tune_index);
    if index.is_err() {
        return;
    }
}
