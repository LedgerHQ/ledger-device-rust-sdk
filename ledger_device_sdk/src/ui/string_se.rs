use crate::ui::fonts::OPEN_SANS;
use crate::ui::layout::*;
use crate::ui::screen_util::{draw, screen_update};
use core::ffi::c_void;
use ledger_secure_sdk_sys;

extern "C" {
    fn pic(link_address: *mut c_void) -> *mut c_void;
}

impl StringPlace for &str {
    fn compute_width(&self, bold: bool) -> usize {
        let font_choice = bold as usize;
        self.as_bytes()
            .iter()
            .map(ledger_secure_sdk_sys::pic_rs)
            .fold(0, |acc, c| {
                acc + OPEN_SANS[font_choice].dims[*c as usize - 0x20] as usize
            })
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        let total_width = self.compute_width(bold);
        let mut cur_x = layout.get_x(total_width) as i32;

        let font_choice = bold as usize;
        for c in self.as_bytes().iter().map(ledger_secure_sdk_sys::pic_rs) {
            let offset_c = *c as usize - 0x20;
            let character = unsafe {
                let tmp = pic(OPEN_SANS[font_choice].chars.0[offset_c].as_ptr() as *mut c_void)
                    as *const u8;
                core::slice::from_raw_parts(tmp, OPEN_SANS[font_choice].chars.0[offset_c].len())
            };
            let c_width = OPEN_SANS[font_choice].dims[offset_c];
            let c_height = OPEN_SANS[font_choice].height as usize;
            let y = loc.get_y(c_height);
            draw(
                cur_x,
                y as i32,
                c_width as u32,
                c_height as u32,
                false,
                character,
            );
            cur_x += c_width as i32;
        }
        screen_update();
    }
}

impl StringPlace for [&str] {
    fn compute_width(&self, bold: bool) -> usize {
        self.iter().fold(0, |acc, s| acc.max(s.compute_width(bold)))
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        let c_height = OPEN_SANS[bold as usize].height as usize;
        let padding = if self.len() > 4 { 0 } else { 1 };
        let total_height = self.len() * (c_height + padding);
        let mut cur_y = loc.get_y(total_height);
        for string in self.iter() {
            string.place(Location::Custom(cur_y), layout, bold);
            cur_y += c_height + 2 * padding;
        }
    }
}

use crate::ui::bagls::se::Label;

impl<'a> StringPlace for Label<'a> {
    fn compute_width(&self, _bold: bool) -> usize {
        self.text.compute_width(self.bold)
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        self.text.place(loc, layout, bold);
    }
}

impl<'a> StringPlace for [Label<'a>] {
    fn compute_width(&self, bold: bool) -> usize {
        self.iter()
            .fold(0, |acc, lbl| acc.max(lbl.compute_width(bold)))
    }

    fn place(&self, _loc: Location, layout: Layout, bold: bool) {
        let c_height = OPEN_SANS[bold as usize].height as usize;
        let padding = ((crate::ui::SCREEN_HEIGHT / self.len()) - c_height) / 2;
        let mut cur_y = padding;
        for label in self.iter() {
            label.place(Location::Custom(cur_y), layout, label.bold);
            cur_y += c_height + 2 * padding;
        }
    }
}
