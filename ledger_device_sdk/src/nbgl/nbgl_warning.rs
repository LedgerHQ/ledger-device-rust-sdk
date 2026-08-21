//! Warning pages shown before, and reachable from, an advanced review.
//!
//! [`NbglWarning`] wraps `nbgl_warning_t`. There are two ways to configure it,
//! and they are alternatives rather than complements:
//!
//! * **Pre-defined** — [`NbglWarning::predefined`] with one or more
//!   [`WarningType`]s. NBGL builds the intro page and both detail modals
//!   itself, using the provider strings for wording.
//! * **Manual** — [`NbglWarning::info`], [`NbglWarning::intro_details`] and
//!   [`NbglWarning::review_details`] describe the pages yourself, for warnings
//!   the pre-defined set does not cover. Pair `intro_details` with
//!   [`NbglWarning::intro_top_right_icon`]: NBGL draws the button that opens
//!   them only when that icon is set, so without it the details are
//!   unreachable.
//!
//! Either way [`NbglWarning::prelude`] can prepend a page before the review.
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::{NbglWarning, WarningType};
//! // Blind signing plus a Web3 Checks risk, with the report attributed.
//! NbglWarning::new()
//!     .predefined(&[WarningType::BlindSigning, WarningType::W3cRiskDetected])
//!     .dapp_provider("Uniswap")
//!     .report_provider("Blockaid")
//!     .report_url("https://report.example/tx/0x1234");
//! ```

use super::*;
use alloc::boxed::Box;

/// A pre-defined warning kind, mapped to `nbgl_warningType_t`.
///
/// Several may apply at once — they form a bitfield in the C API, which
/// [`NbglWarning::predefined`] takes as a slice.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WarningType {
    /// Web3 Checks could not run.
    W3cIssue,
    /// Web3 Checks flagged a risk. See [`NbglWarning::report_url`].
    W3cRiskDetected,
    /// Web3 Checks flagged a threat. See [`NbglWarning::report_url`].
    W3cThreatDetected,
    /// Web3 Checks ran and found no threat.
    W3cNoThreat,
    /// The transaction cannot be decoded and is signed blind.
    BlindSigning,
    /// Signing is gated.
    GatedSigning,
}

impl WarningType {
    fn to_c_type(self) -> nbgl_warningType_t {
        match self {
            WarningType::W3cIssue => W3C_ISSUE_WARN,
            WarningType::W3cRiskDetected => W3C_RISK_DETECTED_WARN,
            WarningType::W3cThreatDetected => W3C_THREAT_DETECTED_WARN,
            WarningType::W3cNoThreat => W3C_NO_THREAT_WARN,
            WarningType::BlindSigning => BLIND_SIGNING_WARN,
            WarningType::GatedSigning => GATED_SIGNING_WARN,
        }
    }

    /// This warning's bit in the `predefinedSet` bitfield.
    fn bit(self) -> u32 {
        1u32 << self.to_c_type()
    }
}

/// A vertically centered block of icon and text, mapped to
/// `nbgl_contentCenter_t`.
///
/// Every field is optional; those left unset are not drawn.
#[derive(Default)]
pub struct CenterInfo<'a> {
    icon: Option<&'a NbglGlyph<'a>>,
    title: Option<&'a str>,
    small_title: Option<&'a str>,
    description: Option<&'a str>,
    sub_text: Option<&'a str>,
    icon_hug: u16,
    padding: bool,
}

