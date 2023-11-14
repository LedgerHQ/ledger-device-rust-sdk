#![allow(dead_code)]

mod opensans;

use opensans::CharArray;

pub struct Font {
    pub chars: CharArray,
    pub dims: [u8; 96],
    pub height: u8,
}

impl Font {
    const fn new(chars: CharArray, dims: [u8; 96], height: u8) -> Font {
        Font {
            chars,
            dims,
            height,
        }
    }
}

const OPEN_SANS_REGULAR_11PX: Font = Font::new(
    opensans::OPEN_SANS_REGULAR_11PX_CHARS,
    opensans::OPEN_SANS_REGULAR_11PX_DIMS,
    12,
);
const OPEN_SANS_EXTRABOLD_11PX: Font = Font::new(
    opensans::OPEN_SANS_EXTRABOLD_11PX_CHARS,
    opensans::OPEN_SANS_EXTRABOLD_11PX_DIMS,
    12,
);

pub const OPEN_SANS: [Font; 2] = [OPEN_SANS_REGULAR_11PX, OPEN_SANS_EXTRABOLD_11PX];
