use super::*;

/// Visual style for a [`CenteredInfo`] content element.
///
/// The available variants differ by device family:
///
/// **Stax / Flex / Apex P:**
/// - [`LargeCaseInfo`](CenteredInfoStyle::LargeCaseInfo) — large-case text.
/// - [`LargeCaseBoldInfo`](CenteredInfoStyle::LargeCaseBoldInfo) — large-case bold text.
/// - [`NormalInfo`](CenteredInfoStyle::NormalInfo) — normal (default) text.
/// - [`PluginInfo`](CenteredInfoStyle::PluginInfo) — plugin-oriented layout.
///
/// **Nano S+ / Nano X:**
/// - [`RegularInfo`](CenteredInfoStyle::RegularInfo) — regular text.
/// - [`BoldText1Info`](CenteredInfoStyle::BoldText1Info) — bold primary text.
/// - [`ButtonInfo`](CenteredInfoStyle::ButtonInfo) — button-style text.
#[derive(Copy, Clone)]
pub enum CenteredInfoStyle {
    /// Large-case text style (Stax / Flex / Apex P only).
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    LargeCaseInfo = 0,
    /// Large-case bold text style (Stax / Flex / Apex P only).
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    LargeCaseBoldInfo,
    /// Normal text style (Stax / Flex / Apex P only).
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    NormalInfo,
    /// Plugin-oriented layout style (Stax / Flex / Apex P only).
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    PluginInfo,
    /// Regular text style (Nano S+ / Nano X only).
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    RegularInfo = 0,
    /// Bold primary text style (Nano S+ / Nano X only).
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    BoldText1Info,
    /// Button-style text (Nano S+ / Nano X only).
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    ButtonInfo,
}

impl From<CenteredInfoStyle> for nbgl_contentCenteredInfoStyle_t {
    fn from(style: CenteredInfoStyle) -> nbgl_contentCenteredInfoStyle_t {
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
        match style {
            CenteredInfoStyle::LargeCaseInfo => LARGE_CASE_INFO,
            CenteredInfoStyle::LargeCaseBoldInfo => LARGE_CASE_BOLD_INFO,
            CenteredInfoStyle::NormalInfo => NORMAL_INFO,
            CenteredInfoStyle::PluginInfo => PLUGIN_INFO,
        }
        #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
        match style {
            CenteredInfoStyle::RegularInfo => REGULAR_INFO,
            CenteredInfoStyle::BoldText1Info => BOLD_TEXT1_INFO,
            CenteredInfoStyle::ButtonInfo => BUTTON_INFO,
        }
    }
}

/// A centered information screen for use with [`NbglGenericReview`].
///
/// Displays up to two (Nano) or three (Stax/Flex/Apex P) lines of text
/// with an optional icon, positioned either at the top of the page or
/// vertically centered. The visual appearance is controlled by a
/// [`CenteredInfoStyle`].
///
/// On Stax / Flex / Apex P an additional `offset_y` parameter allows
/// fine-tuning the vertical position of the content.
pub struct CenteredInfo {
    text1: CString,
    text2: CString,
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    text3: CString,
    icon: Option<nbgl_icon_details_t>,
    on_top: bool,
    style: CenteredInfoStyle,
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    offset_y: i16,
}

impl CenteredInfo {
    /// Creates a new [`CenteredInfo`].
    ///
    /// # Arguments
    ///
    /// * `text1` — Primary text line.
    /// * `text2` — Secondary text line.
    /// * `text3` — *(Stax / Flex / Apex P only)* Tertiary text line.
    /// * `icon` — Optional glyph displayed alongside the text.
    /// * `on_top` — If `true`, the content is pinned to the top of the page;
    ///   otherwise it is vertically centered.
    /// * `style` — The [`CenteredInfoStyle`] that controls the visual layout.
    /// * `offset_y` — *(Stax / Flex / Apex P only)* Vertical pixel offset
    ///   applied to the content.
    pub fn new(
        text1: &str,
        text2: &str,
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))] text3: &str,
        icon: Option<&NbglGlyph>,
        on_top: bool,
        style: CenteredInfoStyle,
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))] offset_y: i16,
    ) -> CenteredInfo {
        CenteredInfo {
            text1: CString::new(text1).unwrap(),
            text2: CString::new(text2).unwrap(),
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            text3: CString::new(text3).unwrap(),
            icon: icon.map_or(None, |g| Some(g.into())),
            on_top: on_top,
            style: style,
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            offset_y: offset_y,
        }
    }
}

