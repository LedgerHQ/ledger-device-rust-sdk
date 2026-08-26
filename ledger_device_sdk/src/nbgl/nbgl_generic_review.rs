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
            icon: icon.map(|g| g.into()),
            on_top,
            style,
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            offset_y,
        }
    }
}

#[allow(clippy::needless_update)]
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
            icon: icon.map(|g| g.into()),
            long_press_text: CString::new(long_press_text).unwrap(),
            tune_id,
        }
    }
}

#[allow(clippy::needless_update)]
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
            icon: icon.map(|g| g.into()),
            button_text: CString::new(button_text).unwrap(),
            tune_id,
        }
    }
}

#[allow(clippy::needless_update)]
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
    /// Owns the C strings, and the extension structs the pairs point at.
    values: CTagValueList,
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
    /// * `_nb_max_lines_for_value` — Maximum number of lines allowed for each
    ///   value before truncation. (ignored, enforced to 0 when calling C function)
    /// * `small_case_for_value` — If `true`, values are rendered in a smaller
    ///   font. Note that NBGL overrides this to `false` on every page it draws
    ///   from a tag/value list (`nbgl_use_case.c:1144`), so it currently has no
    ///   effect.
    /// * `wrapping` — If `true`, long values are word-wrapped instead of
    ///   truncated. This one is honoured.
    ///
    /// `nbMaxLinesForValue` and `hideEndOfLastLine` are deliberately not
    /// offered: NBGL overwrites both with its own values just below
    /// `smallCaseForValue`, so an app setting them would see nothing change.
    /// Of the whole list-level struct only `wrapping` and `token` survive.
    pub fn new(
        tvl: &[Field],
        _nb_max_lines_for_value: u8,
        small_case_for_value: bool,
        wrapping: bool,
    ) -> TagValueList {
        TagValueList {
            values: CTagValueList::from_fields(tvl),
            small_case_for_value,
            wrapping,
        }
    }

    /// Creates a new [`TagValueList`] whose pairs may each carry a
    /// [`FieldExtension`].
    ///
    /// # Arguments
    ///
    /// * `values` — Slice of [`TagValue`] items.
    /// * `small_case_for_value` — If `true`, values are rendered in a smaller
    ///   font.
    /// * `wrapping` — If `true`, long values are word-wrapped instead of
    ///   truncated.
    pub fn new_ext(
        values: &[TagValue],
        small_case_for_value: bool,
        wrapping: bool,
    ) -> TagValueList {
        TagValueList {
            values: CTagValueList::new(values),
            small_case_for_value,
            wrapping,
        }
    }
}

#[allow(clippy::needless_update)]
impl From<&TagValueList> for nbgl_contentTagValueList_t {
    fn from(tvl: &TagValueList) -> nbgl_contentTagValueList_t {
        nbgl_contentTagValueList_t {
            pairs: tvl.values.pairs_ptr(),
            nbPairs: tvl.values.len(),
            nbMaxLinesForValue: 0,
            token: FIRST_USER_TOKEN as u8,
            smallCaseForValue: tvl.small_case_for_value,
            wrapping: tvl.wrapping,
            ..Default::default()
        }
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
            tune_id,
            confirmation_text: confirmation_text_cstring,
            cancel_text: cancel_text_cstring,
        }
    }
}

#[allow(clippy::needless_update)]
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
            info_types_cstrings,
            _info_contents_cstrings: info_contents_cstrings,
            info_types_ptr,
            info_contents_ptr,
        }
    }
}

#[allow(clippy::needless_update)]
impl From<&InfosList> for nbgl_contentInfoList_t {
    fn from(infos_list: &InfosList) -> nbgl_contentInfoList_t {
        nbgl_contentInfoList_t {
            infoTypes: infos_list.info_types_ptr.as_ptr(),
            infoContents: infos_list.info_contents_ptr.as_ptr(),
            nbInfos: infos_list.info_types_cstrings.len() as u8,
            ..Default::default()
        }
    }
}