impl<'a> CenterInfo<'a> {
    pub fn new() -> CenterInfo<'a> {
        CenterInfo::default()
    }

    /// Icon drawn above the text.
    pub fn icon(self, icon: &'a NbglGlyph<'a>) -> CenterInfo<'a> {
        CenterInfo {
            icon: Some(icon),
            ..self
        }
    }

    /// Title, in large black type.
    pub fn title(self, title: &'a str) -> CenterInfo<'a> {
        CenterInfo {
            title: Some(title),
            ..self
        }
    }

    /// Sub-title, in small bold black type.
    pub fn small_title(self, small_title: &'a str) -> CenterInfo<'a> {
        CenterInfo {
            small_title: Some(small_title),
            ..self
        }
    }

    /// Description, in small regular black type.
    pub fn description(self, description: &'a str) -> CenterInfo<'a> {
        CenterInfo {
            description: Some(description),
            ..self
        }
    }

    /// Trailing text, in small regular dark gray type.
    pub fn sub_text(self, sub_text: &'a str) -> CenterInfo<'a> {
        CenterInfo {
            sub_text: Some(sub_text),
            ..self
        }
    }

    /// Vertical margin applied above and below the icon.
    pub fn icon_hug(self, icon_hug: u16) -> CenterInfo<'a> {
        CenterInfo { icon_hug, ..self }
    }

    /// Adds a 40px padding at the bottom.
    pub fn padding(self, padding: bool) -> CenterInfo<'a> {
        CenterInfo { padding, ..self }
    }
}

/// A QR code page, mapped to `nbgl_layoutQRCode_t`.
///
/// Only available on touchscreen devices: `NBGL_QRCODE` is defined for Stax,
/// Flex and Apex, so the Nano bindings have neither the struct nor the union
/// member.
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub struct QrCode<'a> {
    url: &'a str,
    text1: Option<&'a str>,
    text2: Option<&'a str>,
    offset_y: i16,
    centered: bool,
    large_text1: bool,
}

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
impl<'a> QrCode<'a> {
    /// Creates a QR code encoding `url`, centered by default.
    pub fn new(url: &'a str) -> QrCode<'a> {
        QrCode {
            url,
            text1: None,
            text2: None,
            offset_y: 0,
            centered: true,
            large_text1: false,
        }
    }

    /// First caption, drawn in bold under the code.
    pub fn text1(self, text1: &'a str) -> QrCode<'a> {
        QrCode {
            text1: Some(text1),
            ..self
        }
    }

    /// Second caption, drawn in regular type under the first.
    pub fn text2(self, text2: &'a str) -> QrCode<'a> {
        QrCode {
            text2: Some(text2),
            ..self
        }
    }

    /// Vertical shift; positive values move the block down.
    pub fn offset_y(self, offset_y: i16) -> QrCode<'a> {
        QrCode { offset_y, ..self }
    }

    /// Whether to center the block vertically.
    pub fn centered(self, centered: bool) -> QrCode<'a> {
        QrCode { centered, ..self }
    }

    /// Draws the first caption in a 32px font.
    pub fn large_text1(self, large_text1: bool) -> QrCode<'a> {
        QrCode {
            large_text1,
            ..self
        }
    }
}

/// One touchable bar in a [`WarningDetails::BarList`].
pub struct WarningBar<'a> {
    /// Bar label, in bold.
    pub text: &'a str,
    /// Secondary label under `text`.
    pub sub_text: Option<&'a str>,
    /// Icon shown at the start of the bar.
    pub icon: Option<&'a NbglGlyph<'a>>,
    /// Page opened when the bar is touched. `None` makes the bar inert.
    pub details: Option<&'a WarningDetails<'a>>,
}

/// A page reachable from the top-right button of a warning or review,
/// mapped to `nbgl_warningDetails_t` (`nbgl_genericDetails_t`).
pub enum WarningDetails<'a> {
    /// A centered icon-and-text page.
    CenteredInfo {
        /// Page title, also used to navigate back.
        title: &'a str,
        /// Page body.
        info: CenterInfo<'a>,
    },
    /// A QR code page. Touchscreen devices only.
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    QrCode {
        /// Page title, also used to navigate back.
        title: &'a str,
        /// The code and its captions.
        qr: QrCode<'a>,
    },
    /// A list of touchable bars, each opening a sub-page.
    BarList {
        /// Page title, also used to navigate back.
        title: &'a str,
        /// The bars, in display order.
        bars: &'a [WarningBar<'a>],
    },
}

/// A page shown before the review starts, mapped to `nbgl_preludeDetails_t`.
///
/// On Nano only `title` is used.
#[derive(Default)]
pub struct Prelude<'a> {
    icon: Option<&'a NbglGlyph<'a>>,
    title: Option<&'a str>,
    description: Option<&'a str>,
    button_text: Option<&'a str>,
    footer_text: Option<&'a str>,
    details: Option<&'a WarningDetails<'a>>,
}

