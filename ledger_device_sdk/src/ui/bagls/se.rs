use super::Icon;
use crate::ui::fonts::OPEN_SANS;
use crate::ui::layout::*;
use ledger_secure_sdk_sys;

#[derive(Clone, Copy)]
pub struct Label<'a> {
    pub text: &'a str,
    pub bold: bool,
    pub loc: Location,
    layout: Layout,
}

impl Default for Label<'_> {
    fn default() -> Self {
        Label {
            text: "",
            bold: false,
            loc: Location::Middle,
            layout: Layout::Centered,
        }
    }
}

impl<'a> From<&'a str> for Label<'a> {
    fn from(s: &'a str) -> Label<'a> {
        Label {
            text: s,
            bold: false,
            loc: Location::Middle,
            layout: Layout::Centered,
        }
    }
}

impl<'a> Label<'a> {
    pub const fn from_const(s: &'a str) -> Label<'a> {
        Label {
            text: s,
            bold: false,
            loc: Location::Middle,
            layout: Layout::Centered,
        }
    }

    pub const fn location(self, loc: Location) -> Label<'a> {
        Label { loc, ..self }
    }

    pub const fn layout(self, layout: Layout) -> Label<'a> {
        Label { layout, ..self }
    }

    pub const fn bold(&self) -> Label<'a> {
        Label {
            bold: true,
            ..*self
        }
    }
}

impl Draw for Label<'_> {
    fn display(&self) {
        self.text.place(self.loc, self.layout, self.bold);
    }
    fn erase(&self) {
        let total_width = self.text.compute_width(self.bold);
        let c_height = OPEN_SANS[self.bold as usize].height as usize;
        if total_width != 0 {
            let x = self.layout.get_x(total_width);
            let y = self.loc.get_y(c_height);
            pic_draw(
                x as i32,
                y as i32,
                total_width as u32,
                c_height as u32,
                false,
                &crate::ui::bitmaps::BLANK,
            )
        }
    }
}

use crate::ui::bagls::RectFull;

impl Draw for RectFull {
    fn display(&self) {
        unsafe {
            ledger_secure_sdk_sys::bagl_hal_draw_rect(
                1,
                self.pos.0,
                self.pos.1,
                self.width,
                self.height,
            );
        }
    }

    fn erase(&self) {
        unsafe {
            ledger_secure_sdk_sys::bagl_hal_draw_rect(
                0,
                self.pos.0,
                self.pos.1,
                self.width,
                self.height,
            );
        }
    }
}

use core::ffi::c_void;

#[inline(never)]
fn pic_draw(x: i32, y: i32, width: u32, height: u32, inverted: bool, bitmap: &[u8]) {
    let inverted = [inverted as u32, !inverted as u32];
    unsafe {
        let pic_bmp = ledger_secure_sdk_sys::pic(bitmap.as_ptr() as *mut c_void);
        let _ = ledger_secure_sdk_sys::bagl_hal_draw_bitmap_within_rect(
            x,
            y,
            width,
            height,
            2,
            inverted.as_ptr(),
            1,
            pic_bmp as *const u8,
            width * height,
        );
    }
}

impl<'a> Draw for Icon<'a> {
    fn display(&self) {
        let icon = ledger_secure_sdk_sys::pic_rs(self.icon);
        pic_draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            icon.width,
            icon.height,
            icon.inverted,
            icon.bitmap,
        );
    }

    fn erase(&self) {
        let icon = ledger_secure_sdk_sys::pic_rs(self.icon);
        pic_draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            icon.width,
            icon.height,
            icon.inverted,
            &crate::ui::bitmaps::BLANK,
        );
    }
}
