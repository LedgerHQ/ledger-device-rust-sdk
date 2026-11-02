use crate::ui::fonts::OPEN_SANS;
use crate::ui::layout::*;
use crate::ui::screen_util::{draw, screen_update};
use core::ffi::c_void;
use ledger_secure_sdk_sys;

unsafe extern "C" {
    fn pic(link_address: *mut c_void) -> *mut c_void;
}

/// Index in the font tables of the glyph drawn for `c`. The font covers 0x20
/// to 0x7F; any other byte (control characters, UTF-8 sequences) is drawn
/// as the 0x7F placeholder.
fn glyph_index(c: u8) -> usize {
    match c {
        0x20..=0x7F => usize::from(c - 0x20),
        _ => 0x7F - 0x20,
    }
}

impl StringPlace for &str {
    fn compute_width(&self, bold: bool) -> usize {
        let font_choice = bold as usize;
        self.as_bytes()
            .iter()
            .map(ledger_secure_sdk_sys::pic_rs)
            .fold(0, |acc, c| {
                acc + OPEN_SANS[font_choice].dims[glyph_index(*c)] as usize
            })
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        let total_width = self.compute_width(bold);
        let mut cur_x = layout.get_x(total_width) as i32;

        let font_choice = bold as usize;
        for c in self.as_bytes().iter().map(ledger_secure_sdk_sys::pic_rs) {
            let offset_c = glyph_index(*c);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn glyph_index_covers_the_font() {
        assert_eq!(glyph_index(b' '), 0);
        assert_eq!(glyph_index(0x7F), OPEN_SANS[0].dims.len() - 1);
        assert_eq!(glyph_index(b'\n'), glyph_index(0x7F));
        assert_eq!(glyph_index(0x1F), glyph_index(0x7F));
        assert_eq!(glyph_index(0xC3), glyph_index(0x7F));
        assert_eq!(glyph_index(0xFF), glyph_index(0x7F));
    }

    #[test]
    fn unsupported_bytes_are_measured_as_placeholders() {
        for bold in [false, true] {
            let placeholder = "\u{7f}".compute_width(bold);
            assert_eq!("\t".compute_width(bold), placeholder);
            // U+00E9 is two bytes in UTF-8.
            assert_eq!("\u{e9}".compute_width(bold), 2 * placeholder);
            assert_eq!(
                "a\nb".compute_width(bold),
                "a".compute_width(bold) + placeholder + "b".compute_width(bold)
            );
        }
    }
}