impl<'a> Prelude<'a> {
    pub fn new() -> Prelude<'a> {
        Prelude::default()
    }

    /// Icon of the centered info.
    pub fn icon(self, icon: &'a NbglGlyph<'a>) -> Prelude<'a> {
        Prelude {
            icon: Some(icon),
            ..self
        }
    }

    /// Title of the centered info. The only field honoured on Nano.
    pub fn title(self, title: &'a str) -> Prelude<'a> {
        Prelude {
            title: Some(title),
            ..self
        }
    }

    /// Sub-text of the centered info.
    pub fn description(self, description: &'a str) -> Prelude<'a> {
        Prelude {
            description: Some(description),
            ..self
        }
    }

    /// Label of the black button that opens [`Self::details`].
    pub fn button_text(self, button_text: &'a str) -> Prelude<'a> {
        Prelude {
            button_text: Some(button_text),
            ..self
        }
    }

    /// Label of the footer that continues on to the review.
    pub fn footer_text(self, footer_text: &'a str) -> Prelude<'a> {
        Prelude {
            footer_text: Some(footer_text),
            ..self
        }
    }

    /// Page opened by the button.
    pub fn details(self, details: &'a WarningDetails<'a>) -> Prelude<'a> {
        Prelude {
            details: Some(details),
            ..self
        }
    }
}

/// The warning configuration handed to an advanced review, mapped to
/// `nbgl_warning_t`.
///
/// See the [module documentation](self) for how the pre-defined and manual
/// paths differ.
#[derive(Default)]
pub struct NbglWarning<'a> {
    predefined: u32,
    dapp_provider: Option<&'a str>,
    report_url: Option<&'a str>,
    report_provider: Option<&'a str>,
    provider_message: Option<&'a str>,
    intro_details: Option<&'a WarningDetails<'a>>,
    review_details: Option<&'a WarningDetails<'a>>,
    info: Option<&'a CenterInfo<'a>>,
    intro_top_right_icon: Option<&'a NbglGlyph<'a>>,
    review_top_right_icon: Option<&'a NbglGlyph<'a>>,
    prelude: Option<&'a Prelude<'a>>,
}

impl<'a> NbglWarning<'a> {
    pub fn new() -> NbglWarning<'a> {
        NbglWarning::default()
    }

    /// Sets the pre-defined warnings to raise. NBGL builds the intro page and
    /// the detail modals itself; the provider setters below fill in the
    /// wording.
    ///
    /// Replaces any previously set list.
    pub fn predefined(self, warnings: &[WarningType]) -> NbglWarning<'a> {
        NbglWarning {
            predefined: warnings.iter().fold(0u32, |set, w| set | w.bit()),
            ..self
        }
    }

    /// Name of the dApp provider, used in some pre-defined strings.
    pub fn dapp_provider(self, dapp_provider: &'a str) -> NbglWarning<'a> {
        NbglWarning {
            dapp_provider: Some(dapp_provider),
            ..self
        }
    }

    /// URL of the security report, used in some pre-defined strings.
    pub fn report_url(self, report_url: &'a str) -> NbglWarning<'a> {
        NbglWarning {
            report_url: Some(report_url),
            ..self
        }
    }

    /// Name of the security report provider, used in some pre-defined strings.
    pub fn report_provider(self, report_provider: &'a str) -> NbglWarning<'a> {
        NbglWarning {
            report_provider: Some(report_provider),
            ..self
        }
    }

    /// Overrides the default provider message.
    pub fn provider_message(self, provider_message: &'a str) -> NbglWarning<'a> {
        NbglWarning {
            provider_message: Some(provider_message),
            ..self
        }
    }

    /// Page opened by the top-right button of the intro page, when not using
    /// pre-defined warnings.
    ///
    /// **Requires [`Self::intro_top_right_icon`].** With no pre-defined
    /// warnings set, NBGL only draws the intro page's top-right button when
    /// that icon is present, and the button is the only way to reach these
    /// details — otherwise they are built but never shown.
    pub fn intro_details(self, details: &'a WarningDetails<'a>) -> NbglWarning<'a> {
        NbglWarning {
            intro_details: Some(details),
            ..self
        }
    }

    /// Page opened by the top-right button on the review's first and last
    /// pages, when not using pre-defined warnings.
    pub fn review_details(self, details: &'a WarningDetails<'a>) -> NbglWarning<'a> {
        NbglWarning {
            review_details: Some(details),
            ..self
        }
    }

    /// Body of the intro warning page, when not using pre-defined warnings.
    pub fn info(self, info: &'a CenterInfo<'a>) -> NbglWarning<'a> {
        NbglWarning {
            info: Some(info),
            ..self
        }
    }

    /// Icon of the intro page's top-right button, when not using pre-defined
    /// warnings.
    ///
    /// Setting this is what makes the button appear at all, so it is required
    /// for [`Self::intro_details`] to be reachable.
    pub fn intro_top_right_icon(self, icon: &'a NbglGlyph<'a>) -> NbglWarning<'a> {
        NbglWarning {
            intro_top_right_icon: Some(icon),
            ..self
        }
    }

    /// Icon of the review's top-right button, when not using pre-defined
    /// warnings.
    ///
    /// Note that the C SDK declares `reviewTopRightIcon` but never reads it —
    /// the review page picks its own icon. This setter is here for parity with
    /// `nbgl_warning_t`; it currently has no visible effect.
    pub fn review_top_right_icon(self, icon: &'a NbglGlyph<'a>) -> NbglWarning<'a> {
        NbglWarning {
            review_top_right_icon: Some(icon),
            ..self
        }
    }

    /// Starts the flow with a prelude page before the review.
    pub fn prelude(self, prelude: &'a Prelude<'a>) -> NbglWarning<'a> {
        NbglWarning {
            prelude: Some(prelude),
            ..self
        }
    }
}

