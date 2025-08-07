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

extern "C" {
    /// The bitmap for the back icon.
    pub static C_icon_back_bitmap: [u8; 25];
}
pub static BACK: Glyph = Glyph {
    bitmap: unsafe { &C_icon_back_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_back_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_back_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the back_x icon.
    pub static C_icon_back_x_bitmap: [u8; 25];
}
pub static BACK_X: Glyph = Glyph {
    bitmap: unsafe { &C_icon_back_x_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_back_x_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_back_x_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the coggle icon.
    pub static C_icon_coggle_bitmap: [u8; 25];
}
pub static COGGLE: Glyph = Glyph {
    bitmap: unsafe { &C_icon_coggle_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_coggle_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_coggle_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the processing icon.
    pub static C_icon_down_bitmap: [u8; 25];
}
pub static DOWN_ARROW: Glyph = Glyph {
    bitmap: unsafe { &C_icon_down_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_down_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_down_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the left arrow icon.
    pub static C_icon_left_bitmap: [u8; 25];
}
pub static LEFT_ARROW: Glyph = Glyph {
    bitmap: unsafe { &C_icon_left_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_left_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_left_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the right arrow icon.
    pub static C_icon_right_bitmap: [u8; 25];
}
pub static RIGHT_ARROW: Glyph = Glyph {
    bitmap: unsafe { &C_icon_right_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_right_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_right_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the up arrow icon.
    pub static C_icon_up_bitmap: [u8; 25];
}
pub static UP_ARROW: Glyph = Glyph {
    bitmap: unsafe { &C_icon_up_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_up_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_up_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the certificate icon.
    pub static C_icon_certificate_bitmap: [u8; 25];
}
pub static CERTIFICATE: Glyph = Glyph {
    bitmap: unsafe { &C_icon_certificate_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_certificate_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_certificate_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the crossmark icon.
    pub static C_icon_crossmark_bitmap: [u8; 25];
}
pub static CROSSMARK: Glyph = Glyph {
    bitmap: unsafe { &C_icon_crossmark_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_crossmark_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_crossmark_HEIGHT,
    inverted: false,
};
// This is a special case for the cross icon, which is a GIF (used by client apps)
pub const CROSS: Glyph = Glyph::from_include(include_gif!("icons/icon_cross_badge.gif"));
extern "C" {
    /// The bitmap for the dashboard icon.
    pub static C_icon_dashboard_bitmap: [u8; 25];
}
pub static DASHBOARD: Glyph = Glyph {
    bitmap: unsafe { &C_icon_dashboard_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_dashboard_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_dashboard_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the dashboard x icon.
    pub static C_icon_dashboard_x_bitmap: [u8; 25];
}
pub static DASHBOARD_X: Glyph = Glyph {
    bitmap: unsafe { &C_icon_dashboard_x_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_dashboard_x_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_dashboard_x_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the eye icon.
    pub static C_icon_eye_bitmap: [u8; 25];
}
pub static EYE: Glyph = Glyph {
    bitmap: unsafe { &C_icon_eye_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_eye_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_eye_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the processing icon.
    pub static C_icon_processing_bitmap: [u8; 25];
}
pub static PROCESSING: Glyph = Glyph {
    bitmap: unsafe { &C_icon_processing_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_processing_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_processing_HEIGHT,
    inverted: false,
};
extern "C" {
    /// The bitmap for the validate icon.
    pub static C_icon_validate_14_bitmap: [u8; 25];
}
pub static VALIDATE_14: Glyph = Glyph {
    bitmap: unsafe { &C_icon_validate_14_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_validate_14_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_validate_14_HEIGHT,
    inverted: false,
};
// This is a special case for the checkmark icon, which is a GIF (used by client apps)
pub const CHECKMARK: Glyph = Glyph::from_include(include_gif!("icons/badge_check.gif"));
extern "C" {
    /// The bitmap for the warning icon.
    pub static C_icon_warning_bitmap: [u8; 25];
}
pub static WARNING: Glyph = Glyph {
    bitmap: unsafe { &C_icon_warning_bitmap },
    width: ledger_secure_sdk_sys::GLYPH_icon_warning_WIDTH,
    height: ledger_secure_sdk_sys::GLYPH_icon_warning_HEIGHT,
    inverted: false,
};