/// A confirmation screen with a "long press" button for use with
/// [`NbglGenericReview`].
///
/// The user must press and hold the button to confirm, which helps prevent
/// accidental approvals. An optional icon and descriptive text are shown
/// above the button.
pub struct InfoLongPress {
    text: CString,
    icon: Option<nbgl_icon_details_t>,
    long_press_text: CString,
    tune_id: TuneIndex,
}

impl InfoLongPress {
    /// Creates a new [`InfoLongPress`].
    ///
    /// # Arguments
    ///
    /// * `text` — Descriptive text displayed above the button.
    /// * `icon` — Optional glyph displayed alongside the text.
    /// * `long_press_text` — Label shown on the long-press button itself.
    /// * `tune_id` — [`TuneIndex`] of the sound played on button activation.
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

/// A confirmation screen with an action button for use with
/// [`NbglGenericReview`].
///
/// Similar to [`InfoLongPress`] but uses a regular tap button instead of a
/// long-press gesture. An optional icon and descriptive text are shown
/// above the button.
pub struct InfoButton {
    text: CString,
    icon: Option<nbgl_icon_details_t>,
    button_text: CString,
    tune_id: TuneIndex,
}

impl InfoButton {
    /// Creates a new [`InfoButton`].
    ///
    /// # Arguments
    ///
    /// * `text` — Descriptive text displayed above the button.
    /// * `icon` — Optional glyph displayed alongside the text.
    /// * `button_text` — Label shown on the button.
    /// * `tune_id` — [`TuneIndex`] of the sound played on button activation.
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

/// A list of tag/value pairs for use with [`NbglGenericReview`].
///
/// Each pair is rendered as a labelled field (tag on the left, value on the
/// right). Display options control the maximum number of lines per value,
/// text casing, and word-wrapping behaviour.
pub struct TagValueList {
    pairs: Vec<nbgl_contentTagValue_t>,
    _items: Vec<CString>,
    _values: Vec<CString>,
    nb_max_lines_for_value: u8,
    small_case_for_value: bool,
    wrapping: bool,
}

impl TagValueList {
    /// Creates a new [`TagValueList`].
    ///
    /// # Arguments
    ///
    /// * `pairs` — Slice of [`Field`] items, each containing a `name` (tag)
    ///   and a `value`.
    /// * `nb_max_lines_for_value` — Maximum number of lines allowed for each
    ///   value before truncation.
    /// * `small_case_for_value` — If `true`, values are rendered in a smaller
    ///   font.
    /// * `wrapping` — If `true`, long values are word-wrapped instead of
    ///   truncated.
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

/// A tag/value list combined with confirm and cancel buttons for use with
/// [`NbglGenericReview`].
///
/// This is a convenience wrapper that pairs a [`TagValueList`] with two
/// action buttons (confirm / cancel) so the user can review a set of
/// fields and then approve or reject in a single content element.
pub struct TagValueConfirm {
    tag_value_list: nbgl_contentTagValueList_t,
    tune_id: TuneIndex,
    confirmation_text: CString,
    cancel_text: CString,
}

impl TagValueConfirm {
    /// Creates a new [`TagValueConfirm`].
    ///
    /// # Arguments
    ///
    /// * `tag_value_list` — Reference to a previously constructed
    ///   [`TagValueList`] containing the fields to display.
    /// * `tune_id` — [`TuneIndex`] of the sound played on confirmation.
    /// * `confirmation_text` — Label for the confirm button
    ///   (e.g. `"Approve"`).
    /// * `cancel_text` — Label for the cancel button (e.g. `"Reject"`).
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

/// A read-only list of information fields for use with
/// [`NbglGenericReview`].
///
/// Unlike [`TagValueList`], this variant has no display-tuning options
/// and is intended for simple informational screens (e.g. app version,
/// developer name) rather than transaction review data.
pub struct InfosList {
    info_types_cstrings: Vec<CString>,
    _info_contents_cstrings: Vec<CString>,
    info_types_ptr: Vec<*const c_char>,
    info_contents_ptr: Vec<*const c_char>,
}

impl InfosList {
    /// Creates a new [`InfosList`].
    ///
    /// # Arguments
    ///
    /// * `infos` — Slice of [`Field`] items. Each field's `name` is used as
    ///   the label and `value` as the corresponding content.
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

/// Content element that can be added to an [`NbglGenericReview`] via
/// [`NbglGenericReview::add_content`].
///
/// Each variant wraps one of the dedicated content structs exposed by
/// this module:
///
/// | Variant | Underlying type | Typical use |
/// |---|---|---|
/// | `CenteredInfo` | [`CenteredInfo`] | Static informational screen |
/// | `InfoLongPress` | [`InfoLongPress`] | Long-press confirmation |
/// | `InfoButton` | [`InfoButton`] | Tap-button confirmation |
/// | `TagValueList` | [`TagValueList`] | Field review (no buttons) |
/// | `TagValueConfirm` | [`TagValueConfirm`] | Field review with confirm/cancel |
/// | `InfosList` | [`InfosList`] | Read-only info list |
pub enum NbglPageContent {
    /// Centered information screen.
    CenteredInfo(CenteredInfo),
    /// Long-press confirmation screen.
    InfoLongPress(InfoLongPress),
    /// Tap-button confirmation screen.
    InfoButton(InfoButton),
    /// Tag/value pair list without action buttons.
    TagValueList(TagValueList),
    /// Tag/value pair list with confirm and cancel buttons.
    TagValueConfirm(TagValueConfirm),
    /// Read-only information list.
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
                    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                    text3: data.text3.as_ptr() as *const c_char,
                    icon: data
                        .icon
                        .as_ref()
                        .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
                    onTop: data.on_top,
                    style: data.style.into(),
                    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
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

/// Builder for a multi-page generic review screen backed by the NBGL
/// `nbgl_useCaseGenericReview` C API.
///
/// Use this when you need full control over the pages shown during a
/// review flow. Content elements are added one by one with
/// [`add_content`](NbglGenericReview::add_content) and then presented
/// to the user via [`show`](NbglGenericReview::show).
///
/// # Example
///
/// ```rust,ignore
/// let approved = NbglGenericReview::new()
///     .add_content(NbglPageContent::TagValueConfirm(
///         TagValueConfirm::new(&fields, TuneIndex::TapCasual, "Approve", "Reject"),
///     ))
///     .show("Reject transaction");
/// ```
pub struct NbglGenericReview {
    content_list: Vec<NbglPageContent>,
}

impl SyncNBGL for NbglGenericReview {}

impl NbglGenericReview {
    /// Creates an empty [`NbglGenericReview`] with no content pages.
    pub fn new() -> NbglGenericReview {
        NbglGenericReview {
            content_list: Vec::new(),
        }
    }

    /// Appends a content page to the review.
    ///
    /// This method consumes and returns `self` so that calls can be chained:
    ///
    /// ```rust,ignore
    /// let review = NbglGenericReview::new()
    ///     .add_content(NbglPageContent::CenteredInfo(info))
    ///     .add_content(NbglPageContent::TagValueList(fields));
    /// ```
    pub fn add_content(mut self, content: NbglPageContent) -> NbglGenericReview {
        self.content_list.push(content);
        self
    }

    /// Converts the Rust content list into the C representation expected by
    /// the NBGL library.
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

    /// Displays the review to the user and blocks until a decision is made.
    ///
    /// A reject button labelled with `reject_button_str` is shown on the
    /// final page. The method returns `true` if the user approved the review
    /// and `false` if they rejected it.
    ///
    /// # Arguments
    ///
    /// * `reject_button_str` — Text for the reject/cancel button displayed
    ///   at the end of the review flow (e.g. `"Reject transaction"`).
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