/// Builds the warning that the older four-argument `warning_details(...)`
/// setters have always produced: a single Web3 Checks "risk detected", with
/// whichever provider strings were supplied.
///
/// Kept so those setters behave exactly as before now that they delegate to
/// [`NbglWarning`].
pub(crate) fn build_legacy_warning<'a>(
    dapp_provider: Option<&'a str>,
    report_url: Option<&'a str>,
    report_provider: Option<&'a str>,
    provider_message: Option<&'a str>,
) -> NbglWarning<'a> {
    let mut warning = NbglWarning::new().predefined(&[WarningType::W3cRiskDetected]);
    if let Some(s) = dapp_provider {
        warning = warning.dapp_provider(s);
    }
    if let Some(s) = report_url {
        warning = warning.report_url(s);
    }
    if let Some(s) = report_provider {
        warning = warning.report_provider(s);
    }
    if let Some(s) = provider_message {
        warning = warning.provider_message(s);
    }
    warning
}

// ---------------------------------------------------------------------------
// C staging
//
// Every struct below owns the C strings, icons and child nodes that the
// corresponding `nbgl_*` struct only borrows. Nodes that are pointed at are
// boxed so their address survives the owner being moved, and every vector is
// filled to completion before anything stores a pointer into it.
// ---------------------------------------------------------------------------

fn opt_cstring(s: Option<&str>) -> Option<CString> {
    s.map(|s| CString::new(s).unwrap())
}

fn opt_ptr(s: &Option<CString>) -> *const c_char {
    match s {
        Some(s) => s.as_ptr(),
        None => core::ptr::null(),
    }
}

fn opt_icon(glyph: Option<&NbglGlyph>) -> Option<Box<nbgl_icon_details_t>> {
    glyph.map(|g| Box::new(g.into()))
}

fn opt_icon_ptr(icon: &Option<Box<nbgl_icon_details_t>>) -> *const nbgl_icon_details_t {
    match icon {
        Some(icon) => &**icon as *const nbgl_icon_details_t,
        None => core::ptr::null(),
    }
}

/// Owns the strings and icon behind one `nbgl_contentCenter_t`.
///
/// Shared with the generic review's `EXTENDED_CENTER` content, which embeds the
/// same struct.
pub(crate) struct CCenterInfo {
    icon: Option<Box<nbgl_icon_details_t>>,
    title: Option<CString>,
    small_title: Option<CString>,
    description: Option<CString>,
    sub_text: Option<CString>,
    icon_hug: u16,
    padding: bool,
}

