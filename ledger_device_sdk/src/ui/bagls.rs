pub mod se;
pub use self::se::*;

use bitmaps::Glyph;

pub struct RectFull {
    pos: (i32, i32),
    width: u32,
    height: u32,
}

impl RectFull {
    pub const fn new() -> RectFull {
        RectFull {
            pos: (0, 0),
            width: 1,
            height: 1,
        }
    }
    pub const fn pos(self, x: i32, y: i32) -> RectFull {
        RectFull {
            pos: (x, y),
            ..self
        }
    }
    pub const fn width(self, width: u32) -> RectFull {
        RectFull { width, ..self }
    }
    pub const fn height(self, height: u32) -> RectFull {
        RectFull { height, ..self }
    }
}

const fn middle_y(glyph: &Glyph) -> i16 {
    ((crate::ui::SCREEN_HEIGHT as u32 - glyph.height) / 2) as i16
}

#[derive(Clone, Copy)]
pub struct Icon<'a> {
    pub icon: &'a Glyph<'a>,
    pub pos: (i16, i16),
}

impl<'a> From<&'a Glyph<'a>> for Icon<'a> {
    fn from(glyph: &'a Glyph) -> Icon<'a> {
        Icon {
            icon: glyph,
            pos: (0, middle_y(glyph)),
        }
    }
}

impl<'a> Icon<'a> {
    pub const fn from(glyph: &'a Glyph<'a>) -> Icon<'a> {
        Icon {
            icon: glyph,
            pos: (0, middle_y(glyph)),
        }
    }

    /// Set specific x-coordinate
    pub const fn set_x(self, x: i16) -> Icon<'a> {
        Icon {
            pos: (x, self.pos.1),
            ..self
        }
    }

    /// Set specific y-coordinate
    pub const fn set_y(self, y: i16) -> Icon<'a> {
        Icon {
            pos: (self.pos.0, y),
            ..self
        }
    }

    /// Shift horizontally
    pub const fn shift_h(self, n: i16) -> Icon<'a> {
        Icon {
            pos: (self.pos.0 + n, self.pos.1),
            ..self
        }
    }

    /// Shift vertically
    pub const fn shift_v(self, n: i16) -> Icon<'a> {
        Icon {
            pos: (self.pos.0, self.pos.1 + n),
            ..self
        }
    }
}

use crate::ui::bitmaps;

pub const OUTER_PADDING: usize = 2;
pub const SCREENW: i16 = (crate::ui::SCREEN_WIDTH - OUTER_PADDING) as i16;

pub static DOWN_ARROW: Icon =
    Icon::from(&bitmaps::DOWN_ARROW).set_x(SCREENW - bitmaps::DOWN_ARROW.width as i16);
pub static LEFT_ARROW: Icon = Icon::from(&bitmaps::LEFT_ARROW).set_x(OUTER_PADDING as i16);
pub static RIGHT_ARROW: Icon =
    Icon::from(&bitmaps::RIGHT_ARROW).set_x(SCREENW - bitmaps::RIGHT_ARROW.width as i16);
pub static UP_ARROW: Icon = Icon::from(&bitmaps::UP_ARROW).set_x(OUTER_PADDING as i16);
pub static DOWN_S_ARROW: Icon = DOWN_ARROW.shift_v(4);
pub static LEFT_S_ARROW: Icon = LEFT_ARROW.shift_h(4);
pub static RIGHT_S_ARROW: Icon = RIGHT_ARROW.shift_h(-4);
pub static UP_S_ARROW: Icon = UP_ARROW.shift_v(-4);

//pub static BACK_ICON: Icon = Icon::from(&bitmaps::BACK);
pub static BACK_X_ICON: Icon = Icon::from(&bitmaps::BACK_X);
pub static COGGLE_ICON: Icon = Icon::from(&bitmaps::COGGLE);
pub static PROCESSING_ICON: Icon = Icon::from(&bitmaps::PROCESSING);
pub static CERTIFICATE_ICON: Icon = Icon::from(&bitmaps::CERTIFICATE);
pub static CROSSMARK_ICON: Icon = Icon::from(&bitmaps::CROSSMARK);
pub static DASHBOARD_ICON: Icon = Icon::from(&bitmaps::DASHBOARD);
pub static DASHBOARD_X_ICON: Icon = Icon::from(&bitmaps::DASHBOARD_X);
pub static EYE_ICON: Icon = Icon::from(&bitmaps::EYE);
pub static VALIDATE_14_ICON: Icon = Icon::from(&bitmaps::VALIDATE_14);
pub static WARNING_ICON: Icon = Icon::from(&bitmaps::WARNING);
pub static CHECKMARK_ICON: Icon = Icon::from(&bitmaps::CHECKMARK);
pub static CROSS_ICON: Icon = Icon::from(&bitmaps::CROSS);
