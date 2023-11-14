#[derive(Copy, Clone)]
pub enum Layout {
    LeftAligned,
    RightAligned,
    Centered,
}

impl Layout {
    pub fn get_x(&self, width: usize) -> usize {
        match self {
            Layout::LeftAligned => crate::PADDING,
            Layout::Centered => (crate::SCREEN_WIDTH - width) / 2,
            Layout::RightAligned => crate::SCREEN_WIDTH - crate::PADDING - width,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Location {
    Top,
    Middle,
    Bottom,
    Custom(usize),
}

impl Location {
    pub fn get_y(&self, height: usize) -> usize {
        match self {
            Location::Top => 0,
            Location::Middle => (crate::SCREEN_HEIGHT - height) / 2,
            Location::Bottom => crate::SCREEN_HEIGHT - height,
            Location::Custom(y) => *y,
        }
    }
}

#[cfg(target_os = "nanos")]
pub const MAX_LINES: usize = 2;

#[cfg(not(target_os = "nanos"))]
pub const MAX_LINES: usize = 3;

pub trait Place {
    fn place_pad(&self, loc: Location, layout: Layout, padding: i32);
    fn place(&self, loc: Location, layout: Layout) {
        self.place_pad(loc, layout, 0)
    }
}

pub trait StringPlace {
    fn compute_width(&self, bold: bool) -> usize;
    fn place(&self, loc: Location, layout: Layout, bold: bool);
}

pub trait Draw {
    /// Display right away (updates screen)
    fn instant_display(&self) {
        self.display();
        crate::screen_util::screen_update();
    }
    fn instant_erase(&self) {
        self.erase();
        crate::screen_util::screen_update();
    }
    fn display(&self);
    fn erase(&self);
}
