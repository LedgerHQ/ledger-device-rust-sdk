use super::Icon;
use crate::layout::{Draw, Layout, Location};
use ledger_secure_sdk_sys;
use ledger_secure_sdk_sys::seph::SephTags;

#[repr(u8)]
pub enum BaglTypes {
    NoneType = 0,
    Button = 1,
    Label = 2,
    Rectangle = 3,
    Line = 4,
    Icon = 5,
    Circle = 6,
    LabelLine = 7,
}

pub const BAGL_FONT_ALIGNMENT_CENTER: u32 = 32768;

#[repr(C)]
pub struct BaglComponent {
    pub type_: u8,
    pub userid: u8,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub stroke: u8,
    pub radius: u8,
    pub fill: u8,
    pub fgcolor: u32,
    pub bgcolor: u32,
    pub font_id: u16,
    pub icon_id: u8,
}

impl BaglComponent {
    pub fn paint(&self) {
        let bagl_comp = unsafe {
            core::slice::from_raw_parts(
                self as *const BaglComponent as *const u8,
                core::mem::size_of::<BaglComponent>(),
            )
        };

        ledger_secure_sdk_sys::seph::seph_send(&[
            SephTags::ScreenDisplayStatus as u8,
            0,
            bagl_comp.len() as u8,
        ]);
        ledger_secure_sdk_sys::seph::seph_send(bagl_comp);
    }
}

pub trait SendToDisplay {
    fn wait_for_status(&self) {
        if ledger_secure_sdk_sys::seph::is_status_sent() {
            // TODO: this does not seem like the right way to fix the problem...
            let mut spi_buffer = [0u8; 16];
            ledger_secure_sdk_sys::seph::seph_recv(&mut spi_buffer, 0);
        }
    }
    fn paint(&self);
    fn send_to_display(&self) {
        BLANK.paint();
        self.paint();
    }
}

pub enum Bagl<'a> {
    LABELLINE(Label<'a>),
    RECT(Rect),
    ICON(Icon<'a>),
}

impl Bagl<'_> {
    /// Erase screen and display the bagl
    pub fn display(&self) {
        match self {
            Bagl::LABELLINE(x) => x.send_to_display(),
            Bagl::RECT(x) => x.send_to_display(),
            Bagl::ICON(x) => x.send_to_display(),
        }
    }

    /// Only paint to current screen (draw over)
    pub fn paint(&self) {
        match self {
            Bagl::LABELLINE(x) => x.paint(),
            Bagl::RECT(x) => x.paint(),
            Bagl::ICON(x) => x.paint(),
        }
    }
}

#[repr(C)]
pub struct bagl_element_rs<'a> {
    pub component: BaglComponent,
    pub text: Option<&'a str>,
}

impl Draw for Icon<'_> {
    fn display(&self) {
        self.paint();
    }
    fn erase(&self) {
        let icon = ledger_secure_sdk_sys::pic_rs(self.icon);
        Rect::new()
            .pos(self.pos.0, self.pos.1)
            .dims(icon.width as u16, icon.height as u16)
            .colors(0, 0xffffff)
            .fill(true)
            .paint();
    }
}

#[repr(u8)]
pub enum Font {
    LucidaConsole8px = 0,
    OpenSansLight16_22px,
    OpenSansRegular8_11px,
    OpenSansRegular10_13px,
    OpenSansRegular11_14px,
    OpenSansRegular13_18px,
    OpenSansRegular22_30px,
    OpenSansSemibold8_11px,
    OpenSansExtrabold11px,
    OpenSansLight16px,
    OpenSansRegular11px,
    OpenSansSemibold10_13px,
    OpenSansSemibold11_16px,
    OpenSansSemibold13_18px,
    Symbols0,
    Symbols1,
}

pub struct Label<'a> {
    pub loc: Location,
    pub layout: Layout,
    pub dims: (u16, u16),
    pub bold: bool,
    pub text: &'a str,
}

impl<'a> Label<'a> {
    pub const fn new() -> Self {
        Label {
            loc: Location::Middle,
            layout: Layout::Centered,
            dims: (128, 11),
            bold: false,
            text: "",
        }
    }

