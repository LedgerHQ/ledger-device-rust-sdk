//! A wrapper around the NBGL [nbgl_useCaseNavigableContent](https://github.com/LedgerHQ/ledger-secure-sdk/blob/master/lib_nbgl/src/nbgl_use_case.c#L3503) C API binding.
//!
//! Draws a set of pages with a touchable header, used to walk through content
//! that is not a review — settings-like screens, or any flow where the header
//! goes back to an upper level. The C SDK deprecates `nbgl_useCaseSettings` in
//! favour of it.
//!
//! Pages are declared up front, one [`NbglPageContent`] each:
//!
//! ```no_run
//! # use ledger_device_sdk::nbgl::{ChoicesList, NbglNavigableContent, NbglPageContent};
//! let mut content = NbglNavigableContent::new()
//!     .title("Network")
//!     .add_page(NbglPageContent::ChoicesList(ChoicesList::new(
//!         &["Mainnet", "Testnet"],
//!         0,
//!     )));
//! content.show();
//! ```
//!
//! Unlike a review, the flow has no confirm step: it ends when the user leaves
//! through the header.

use super::*;
use core::sync::atomic::{AtomicPtr, Ordering};

/// The instance currently on screen, so the C callbacks can reach its pages.
///
/// `nbgl_navCallback_t` carries no user data, and the pages have to outlive the
/// callback that hands them to NBGL, so they are owned by the instance rather
/// than built on demand.
static NAV_REF: AtomicPtr<NbglNavigableContent> = AtomicPtr::new(core::ptr::null_mut());

/// One page of the flow: its content, plus the header fields
/// `nbgl_pageContent_t` carries alongside the content union.
struct PageEntry {
    content: NbglPageContent,
    // The three below are read only on touchscreen devices: Nano's
    // nbgl_pageContent_t has no title or top-right button. The API stays the
    // same on every target, the fields simply go unused there.
    #[cfg_attr(any(target_os = "nanosplus", target_os = "nanox"), allow(dead_code))]
    title: Option<CString>,
    #[cfg_attr(any(target_os = "nanosplus", target_os = "nanox"), allow(dead_code))]
    touchable_title: bool,
    #[cfg_attr(any(target_os = "nanosplus", target_os = "nanox"), allow(dead_code))]
    top_right_icon: Option<nbgl_icon_details_t>,
}

/// Writes one page's content into the `nbgl_pageContent_t` NBGL provides.
///
/// `nbgl_pageContent_t` has its own union, distinct from the `nbgl_content_u`
/// used by [`NbglGenericReview`] even though the members match, so the mapping
/// is spelled out rather than shared.
fn fill_page_content(entry: &PageEntry, page: &mut nbgl_pageContent_t) {
    // Nano's nbgl_pageContent_t carries only the type and the union: the
    // per-page title and top-right button are touchscreen-only.
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    {
        page.title = match &entry.title {
            Some(title) => title.as_ptr(),
            None => core::ptr::null(),
        };
        page.isTouchableTitle = entry.touchable_title;
        page.titleToken = BACK_TOKEN;
        page.topRightIcon = match &entry.top_right_icon {
            Some(icon) => icon as *const nbgl_icon_details_t,
            None => core::ptr::null(),
        };
    }

    match &entry.content {
        NbglPageContent::CenteredInfo(data) => {
            page.type_ = CENTERED_INFO;
            page.__bindgen_anon_1.centeredInfo = data.into();
        }
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
        NbglPageContent::ExtendedCenter(data) => {
            page.type_ = EXTENDED_CENTER;
            page.__bindgen_anon_1.extendedCenter = data.into();
        }
        #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
        NbglPageContent::InfoLongPress(data) => {
            page.type_ = INFO_LONG_PRESS;
            page.__bindgen_anon_1.infoLongPress = data.into();
        }
        NbglPageContent::InfoButton(data) => {
            page.type_ = INFO_BUTTON;
            page.__bindgen_anon_1.infoButton = data.into();
        }
        NbglPageContent::TagValueList(data) => {
            page.type_ = TAG_VALUE_LIST;
            page.__bindgen_anon_1.tagValueList = data.into();
        }
        NbglPageContent::TagValueConfirm(data) => {
            page.type_ = TAG_VALUE_CONFIRM;
            page.__bindgen_anon_1.tagValueConfirm = data.into();
        }
        NbglPageContent::SwitchesList(data) => {
            page.type_ = SWITCHES_LIST;
            page.__bindgen_anon_1.switchesList = data.into();
        }
        NbglPageContent::InfosList(data) => {
            page.type_ = INFOS_LIST;
            page.__bindgen_anon_1.infosList = data.into();
        }
        NbglPageContent::ChoicesList(data) => {
            page.type_ = CHOICES_LIST;
            page.__bindgen_anon_1.choicesList = data.into();
        }
        NbglPageContent::BarsList(data) => {
            page.type_ = BARS_LIST;
            page.__bindgen_anon_1.barsList = data.into();
        }
        // Rejected by `show` on this target, so unreachable here.
        #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
        _ => {}
    }
}