unsafe extern "C" fn action_callback(token: c_int, _index: u8, _page: c_int) {
    unsafe {
        if token == FIRST_USER_TOKEN as i32 {
            G_RET = SyncNbgl::UxSyncRetApproved.into();
        } else if token == (FIRST_USER_TOKEN + 1) as i32 {
            G_RET = SyncNbgl::UxSyncRetRejected.into();
        }
        G_ENDED = true;
    }
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
/// A list of on/off switches, for a `SWITCHES_LIST` content.
///
/// The switches are drawn with their initial state; `NbglGenericReview` does
/// not itself route touches back to the app, so this is for display within a
/// review rather than for settings — see `NbglHomeAndSettings::settings` for
/// switches that persist.
pub struct SwitchesList {
    _texts: Vec<[CString; 2]>,
    switches: Vec<nbgl_contentSwitch_t>,
}

impl SwitchesList {
    /// Creates a new [`SwitchesList`].
    ///
    /// # Arguments
    ///
    /// * `switches` — One `(text, sub_text, initial_state)` per switch.
    pub fn new(switches: &[(&str, &str, bool)]) -> SwitchesList {
        let texts: Vec<[CString; 2]> = switches
            .iter()
            .map(|(text, sub_text, _)| {
                [
                    CString::new(*text).unwrap(),
                    CString::new(*sub_text).unwrap(),
                ]
            })
            .collect();

        let c_switches: Vec<nbgl_contentSwitch_t> = texts
            .iter()
            .zip(switches.iter())
            .enumerate()
            .map(|(i, (pair, (_, _, state)))| nbgl_contentSwitch_t {
                text: pair[0].as_ptr(),
                subText: pair[1].as_ptr(),
                initState: if *state { ON_STATE } else { OFF_STATE },
                token: (FIRST_USER_TOKEN + i as u32) as u8,
                ..Default::default()
            })
            .collect();

        SwitchesList {
            _texts: texts,
            switches: c_switches,
        }
    }
}

impl From<&SwitchesList> for nbgl_contentSwitchesList_t {
    fn from(list: &SwitchesList) -> nbgl_contentSwitchesList_t {
        nbgl_contentSwitchesList_t {
            switches: list.switches.as_ptr(),
            nbSwitches: list.switches.len() as u8,
        }
    }
}

/// A list of radio-button choices, for a `CHOICES_LIST` content.
pub struct ChoicesList {
    _names: Vec<CString>,
    names_ptr: Vec<*const c_char>,
    init_choice: u8,
}

impl ChoicesList {
    /// Creates a new [`ChoicesList`].
    ///
    /// # Arguments
    ///
    /// * `names` — The choices, in display order.
    /// * `init_choice` — Index of the choice selected when the page opens.
    pub fn new(names: &[&str], init_choice: u8) -> ChoicesList {
        let cnames: Vec<CString> = names.iter().map(|n| CString::new(*n).unwrap()).collect();
        let names_ptr: Vec<*const c_char> = cnames.iter().map(|n| n.as_ptr()).collect();
        ChoicesList {
            _names: cnames,
            names_ptr,
            init_choice,
        }
    }
}

#[allow(clippy::needless_update)]
impl From<&ChoicesList> for nbgl_contentRadioChoice_t {
    fn from(list: &ChoicesList) -> nbgl_contentRadioChoice_t {
        nbgl_contentRadioChoice_t {
            __bindgen_anon_1: nbgl_contentRadioChoice_t__bindgen_ty_1 {
                names: list.names_ptr.as_ptr(),
            },
            nbChoices: list.names_ptr.len() as u8,
            initChoice: list.init_choice,
            token: FIRST_USER_TOKEN as u8,
            ..Default::default()
        }
    }
}

/// A list of touchable bars, for a `BARS_LIST` content.
pub struct BarsList {
    _texts: Vec<CString>,
    texts_ptr: Vec<*const c_char>,
    tokens: Vec<u8>,
}

impl BarsList {
    /// Creates a new [`BarsList`].
    ///
    /// # Arguments
    ///
    /// * `texts` — Label of each bar, in display order.
    pub fn new(texts: &[&str]) -> BarsList {
        let ctexts: Vec<CString> = texts.iter().map(|t| CString::new(*t).unwrap()).collect();
        let texts_ptr: Vec<*const c_char> = ctexts.iter().map(|t| t.as_ptr()).collect();
        let tokens: Vec<u8> = (0..texts.len())
            .map(|i| (FIRST_USER_TOKEN + i as u32) as u8)
            .collect();
        BarsList {
            _texts: ctexts,
            texts_ptr,
            tokens,
        }
    }
}

#[allow(clippy::needless_update)]
impl From<&BarsList> for nbgl_contentBarsList_t {
    fn from(list: &BarsList) -> nbgl_contentBarsList_t {
        nbgl_contentBarsList_t {
            barTexts: list.texts_ptr.as_ptr(),
            tokens: list.tokens.as_ptr(),
            nbBars: list.texts_ptr.len() as u8,
            ..Default::default()
        }
    }
}

/// A centered info block with an optional tip box under it, for an
/// `EXTENDED_CENTER` content.
///
/// The tip box here is the inline `nbgl_contentTipBox_t` — text and icon only.
/// It is not the [`TipBox`] of a review's first page, which additionally
/// carries the modal opened on touch.
pub struct ExtendedCenter {
    center: CCenterInfo,
    tip_text: Option<CString>,
    tip_icon: Option<nbgl_icon_details_t>,
}

impl ExtendedCenter {
    /// Creates a new [`ExtendedCenter`].
    ///
    /// # Arguments
    ///
    /// * `center` — The centered icon-and-text block.
    /// * `tip_text` — Label of the tip box drawn under it, if any.
    /// * `tip_icon` — Icon of that tip box.
    pub fn new(
        center: &CenterInfo,
        tip_text: Option<&str>,
        tip_icon: Option<&NbglGlyph>,
    ) -> ExtendedCenter {
        ExtendedCenter {
            center: CCenterInfo::new(center),
            tip_text: tip_text.map(|t| CString::new(t).unwrap()),
            tip_icon: tip_icon.map(|g| g.into()),
        }
    }
}

#[allow(clippy::needless_update)]
impl From<&ExtendedCenter> for nbgl_contentExtendedCenter_t {
    fn from(center: &ExtendedCenter) -> nbgl_contentExtendedCenter_t {
        nbgl_contentExtendedCenter_t {
            contentCenter: center.center.as_c_type(),
            tipBox: nbgl_contentTipBox_t {
                text: match &center.tip_text {
                    Some(text) => text.as_ptr(),
                    None => core::ptr::null(),
                },
                icon: match &center.tip_icon {
                    Some(icon) => icon as *const nbgl_icon_details_t,
                    None => core::ptr::null(),
                },
                token: FIRST_USER_TOKEN as u8,
                ..Default::default()
            },
        }
    }
}

/// | `TagValueConfirm` | [`TagValueConfirm`] | Field review with confirm/cancel |
/// | `InfosList` | [`InfosList`] | Read-only info list |
/// | `SwitchesList` | [`SwitchesList`] | On/off switch list |
/// | `ChoicesList` | [`ChoicesList`] | Radio-button choices |
/// | `BarsList` | [`BarsList`] | Touchable bars |
/// | `ExtendedCenter` | [`ExtendedCenter`] | Centered info with a tip box |
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
    /// List of on/off switches.
    SwitchesList(SwitchesList),
    /// Radio-button choices.
    ChoicesList(ChoicesList),
    /// List of touchable bars.
    BarsList(BarsList),
    /// Centered info with an optional tip box.
    ExtendedCenter(ExtendedCenter),
}

