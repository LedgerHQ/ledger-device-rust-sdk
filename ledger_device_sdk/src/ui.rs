#![cfg_attr(not(version("1.63")), feature(array_from_fn))]

pub mod bagls;

pub mod string_se;

pub mod bitmaps;
pub mod fonts;
pub mod layout;

pub mod gadgets;
pub mod screen_util;

pub const PADDING: usize = 2;
pub const Y_PADDING: usize = 3;
pub const SCREEN_WIDTH: usize = 128;

pub const SCREEN_HEIGHT: usize = 64;