/// Whether this device's `nbgl_pageContent_t` can carry the given content.
///
/// Nano's union has eight members where the touchscreen devices have eleven,
/// so two of the content types cannot appear in a navigable flow there.
fn is_supported(content: &NbglPageContent) -> bool {
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    if matches!(
        content,
        NbglPageContent::ExtendedCenter(_) | NbglPageContent::InfoLongPress(_)
    ) {
        return false;
    }
    let _ = content;
    true
}

/// Token reported when a touchable page title is pressed.
#[cfg_attr(any(target_os = "nanosplus", target_os = "nanox"), allow(dead_code))]
const BACK_TOKEN: u8 = 0;

/// Callback through which NBGL asks for the content of each page.
unsafe extern "C" fn navigation_callback(page: u8, content: *mut nbgl_pageContent_t) -> bool {
    let ptr = NAV_REF.load(Ordering::Relaxed);
    if ptr.is_null() || content.is_null() {
        return false;
    }
    let this = unsafe { &*ptr };
    match this.pages.get(page as usize) {
        Some(entry) => {
            fill_page_content(entry, unsafe { &mut *content });
            true
        }
        // Returning false tells NBGL there is nothing to draw for this page.
        None => false,
    }
}

/// Callback through which NBGL reports a control being touched.
unsafe extern "C" fn controls_callback(token: c_int, index: u8) {
    let ptr = NAV_REF.load(Ordering::Relaxed);
    if ptr.is_null() {
        return;
    }
    if let Some(on_control) = unsafe { (*ptr).on_control } {
        on_control(token as u8, index);
    }
}

/// A builder for a flow of navigable pages.
pub struct NbglNavigableContent {
    title: CString,
    init_page: u8,
    pages: Vec<PageEntry>,
    on_control: Option<fn(u8, u8)>,
}

impl SyncNBGL for NbglNavigableContent {}

impl Default for NbglNavigableContent {
    fn default() -> Self {
        Self::new()
    }
}

impl NbglNavigableContent {
    /// Creates a new navigable content builder.
    pub fn new() -> NbglNavigableContent {
        NbglNavigableContent {
            title: CString::default(),
            init_page: 0,
            pages: Vec::new(),
            on_control: None,
        }
    }

    /// Sets the text of the touchable header.
    pub fn title(self, title: &str) -> NbglNavigableContent {
        NbglNavigableContent {
            title: CString::new(title).unwrap(),
            ..self
        }
    }

    /// Sets the page the flow opens on, counting from 0.
    pub fn init_page(self, init_page: u8) -> NbglNavigableContent {
        NbglNavigableContent { init_page, ..self }
    }

    /// Sets the function run when a control on a page is touched.
    ///
    /// It receives the token of the touched control and, for a choices list,
    /// the index chosen. The content types in this module assign
    /// `FIRST_USER_TOKEN + i` to their `i`th element, except
    /// [`ChoicesList`], which uses a single token and reports the selection in
    /// `index`.
    ///
    /// The function runs inside NBGL's event dispatch, so it should return
    /// promptly.
    pub fn on_control(self, on_control: fn(u8, u8)) -> NbglNavigableContent {
        NbglNavigableContent {
            on_control: Some(on_control),
            ..self
        }
    }

    /// Appends a page.
    pub fn add_page(mut self, content: NbglPageContent) -> NbglNavigableContent {
        self.pages.push(PageEntry {
            content,
            title: None,
            touchable_title: false,
            top_right_icon: None,
        });
        self
    }

    /// Appends a page carrying its own title.
    ///
    /// # Arguments
    /// * `content` - The page's content.
    /// * `title` - Title drawn on this page.
    /// * `touchable_title` - If `true`, the title is preceded by a back arrow.
    /// * `top_right_icon` - Icon of the page's top-right button, if any.
    ///
    /// All three are ignored on Nano, whose page content carries only the
    /// content union.
    pub fn add_titled_page(
        mut self,
        content: NbglPageContent,
        title: &str,
        touchable_title: bool,
        top_right_icon: Option<&NbglGlyph>,
    ) -> NbglNavigableContent {
        self.pages.push(PageEntry {
            content,
            title: Some(CString::new(title).unwrap()),
            touchable_title,
            top_right_icon: top_right_icon.map(|g| g.into()),
        });
        self
    }

    /// Shows the flow, returning when the user leaves through the header.
    ///
    /// # Panics
    /// Panics if no page was added, since NBGL would have nothing to draw.
    pub fn show(&mut self) {
        if self.pages.is_empty() {
            panic!("No page added.");
        }
        // Fail loudly here rather than have NBGL draw an empty page for a
        // content type this device's page union cannot represent.
        if self.pages.iter().any(|p| !is_supported(&p.content)) {
            panic!("Content type not supported in a navigable flow on this device.");
        }

        // The callbacks reach the pages through this pointer, so it has to be
        // set before NBGL can ask for the first page.
        NAV_REF.store(self as *mut NbglNavigableContent, Ordering::Relaxed);

        unsafe {
            self.ux_sync_init();
            nbgl_useCaseNavigableContent(
                match self.title.is_empty() {
                    true => core::ptr::null(),
                    false => self.title.as_ptr() as *const c_char,
                },
                self.init_page,
                self.pages.len() as u8,
                Some(quit_callback),
                Some(navigation_callback),
                Some(controls_callback),
            );
            self.ux_sync_wait(false);
        }

        NAV_REF.store(core::ptr::null_mut(), Ordering::Relaxed);
    }
}
