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

impl From<&CenteredInfo> for nbgl_contentCenteredInfo_t {
    fn from(info: &CenteredInfo) -> nbgl_contentCenteredInfo_t {
        nbgl_contentCenteredInfo_t {
            text1: info.text1.as_ptr() as *const c_char,
            text2: info.text2.as_ptr() as *const c_char,
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            text3: info.text3.as_ptr() as *const c_char,
            icon: info
                .icon
                .as_ref()
                .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
            onTop: info.on_top,
            style: info.style.into(),
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            offsetY: info.offset_y,
            ..Default::default()
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

impl From<&InfoLongPress> for nbgl_contentInfoLongPress_t {
    fn from(info: &InfoLongPress) -> nbgl_contentInfoLongPress_t {
        nbgl_contentInfoLongPress_t {
            text: info.text.as_ptr() as *const c_char,
            icon: info
                .icon
                .as_ref()
                .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
            longPressText: info.long_press_text.as_ptr() as *const c_char,
            longPressToken: FIRST_USER_TOKEN as u8,
            tuneId: info.tune_id as u8,
            ..Default::default()
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

impl From<&InfoButton> for nbgl_contentInfoButton_t {
    fn from(info: &InfoButton) -> nbgl_contentInfoButton_t {
        nbgl_contentInfoButton_t {
            text: info.text.as_ptr() as *const c_char,
            icon: info
                .icon
                .as_ref()
                .map_or(core::ptr::null(), |icon| icon as *const nbgl_icon_details_t),
            buttonText: info.button_text.as_ptr() as *const c_char,
            buttonToken: FIRST_USER_TOKEN as u8,
            tuneId: info.tune_id as u8,
            ..Default::default()
        }
    }
}

/// A list of tag/value pairs for use with [`NbglGenericReview`].
///
/// Each pair is rendered as a labelled field (tag on the left, value on the
/// right). Display options control the maximum number of lines per value,
/// text casing, and word-wrapping behaviour.
pub struct TagValueList {
    _cfields: Vec<CField>,
    /// Vector of C-compatible strings representing the tag/value pairs.
    pairs: Vec<nbgl_contentTagValue_t>,
    /// Maximum number of lines allowed for each value before truncation.
    nb_max_lines_for_value: u8,
    /// If `true`, values are rendered in a smaller font.
    small_case_for_value: bool,
    /// If `true`, long values are word-wrapped instead of truncated.
    wrapping: bool,
}

impl TagValueList {
    /// Creates a new [`TagValueList`].
    ///
    /// # Arguments
    ///
    /// * `tvl` — Slice of [`Field`] items, each containing a `name` (tag)
    ///   and a `value`.
    /// * `nb_max_lines_for_value` — Maximum number of lines allowed for each
    ///   value before truncation.
    /// * `small_case_for_value` — If `true`, values are rendered in a smaller
    ///   font.
    /// * `wrapping` — If `true`, long values are word-wrapped instead of
    ///   truncated.
    pub fn new(
        tvl: &[Field],
        nb_max_lines_for_value: u8,
        small_case_for_value: bool,
        wrapping: bool,
    ) -> TagValueList {
        let cfields: Vec<CField> = tvl.iter().map(|field| field.into()).collect();
        let pairs: Vec<nbgl_contentTagValue_t> = cfields.iter().map(|pair| pair.into()).collect();
        TagValueList {
            _cfields: cfields,
            pairs: pairs,
            nb_max_lines_for_value,
            small_case_for_value,
            wrapping,
        }
    }
}

impl From<&TagValueList> for nbgl_contentTagValueList_t {
    fn from(tvl: &TagValueList) -> nbgl_contentTagValueList_t {
        let nbgl_content_tvl = nbgl_contentTagValueList_t {
            pairs: tvl.pairs.as_ptr() as *const nbgl_contentTagValue_t,
            nbPairs: tvl.pairs.len() as u8,
            nbMaxLinesForValue: tvl.nb_max_lines_for_value,
            token: FIRST_USER_TOKEN as u8,
            smallCaseForValue: tvl.small_case_for_value,
            wrapping: tvl.wrapping,
            ..Default::default()
        };
        nbgl_content_tvl
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

impl From<&TagValueConfirm> for nbgl_contentTagValueConfirm_t {
    fn from(tvc: &TagValueConfirm) -> nbgl_contentTagValueConfirm_t {
        nbgl_contentTagValueConfirm_t {
            tagValueList: tvc.tag_value_list,
            detailsButtonToken: (FIRST_USER_TOKEN + 2) as u8,
            tuneId: tvc.tune_id as u8,
            confirmationText: tvc.confirmation_text.as_ptr() as *const c_char,
            cancelText: tvc.cancel_text.as_ptr() as *const c_char,
            confirmationToken: FIRST_USER_TOKEN as u8,
            cancelToken: (FIRST_USER_TOKEN + 1) as u8,
            ..Default::default()
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

impl From<&InfosList> for nbgl_contentInfoList_t {
    fn from(infos_list: &InfosList) -> nbgl_contentInfoList_t {
        nbgl_contentInfoList_t {
            infoTypes: infos_list.info_types_ptr.as_ptr() as *const *const c_char,
            infoContents: infos_list.info_contents_ptr.as_ptr() as *const *const c_char,
            nbInfos: infos_list.info_types_cstrings.len() as u8,
            ..Default::default()
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

impl From<&NbglPageContent> for nbgl_content_t {
    fn from(content: &NbglPageContent) -> nbgl_content_t {
        match content {
            NbglPageContent::CenteredInfo(data) => nbgl_content_t {
                content: nbgl_content_u {
                    centeredInfo: data.into(),
                },
                type_: CENTERED_INFO,
                contentActionCallback: None,
            },
            NbglPageContent::TagValueList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    tagValueList: data.into(),
                },
                type_: TAG_VALUE_LIST,
                contentActionCallback: None,
            },
            NbglPageContent::TagValueConfirm(data) => nbgl_content_t {
                content: nbgl_content_u {
                    tagValueConfirm: data.into(),
                },
                type_: TAG_VALUE_CONFIRM,
                contentActionCallback: Some(action_callback),
            },
            NbglPageContent::InfoLongPress(data) => nbgl_content_t {
                content: nbgl_content_u {
                    infoLongPress: data.into(),
                },
                type_: INFO_LONG_PRESS,
                contentActionCallback: Some(action_callback),
            },
            NbglPageContent::InfoButton(data) => nbgl_content_t {
                content: nbgl_content_u {
                    infoButton: data.into(),
                },
                type_: INFO_BUTTON,
                contentActionCallback: Some(action_callback),
            },
            NbglPageContent::InfosList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    infosList: data.into(),
                },
                type_: INFOS_LIST,
                contentActionCallback: None,
            },
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
            .map(|content| content.into())
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