/// App function to run when a control inside a generic content is touched.
///
/// `nbgl_contentActionCallback_t` carries no user data, so the app's function is
/// parked here for [`content_action_callback`] to find. Only one such flow is on
/// screen at a time, so a single slot is enough for every container that emits
/// these contents.
static mut CONTENT_ACTION: Option<fn(u8, u8)> = None;

/// Records the app function the next flow should report control touches to.
pub(crate) fn set_content_action(on_action: Option<fn(u8, u8)>) {
    unsafe {
        CONTENT_ACTION = on_action;
    }
}

/// Callback registered with NBGL for content whose controls the app can act on.
///
/// The page index NBGL also supplies is dropped: the token already identifies
/// the element, each content type assigning `FIRST_USER_TOKEN + i` to its `i`th.
pub(crate) unsafe extern "C" fn content_action_callback(token: c_int, index: u8, _page: c_int) {
    if let Some(on_action) = unsafe { CONTENT_ACTION } {
        on_action(token as u8, index);
    }
}

impl From<&NbglPageContent> for nbgl_content_t {
    /// Builds the C content with no action callback on the interactive lists.
    /// Use [`NbglPageContent::to_c_content`] to route their controls to the app.
    fn from(content: &NbglPageContent) -> nbgl_content_t {
        content.to_c_content(None)
    }
}

