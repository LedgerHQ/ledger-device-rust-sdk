use crate::ui::screen_util::draw;
use ledger_secure_sdk_sys;

pub struct Glyph<'a> {
    pub bitmap: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub inverted: bool,
}

impl<'a> Glyph<'a> {
    pub const fn new(bitmap: &'a [u8], width: u32, height: u32) -> Glyph<'a> {
        Glyph {
            bitmap,
            width,
            height,
            inverted: false,
        }
    }
    pub const fn from_include(packed: (&'a [u8], u32, u32)) -> Glyph<'a> {
        Glyph {
            bitmap: packed.0,
            width: packed.1,
            height: packed.2,
            inverted: false,
        }
    }
    pub const fn invert(self) -> Glyph<'a> {
        Glyph {
            inverted: true,
            ..self
        }
    }
    pub fn draw(&self, x: i32, y: i32) {
        draw(x, y, self.width, self.height, self.inverted, self.bitmap);
    }
}

pub fn manual_screen_clear() {
    let inverted = [0u32, 1u32];
    unsafe {
        ledger_secure_sdk_sys::bagl_hal_draw_bitmap_within_rect(
            0,
            0,
            128,
            64,
            2,
            inverted.as_ptr(),
            1,
            BLANK.as_ptr(),
            128 * 64,
        );
    }
}

use include_gif::include_gif;

pub const BLANK: [u8; 1024] = [0u8; 1024];

pub const BACK: Glyph = Glyph::from_include(include_gif!("icons/badge_back.gif"));
pub const CHECKMARK: Glyph = Glyph::from_include(include_gif!("icons/badge_check.gif"));
pub const COGGLE: Glyph = Glyph::from_include(include_gif!("icons/icon_coggle.gif"));
pub const CROSS: Glyph = Glyph::from_include(include_gif!("icons/icon_cross_badge.gif"));
pub const DOWN_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_down.gif"));
pub const LEFT_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_left.gif"));
pub const RIGHT_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_right.gif"));
pub const UP_ARROW: Glyph = Glyph::from_include(include_gif!("icons/icon_up.gif"));
pub const CERTIFICATE: Glyph = Glyph::from_include(include_gif!("icons/icon_certificate.gif"));
pub const CROSSMARK: Glyph = Glyph::from_include(include_gif!("icons/icon_crossmark.gif"));
pub const DASHBOARD: Glyph = Glyph::from_include(include_gif!("icons/icon_dashboard.gif"));
pub const DASHBOARD_X: Glyph = Glyph::from_include(include_gif!("icons/icon_dashboard_x.gif"));
pub const EYE: Glyph = Glyph::from_include(include_gif!("icons/icon_eye.gif"));
pub const PROCESSING: Glyph = Glyph::from_include(include_gif!("icons/icon_processing.gif"));
pub const VALIDATE_14: Glyph = Glyph::from_include(include_gif!("icons/icon_validate_14.gif"));
pub const WARNING: Glyph = Glyph::from_include(include_gif!("icons/icon_warning.gif"));