impl CCenterInfo {
    pub(crate) fn new(info: &CenterInfo) -> CCenterInfo {
        CCenterInfo {
            icon: opt_icon(info.icon),
            title: opt_cstring(info.title),
            small_title: opt_cstring(info.small_title),
            description: opt_cstring(info.description),
            sub_text: opt_cstring(info.sub_text),
            icon_hug: info.icon_hug,
            padding: info.padding,
        }
    }

    pub(crate) fn as_c_type(&self) -> nbgl_contentCenter_t {
        nbgl_contentCenter_t {
            illustrType: ICON_ILLUSTRATION,
            icon: opt_icon_ptr(&self.icon),
            animation: core::ptr::null(),
            animOffsetX: 0,
            animOffsetY: 0,
            title: opt_ptr(&self.title),
            smallTitle: opt_ptr(&self.small_title),
            description: opt_ptr(&self.description),
            subText: opt_ptr(&self.sub_text),
            iconHug: self.icon_hug,
            padding: self.padding,
        }
    }
}

/// Owns the strings behind one `nbgl_layoutQRCode_t`.
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
struct CQrCode {
    url: CString,
    text1: Option<CString>,
    text2: Option<CString>,
    offset_y: i16,
    centered: bool,
    large_text1: bool,
}

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
impl CQrCode {
    fn new(qr: &QrCode) -> CQrCode {
        CQrCode {
            url: CString::new(qr.url).unwrap(),
            text1: opt_cstring(qr.text1),
            text2: opt_cstring(qr.text2),
            offset_y: qr.offset_y,
            centered: qr.centered,
            large_text1: qr.large_text1,
        }
    }

    fn as_c_type(&self) -> nbgl_layoutQRCode_t {
        nbgl_layoutQRCode_t {
            url: self.url.as_ptr(),
            text1: opt_ptr(&self.text1),
            text2: opt_ptr(&self.text2),
            offsetY: self.offset_y,
            centered: self.centered,
            largeText1: self.large_text1,
        }
    }
}

/// Owns one bar list, including the sub-page of every bar.
struct CBarList {
    _texts: Vec<CString>,
    _sub_texts: Vec<Option<CString>>,
    _icons: Vec<Option<Box<nbgl_icon_details_t>>>,
    texts_ptr: Vec<*const c_char>,
    sub_texts_ptr: Vec<*const c_char>,
    icons_ptr: Vec<*const nbgl_icon_details_t>,
    /// One child per bar; a bar with no sub-page gets a `NO_TYPE_WARNING` entry.
    _children: Vec<Option<Box<CDetails>>>,
    details: Vec<nbgl_genericDetails_t>,
}

impl CBarList {
    fn new(bars: &[WarningBar]) -> Box<CBarList> {
        let texts: Vec<CString> = bars.iter().map(|b| CString::new(b.text).unwrap()).collect();
        let sub_texts: Vec<Option<CString>> =
            bars.iter().map(|b| opt_cstring(b.sub_text)).collect();
        let icons: Vec<Option<Box<nbgl_icon_details_t>>> =
            bars.iter().map(|b| opt_icon(b.icon)).collect();

        let texts_ptr: Vec<*const c_char> = texts.iter().map(|s| s.as_ptr()).collect();
        let sub_texts_ptr: Vec<*const c_char> = sub_texts.iter().map(opt_ptr).collect();
        let icons_ptr: Vec<*const nbgl_icon_details_t> = icons.iter().map(opt_icon_ptr).collect();

        // Children are built first and kept boxed, so the `details` array below
        // can point into them without being invalidated later.
        let children: Vec<Option<Box<CDetails>>> = bars
            .iter()
            .map(|b| b.details.map(|d| CDetails::new(d)))
            .collect();
        let details: Vec<nbgl_genericDetails_t> = children
            .iter()
            .map(|child| match child {
                Some(child) => child.as_c_type(),
                // An inert bar: C reads the entry but the type says "nothing".
                None => nbgl_genericDetails_t {
                    title: core::ptr::null(),
                    type_: NO_TYPE_WARNING,
                    __bindgen_anon_1: nbgl_genericDetails_s__bindgen_ty_1 {
                        centeredInfo: nbgl_contentCenter_t::default(),
                    },
                },
            })
            .collect();

        Box::new(CBarList {
            _texts: texts,
            _sub_texts: sub_texts,
            _icons: icons,
            texts_ptr,
            sub_texts_ptr,
            icons_ptr,
            _children: children,
            details,
        })
    }