impl NbglPageContent {
    /// Builds the C content, reporting control touches through `on_action`.
    ///
    /// `TagValueConfirm`, `InfoLongPress` and `InfoButton` keep the SDK's own
    /// callback regardless: that is how a review detects approval, and
    /// overriding it would break the flow's result.
    pub(crate) fn to_c_content(&self, on_action: nbgl_contentActionCallback_t) -> nbgl_content_t {
        let content = self;
        match content {
            NbglPageContent::CenteredInfo(data) => nbgl_content_t {
                content: nbgl_content_u {
                    centeredInfo: data.into(),
                },
                type_: CENTERED_INFO,
                contentActionCallback: on_action,
            },
            NbglPageContent::TagValueList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    tagValueList: data.into(),
                },
                type_: TAG_VALUE_LIST,
                contentActionCallback: on_action,
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
                contentActionCallback: on_action,
            },
            NbglPageContent::SwitchesList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    switchesList: data.into(),
                },
                type_: SWITCHES_LIST,
                contentActionCallback: on_action,
            },
            NbglPageContent::ChoicesList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    choicesList: data.into(),
                },
                type_: CHOICES_LIST,
                contentActionCallback: on_action,
            },
            NbglPageContent::BarsList(data) => nbgl_content_t {
                content: nbgl_content_u {
                    barsList: data.into(),
                },
                type_: BARS_LIST,
                contentActionCallback: on_action,
            },
            NbglPageContent::ExtendedCenter(data) => nbgl_content_t {
                content: nbgl_content_u {
                    extendedCenter: data.into(),
                },
                type_: EXTENDED_CENTER,
                contentActionCallback: on_action,
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
    on_action: Option<fn(u8, u8)>,
}

impl SyncNBGL for NbglGenericReview {}

impl Default for NbglGenericReview {
    fn default() -> Self {
        Self::new()
    }
}

impl NbglGenericReview {
    /// Creates an empty [`NbglGenericReview`] with no content pages.
    pub fn new() -> NbglGenericReview {
        NbglGenericReview {
            content_list: Vec::new(),
            on_action: None,
        }
    }

    /// Sets the function run when a control inside one of the contents is
    /// touched — a switch, a choice, or a bar.
    ///
    /// It receives the token of the touched element and, for a choices list,
    /// the index chosen. Each content type assigns `FIRST_USER_TOKEN + i` to
    /// its `i`th element, except `ChoicesList`, which uses one token and
    /// reports the selection in `index`.
    ///
    /// Without this the interactive lists are drawn but report nothing.
    pub fn on_action(mut self, on_action: fn(u8, u8)) -> NbglGenericReview {
        self.on_action = Some(on_action);
        self
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
        let on_action = self.action_callback();
        self.content_list
            .iter()
            .map(|content| content.to_c_content(on_action))
            .collect()
    }

    /// The C callback to hand the contents, or None if the app set no handler.
    fn action_callback(&self) -> nbgl_contentActionCallback_t {
        set_content_action(self.on_action);
        match self.on_action {
            Some(_) => Some(content_action_callback),
            None => None,
        }
    }

    fn show_internal(&self, reject_button_str: &str) -> bool {
        unsafe {
            let c_content_list: Vec<nbgl_content_t> = self.to_c_content_list();

            let content_struct = nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: c_content_list.as_ptr(),
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
            matches!(sync_ret, SyncNbgl::UxSyncRetApproved)
        }
    }

    /// Displays the review to the user and blocks until a decision is made.
    ///
    /// A reject button labelled with `reject_button_str` is shown on the
    /// final page. The method returns `true` if the user approved the review
    /// and `false` if they rejected it.
    ///
    /// # Arguments
    ///
    /// * `_comm` - Mutable reference to Comm.
    /// * `reject_button_str` — Text for the reject/cancel button displayed
    ///   at the end of the review flow (e.g. `"Reject transaction"`).
    #[cfg(feature = "io_new")]
    pub fn show<const N: usize>(
        &self,
        _comm: &mut crate::io::Comm<N>,
        reject_button_str: &str,
    ) -> bool {
        self.show_internal(reject_button_str)
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
    #[cfg(not(feature = "io_new"))]
    pub fn show(&self, reject_button_str: &str) -> bool {
        self.show_internal(reject_button_str)
    }
}
