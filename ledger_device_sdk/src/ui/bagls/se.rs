use super::Icon;
use crate::ui::fonts::OPEN_SANS;
use crate::ui::layout::*;
use crate::ui::screen_util::draw;
use ledger_secure_sdk_sys;

pub struct Label<'a> {
    pub text: &'a str,
    pub bold: bool,
    pub loc: Location,
    layout: Layout,
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
            draw(
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

impl<'a> Draw for Icon<'a> {
    fn display(&self) {
        draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            self.icon.width,
            self.icon.height,
            self.icon.inverted,
            self.icon.bitmap,
        );
    }

    fn erase(&self) {
        draw(
            self.pos.0 as i32,
            self.pos.1 as i32,
            self.icon.width,
            self.icon.height,
            self.icon.inverted,
            &crate::ui::bitmaps::BLANK,
        );
    }
}