    fn as_c_type(&self) -> nbgl_genericBarList_t {
        nbgl_genericBarList_t {
            nbBars: self.texts_ptr.len() as u8,
            texts: self.texts_ptr.as_ptr(),
            subTexts: self.sub_texts_ptr.as_ptr(),
            // C declares this `*mut` although it only reads it.
            icons: self.icons_ptr.as_ptr() as *mut *const nbgl_icon_details_t,
            details: self.details.as_ptr(),
        }
    }
}

/// Owns one details page and, for a bar list, the whole sub-tree under it.
///
/// Shared with the choice-with-details use cases, which take the same
/// `nbgl_warningDetails_t`.
pub(crate) struct CDetails {
    title: CString,
    center: Option<Box<CCenterInfo>>,
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    qr: Option<Box<CQrCode>>,
    bars: Option<Box<CBarList>>,
    type_: nbgl_genericDetailsType_t,
}

impl CDetails {
    pub(crate) fn new(details: &WarningDetails) -> Box<CDetails> {
        match details {
            WarningDetails::CenteredInfo { title, info } => Box::new(CDetails {
                title: CString::new(*title).unwrap(),
                center: Some(Box::new(CCenterInfo::new(info))),
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                qr: None,
                bars: None,
                type_: CENTERED_INFO_WARNING,
            }),
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            WarningDetails::QrCode { title, qr } => Box::new(CDetails {
                title: CString::new(*title).unwrap(),
                center: None,
                qr: Some(Box::new(CQrCode::new(qr))),
                bars: None,
                type_: QRCODE_WARNING,
            }),
            WarningDetails::BarList { title, bars } => Box::new(CDetails {
                title: CString::new(*title).unwrap(),
                center: None,
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                qr: None,
                bars: Some(CBarList::new(bars)),
                type_: BAR_LIST_WARNING,
            }),
        }
    }

    pub(crate) fn as_c_type(&self) -> nbgl_genericDetails_t {
        // Written as a sequence of checks rather than one match on a tuple:
        // the `qr` arm only exists on touchscreen devices, and a cfg inside a
        // tuple pattern would not compile away cleanly.
        let mut anon = nbgl_genericDetails_s__bindgen_ty_1 {
            centeredInfo: nbgl_contentCenter_t::default(),
        };
        if let Some(center) = &self.center {
            anon = nbgl_genericDetails_s__bindgen_ty_1 {
                centeredInfo: center.as_c_type(),
            };
        } else if let Some(bars) = &self.bars {
            anon = nbgl_genericDetails_s__bindgen_ty_1 {
                barList: bars.as_c_type(),
            };
        }
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
        if let Some(qr) = &self.qr {
            anon = nbgl_genericDetails_s__bindgen_ty_1 {
                qrCode: qr.as_c_type(),
            };
        }

        nbgl_genericDetails_t {
            title: self.title.as_ptr(),
            type_: self.type_,
            __bindgen_anon_1: anon,
        }
    }
}

/// Owns the strings, icon and sub-page behind one `nbgl_preludeDetails_t`.
struct CPrelude {
    icon: Option<Box<nbgl_icon_details_t>>,
    title: Option<CString>,
    description: Option<CString>,
    button_text: Option<CString>,
    footer_text: Option<CString>,
    /// Kept alive because `c_details` holds pointers into it.
    _details: Option<Box<CDetails>>,
    /// Materialised so `nbgl_preludeDetails_t.details` has something to point at.
    c_details: Option<Box<nbgl_genericDetails_t>>,
}

impl CPrelude {
    fn new(prelude: &Prelude) -> Box<CPrelude> {
        let details = prelude.details.map(CDetails::new);
        let c_details = details.as_ref().map(|d| Box::new(d.as_c_type()));

        Box::new(CPrelude {
            icon: opt_icon(prelude.icon),
            title: opt_cstring(prelude.title),
            description: opt_cstring(prelude.description),
            button_text: opt_cstring(prelude.button_text),
            footer_text: opt_cstring(prelude.footer_text),
            _details: details,
            c_details,
        })
    }