    pub const fn from_const(text: &'static str) -> Self {
        Label {
            loc: Location::Middle,
            layout: Layout::Centered,
            dims: (128, 11),
            bold: false,
            text,
        }
    }

    pub const fn location(self, loc: Location) -> Self {
        Label { loc, ..self }
    }
    pub const fn layout(self, layout: Layout) -> Self {
        Label { layout, ..self }
    }
    pub const fn dims(self, w: u16, h: u16) -> Self {
        Label {
            dims: (w, h),
            ..self
        }
    }
    pub const fn bold(self) -> Self {
        Label { bold: true, ..self }
    }
    pub fn text(self, text: &'a str) -> Self {
        Label { text, ..self }
    }
}

impl<'a> From<&'a str> for Label<'a> {
    fn from(text: &'a str) -> Label<'a> {
        Label::new().text(text)
    }
}

impl<'a> Draw for Label<'a> {
    fn display(&self) {
        self.paint();
    }
    fn erase(&self) {
        let x = self.layout.get_x(self.dims.0 as usize) as i16;
        let y = self.loc.get_y(self.dims.1 as usize) as i16;
        Rect::new()
            .pos(x, y)
            .dims(self.dims.0, self.dims.1)
            .colors(0, 0xffffff)
            .fill(true)
            .paint();
    }
}

pub struct Rect {
    pub pos: (i16, i16),
    pub dims: (u16, u16),
    pub colors: (u32, u32),
    pub fill: bool,
    pub userid: u8,
}

impl Rect {
    pub const fn new() -> Rect {
        Rect {
            pos: (32 - 5, 64 - 5),
            dims: (10, 10),
            colors: (0xffffffu32, 0),
            fill: false,
            userid: 0,
        }
    }
    pub const fn pos(self, x: i16, y: i16) -> Rect {
        Rect {
            pos: (x, y),
            ..self
        }
    }
    pub const fn colors(self, fg: u32, bg: u32) -> Rect {
        Rect {
            colors: (fg, bg),
            ..self
        }
    }
    pub const fn dims(self, w: u16, h: u16) -> Rect {
        Rect {
            dims: (w, h),
            ..self
        }
    }
    pub const fn fill(self, x: bool) -> Rect {
        Rect { fill: x, ..self }
    }
    pub const fn userid(self, id: u8) -> Rect {
        Rect { userid: id, ..self }
    }
}

impl Draw for Rect {
    fn display(&self) {
        self.paint();
    }
    fn erase(&self) {
        Rect::new()
            .pos(self.pos.0, self.pos.1)
            .dims(self.dims.0, self.dims.1)
            .colors(0, 0xffffff)
            .fill(true)
            .paint();
    }
}

impl Draw for RectFull {
    fn display(&self) {
        self.paint();
    }
    fn erase(&self) {
        Rect::new()
            .pos(self.pos.0 as i16, self.pos.1 as i16)
            .dims(self.width as u16, self.height as u16)
            .colors(0, 0xffffff)
            .fill(true)
            .paint();
    }
}

impl SendToDisplay for Icon<'_> {
    fn paint(&self) {
        self.wait_for_status();
        let icon = ledger_secure_sdk_sys::pic_rs(self.icon);
        let baglcomp = BaglComponent {
            type_: BaglTypes::Icon as u8,
            userid: 0,
            x: self.pos.0,
            y: self.pos.1,
            width: icon.width as u16,
            height: icon.height as u16,
            stroke: 0,
            radius: 0,
            fill: 0,
            fgcolor: 0,
            bgcolor: 0,
            font_id: 0,
            icon_id: 0,
        };
        let bagl_comp = unsafe {
            core::slice::from_raw_parts(
                &baglcomp as *const BaglComponent as *const u8,
                core::mem::size_of::<BaglComponent>(),
            )
        };
        let lenbytes = ((bagl_comp.len() + 1 + (2 * 4) + icon.bitmap.len()) as u16).to_be_bytes();
        ledger_secure_sdk_sys::seph::seph_send(&[
            SephTags::ScreenDisplayStatus as u8,
            lenbytes[0],
            lenbytes[1],
        ]);
        ledger_secure_sdk_sys::seph::seph_send(bagl_comp);
        // bpp (1), 'color_index' (2*4)
        ledger_secure_sdk_sys::seph::seph_send(&[1, 0, 0, 0, 0, 0xff, 0xff, 0xff, 0]);
        let bmp = unsafe {
            core::slice::from_raw_parts(
                ledger_secure_sdk_sys::pic(icon.bitmap.as_ptr() as *mut c_void) as *const u8,
                icon.bitmap.len(),
            )
        };
        ledger_secure_sdk_sys::seph::seph_send(bmp);
    }
}

