#[derive(Copy, Clone)]
pub enum Layout {
    LeftAligned,
    RightAligned,
    Centered,
    Custom(usize),
}

impl Layout {
    pub fn get_x(&self, width: usize) -> usize {
        match self {
            Layout::LeftAligned => crate::ui::PADDING,
            Layout::Centered => (crate::ui::SCREEN_WIDTH - width) / 2,
            Layout::RightAligned => crate::ui::SCREEN_WIDTH - crate::ui::PADDING - width,
            Layout::Custom(x) => *x,
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
            Location::Top => crate::ui::Y_PADDING,
            Location::Middle => (crate::ui::SCREEN_HEIGHT - height) / 2,
            Location::Bottom => crate::ui::SCREEN_HEIGHT - height - crate::ui::Y_PADDING,
            Location::Custom(y) => *y,
        }
    }
}

pub const MAX_LINES: usize = 4;

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
        crate::ui::screen_util::screen_update();
    }
    fn instant_erase(&self) {
        self.erase();
        crate::ui::screen_util::screen_update();
    }
    fn display(&self);
    fn erase(&self);
}
