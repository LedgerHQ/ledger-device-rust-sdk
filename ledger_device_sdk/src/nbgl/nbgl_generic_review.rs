use super::*;

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
    icon: Option<nbgl_icon_details_t>,
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
        CenteredInfo {
            text1: CString::new(text1).unwrap(),
            text2: CString::new(text2).unwrap(),
            text3: CString::new(text3).unwrap(),
            icon: icon.map_or(None, |g| Some(g.into())),
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
    icon: Option<nbgl_icon_details_t>,
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
        InfoLongPress {
            text: CString::new(text).unwrap(),
            icon: icon.map_or(None, |g| Some(g.into())),
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
    icon: Option<nbgl_icon_details_t>,
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
        InfoButton {
            text: CString::new(text).unwrap(),
            icon: icon.map_or(None, |g| Some(g.into())),
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
    _items: Vec<CString>,
    _values: Vec<CString>,
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
        let mut c_field_strings: Vec<nbgl_contentTagValue_t> = Vec::with_capacity(pairs.len());
        let mut c_field_names: Vec<CString> = Vec::with_capacity(pairs.len());
        let mut c_field_values: Vec<CString> = Vec::with_capacity(pairs.len());
        for field in pairs {
            let name = CString::new(field.name).unwrap();
            let value = CString::new(field.value).unwrap();
            let tag_value = nbgl_contentTagValue_t {
                item: name.as_ptr() as *const c_char,
                value: value.as_ptr() as *const c_char,
                ..Default::default()
            };
            c_field_strings.push(tag_value);
            c_field_names.push(name);
            c_field_values.push(value);
        }
        TagValueList {
            pairs: c_field_strings,
            _items: c_field_names,
            _values: c_field_values,
            nb_max_lines_for_value,
            small_case_for_value,
            wrapping,
        }
    }
}

impl From<&TagValueList> for nbgl_contentTagValueList_t {
    fn from(list: &TagValueList) -> nbgl_contentTagValueList_t {
        let list = nbgl_contentTagValueList_t {
            pairs: list.pairs.as_ptr() as *const nbgl_contentTagValue_t,
            nbPairs: list.pairs.len() as u8,
            nbMaxLinesForValue: list.nb_max_lines_for_value,
            token: FIRST_USER_TOKEN as u8,
            smallCaseForValue: list.small_case_for_value,
            wrapping: list.wrapping,
            ..Default::default()
        };
        list
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
    _info_contents_cstrings: Vec<CString>,
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
            _info_contents_cstrings: info_contents_cstrings,
            info_types_ptr: info_types_ptr,
            info_contents_ptr: info_contents_ptr,
        }
    }
}

unsafe extern "C" fn action_callback(token: c_int, _index: u8, _page: c_int) {
    if token == FIRST_USER_TOKEN as i32 {
        G_RET = SyncNbgl::UxSyncRetApproved.into();
    } else if token == (FIRST_USER_TOKEN + 1) as i32 {
        G_RET = SyncNbgl::UxSyncRetRejected.into();
    }
    G_ENDED = true;
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
            NbglPageContent::CenteredInfo(data) => {
                let centered_info = nbgl_contentCenteredInfo_t {
                    text1: data.text1.as_ptr() as *const c_char,
                    text2: data.text2.as_ptr() as *const c_char,
                    text3: data.text3.as_ptr() as *const c_char,
                    icon: data
                        .icon
                        .as_ref()
                        .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
                    onTop: data.on_top,
                    style: data.style.into(),
                    offsetY: data.offset_y,
                    ..Default::default()
                };
                (
                    nbgl_content_u {
                        centeredInfo: centered_info,
                    },
                    CENTERED_INFO,
                    None,
                )
            }
            NbglPageContent::TagValueList(data) => {
                let tag_list = nbgl_contentTagValueList_t {
                    pairs: data.pairs.as_ptr() as *const nbgl_contentTagValue_t,
                    nbPairs: data.pairs.len() as u8,
                    nbMaxLinesForValue: data.nb_max_lines_for_value,
                    smallCaseForValue: data.small_case_for_value,
                    wrapping: data.wrapping,
                    ..Default::default()
                };
                (
                    nbgl_content_u {
                        tagValueList: tag_list,
                    },
                    TAG_VALUE_LIST,
                    None,
                )
            }
            NbglPageContent::TagValueConfirm(data) => {
                let confirm = nbgl_contentTagValueConfirm_t {
                    tagValueList: data.tag_value_list,
                    detailsButtonToken: (FIRST_USER_TOKEN + 2) as u8,
                    tuneId: data.tune_id as u8,
                    confirmationText: data.confirmation_text.as_ptr() as *const c_char,
                    cancelText: data.cancel_text.as_ptr() as *const c_char,
                    confirmationToken: FIRST_USER_TOKEN as u8,
                    cancelToken: (FIRST_USER_TOKEN + 1) as u8,
                    ..Default::default()
                };
                (
                    nbgl_content_u {
                        tagValueConfirm: confirm,
                    },
                    TAG_VALUE_CONFIRM,
                    Some(action_callback),
                )
            }
            NbglPageContent::InfoLongPress(data) => {
                let long_press = nbgl_contentInfoLongPress_t {
                    text: data.text.as_ptr() as *const c_char,
                    icon: data
                        .icon
                        .as_ref()
                        .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
                    longPressText: data.long_press_text.as_ptr() as *const c_char,
                    longPressToken: FIRST_USER_TOKEN as u8,
                    tuneId: data.tune_id as u8,
                    ..Default::default()
                };
                (
                    nbgl_content_u {
                        infoLongPress: long_press,
                    },
                    INFO_LONG_PRESS,
                    Some(action_callback),
                )
            }
            NbglPageContent::InfoButton(data) => {
                let button = nbgl_contentInfoButton_t {
                    text: data.text.as_ptr() as *const c_char,
                    icon: data
                        .icon
                        .as_ref()
                        .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
                    buttonText: data.button_text.as_ptr() as *const c_char,
                    buttonToken: FIRST_USER_TOKEN as u8,
                    tuneId: data.tune_id as u8,
                    ..Default::default()
                };
                (
                    nbgl_content_u { infoButton: button },
                    INFO_BUTTON,
                    Some(action_callback),
                )
            }
            NbglPageContent::InfosList(data) => {
                let infos_list = nbgl_contentInfoList_t {
                    infoTypes: data.info_types_ptr.as_ptr() as *const *const c_char,
                    infoContents: data.info_contents_ptr.as_ptr() as *const *const c_char,
                    nbInfos: data.info_types_cstrings.len() as u8,
                    ..Default::default()
                };
                (
                    nbgl_content_u {
                        infosList: infos_list,
                    },
                    INFOS_LIST,
                    None,
                )
            }
        }
    }
}

/// A wrapper around the asynchronous NBGL nbgl_useCaseGenericReview C API binding.
/// Used to display custom built review screens. User can add different kind of
/// contents (CenteredInfo, InfoLongPress, InfoButton, TagValueList, TagValueConfirm, InfosList)
/// to the review screen using the add_content method.
pub struct NbglGenericReview {
    content_list: Vec<NbglPageContent>,
}

impl SyncNBGL for NbglGenericReview {}

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

    pub fn show(&self, reject_button_str: &str) -> bool {
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

            self.ux_sync_init();
            nbgl_useCaseGenericReview(
                &content_struct as *const nbgl_genericContents_t,
                reject_button_cstring.as_ptr() as *const c_char,
                Some(rejected_callback),
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