impl SendToDisplay for Rect {
    fn paint(&self) {
        self.wait_for_status();
        let baglcomp = BaglComponent {
            type_: BaglTypes::Rectangle as u8,
            userid: self.userid,
            x: self.pos.0,
            y: self.pos.1,
            width: self.dims.0,
            height: self.dims.1,
            stroke: 0,
            radius: 0,
            fill: self.fill as u8,
            fgcolor: self.colors.0,
            bgcolor: self.colors.1,
            font_id: 0,
            icon_id: 0,
        };
        baglcomp.paint();
    }
}

use super::RectFull;

impl SendToDisplay for RectFull {
    fn paint(&self) {
        self.wait_for_status();
        let baglcomp = BaglComponent {
            type_: BaglTypes::Rectangle as u8,
            userid: 0,
            x: self.pos.0 as i16,
            y: self.pos.1 as i16,
            width: self.width as u16,
            height: self.height as u16,
            stroke: 0,
            radius: 0,
            fill: 1,
            fgcolor: 0xffffff,
            bgcolor: 0,
            font_id: 0,
            icon_id: 0,
        };
        baglcomp.paint();
    }
}

use core::ffi::c_void;

impl<'a> SendToDisplay for Label<'a> {
    fn paint(&self) {
        self.wait_for_status();
        let font_id = if self.bold {
            Font::OpenSansExtrabold11px
        } else {
            Font::OpenSansRegular11px
        };
        let x = match self.layout {
            Layout::RightAligned => self.layout.get_x(self.text.len() * 7),
            _ => 0,
        };
        let y = self.loc.get_y(self.dims.1 as usize) as i16;
        let width = match self.layout {
            Layout::Centered => crate::SCREEN_WIDTH,
            _ => self.text.len() * 6,
        };
        let baglcomp = BaglComponent {
            type_: BaglTypes::LabelLine as u8,
            userid: 0, // FIXME
            x: x as i16,
            y: y - 1 + self.dims.1 as i16,
            width: width as u16, //self.dims.0,
            height: self.dims.1,
            stroke: 0,
            radius: 0,
            fill: 0,
            fgcolor: 0xffffffu32,
            bgcolor: 0,
            font_id: font_id as u16 | BAGL_FONT_ALIGNMENT_CENTER as u16,
            icon_id: 0,
        };

        let bagl_comp = unsafe {
            core::slice::from_raw_parts(
                &baglcomp as *const BaglComponent as *const u8,
                core::mem::size_of::<BaglComponent>(),
            )
        };
        let lenbytes = ((bagl_comp.len() + self.text.len()) as u16).to_be_bytes();
        ledger_secure_sdk_sys::seph::seph_send(&[
            SephTags::ScreenDisplayStatus as u8,
            lenbytes[0],
            lenbytes[1],
        ]);
        ledger_secure_sdk_sys::seph::seph_send(bagl_comp);

        unsafe {
            let pic_text = ledger_secure_sdk_sys::pic(self.text.as_ptr() as *mut u8 as *mut c_void);
            ledger_secure_sdk_sys::io_seph_send(pic_text as *mut u8, self.text.len() as u16);
        }
    }
}

/// Some common constant Bagls
pub const BLANK: Rect = Rect::new()
    .pos(0, 0)
    .dims(128, 32)
    .colors(0, 0xffffff)
    .fill(true);
