use crate::bagls::*;
use crate::layout::*;

impl StringPlace for &str {
    // unused when using MCU display
    fn compute_width(&self, _bold: bool) -> usize {
        0
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        let mut lbl = Label::new()
            .dims(128, 11)
            .location(loc)
            .layout(layout)
            .text(self);
        if bold {
            lbl = lbl.bold();
        }
        lbl.paint();
    }
}

impl StringPlace for [&str] {
    // unused when using MCU display
    fn compute_width(&self, _bold: bool) -> usize {
        0
    }

    fn place(&self, loc: Location, layout: Layout, bold: bool) {
        let c_height = 11; // Default font height
        let total_height = self.len() * c_height;
        let mut cur_y = loc.get_y(total_height);
        for string in self.iter() {
            string.place(Location::Custom(cur_y), layout, bold);
            cur_y += c_height;
        }
    }
}

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

    fn place(&self, loc: Location, layout: Layout, _bold: bool) {
        let padding = if self.len() > 4 { 0 } else { 2 };
        let total_height = self.iter().fold(0, |acc, _| acc + padding + 11);
        let mut cur_y = loc.get_y(total_height);
        for label in self.iter() {
            label.place(Location::Custom(cur_y), layout, label.bold);
            cur_y += 11 + padding;
        }
    }
}