    fn as_c_type(&self) -> nbgl_preludeDetails_t {
        nbgl_preludeDetails_t {
            icon: opt_icon_ptr(&self.icon),
            title: opt_ptr(&self.title),
            description: opt_ptr(&self.description),
            buttonText: opt_ptr(&self.button_text),
            footerText: opt_ptr(&self.footer_text),
            details: match &self.c_details {
                Some(d) => &**d as *const nbgl_genericDetails_t,
                None => core::ptr::null(),
            },
        }
    }
}

/// Owns every C-side buffer backing an `nbgl_warning_t`.
///
/// NBGL only borrows the struct it is handed, so a value of this type must
/// outlive the use-case call that received it.
pub(crate) struct CWarning {
    predefined: u32,
    dapp_provider: Option<CString>,
    report_url: Option<CString>,
    report_provider: Option<CString>,
    provider_message: Option<CString>,
    // Kept alive because the materialised `c_*` structs below hold pointers
    // into them.
    _intro_details: Option<Box<CDetails>>,
    _review_details: Option<Box<CDetails>>,
    _info: Option<Box<CCenterInfo>>,
    intro_top_right_icon: Option<Box<nbgl_icon_details_t>>,
    review_top_right_icon: Option<Box<nbgl_icon_details_t>>,
    _prelude: Option<Box<CPrelude>>,
    // The structs below are materialised once so the pointer fields of
    // `nbgl_warning_t` have stable addresses to reference.
    c_intro_details: Option<Box<nbgl_genericDetails_t>>,
    c_review_details: Option<Box<nbgl_genericDetails_t>>,
    c_info: Option<Box<nbgl_contentCenter_t>>,
    c_prelude: Option<Box<nbgl_preludeDetails_t>>,
}

impl CWarning {
    pub(crate) fn new(warning: &NbglWarning) -> CWarning {
        let intro_details = warning.intro_details.map(CDetails::new);
        let review_details = warning.review_details.map(CDetails::new);
        let info = warning.info.map(|i| Box::new(CCenterInfo::new(i)));
        let prelude = warning.prelude.map(CPrelude::new);

        let c_intro_details = intro_details.as_ref().map(|d| Box::new(d.as_c_type()));
        let c_review_details = review_details.as_ref().map(|d| Box::new(d.as_c_type()));
        let c_info = info.as_ref().map(|i| Box::new(i.as_c_type()));
        let c_prelude = prelude.as_ref().map(|p| Box::new(p.as_c_type()));

        CWarning {
            predefined: warning.predefined,
            dapp_provider: opt_cstring(warning.dapp_provider),
            report_url: opt_cstring(warning.report_url),
            report_provider: opt_cstring(warning.report_provider),
            provider_message: opt_cstring(warning.provider_message),
            _intro_details: intro_details,
            _review_details: review_details,
            _info: info,
            intro_top_right_icon: opt_icon(warning.intro_top_right_icon),
            review_top_right_icon: opt_icon(warning.review_top_right_icon),
            _prelude: prelude,
            c_intro_details,
            c_review_details,
            c_info,
            c_prelude,
        }
    }

    pub(crate) fn as_c_type(&self) -> nbgl_warning_t {
        nbgl_warning_t {
            predefinedSet: self.predefined,
            dAppProvider: opt_ptr(&self.dapp_provider),
            reportUrl: opt_ptr(&self.report_url),
            reportProvider: opt_ptr(&self.report_provider),
            providerMessage: opt_ptr(&self.provider_message),
            introDetails: match &self.c_intro_details {
                Some(d) => &**d as *const nbgl_genericDetails_t,
                None => core::ptr::null(),
            },
            reviewDetails: match &self.c_review_details {
                Some(d) => &**d as *const nbgl_genericDetails_t,
                None => core::ptr::null(),
            },
            info: match &self.c_info {
                Some(i) => &**i as *const nbgl_contentCenter_t,
                None => core::ptr::null(),
            },
            introTopRightIcon: opt_icon_ptr(&self.intro_top_right_icon),
            reviewTopRightIcon: opt_icon_ptr(&self.review_top_right_icon),
            prelude: match &self.c_prelude {
                Some(p) => &**p as *const nbgl_preludeDetails_t,
                None => core::ptr::null(),
            },
        }
    }
}
