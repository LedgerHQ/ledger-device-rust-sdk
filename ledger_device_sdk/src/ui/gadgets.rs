#![allow(dead_code)]

use core::str::from_utf8;

use crate::{buttons::ButtonEvent::*, io};
use ledger_secure_sdk_sys::{
    buttons::{get_button_event, ButtonEvent, ButtonsState},
    seph,
};

use crate::ui::bitmaps::Glyph;

use crate::ui::{bagls::*, fonts::OPEN_SANS};

use crate::ui::layout;
use crate::ui::layout::{Draw, Location, StringPlace};

use numtoa::NumToA;

const MAX_CHAR_PER_LINE: usize = 17;

/// Handles communication to filter
/// out actual events, and converts key
/// events into presses/releases
pub fn get_event(buttons: &mut ButtonsState) -> Option<ButtonEvent> {
    if !seph::is_status_sent() {
        seph::send_general_status();
    }

    // TODO: Receiving an APDU while in UX will lead to .. exit ?
    while seph::is_status_sent() {
        seph::seph_recv(&mut buttons.cmd_buffer, 0);
        let tag = buttons.cmd_buffer[0];

        // button push event
        if tag == 0x05 {
            let button_info = buttons.cmd_buffer[3] >> 1;
            return get_button_event(buttons, button_info);
        }
    }
    None
}

pub fn clear_screen() {
    #[cfg(not(target_os = "nanos"))]
    {
        #[cfg(not(feature = "speculos"))]
        unsafe {
            ledger_secure_sdk_sys::screen_clear();
        }

        #[cfg(feature = "speculos")]
        {
            // Speculos does not emulate the screen_clear syscall yet
            RectFull::new()
                .width(crate::ui::SCREEN_WIDTH as u32)
                .height(crate::ui::SCREEN_HEIGHT as u32)
                .erase();
        }
    }

    #[cfg(target_os = "nanos")]
    BLANK.paint();
}

/// Shorthand to display a single message
/// and wait for button action
pub fn popup(message: &str) {
    clear_screen();
    SingleMessage::new(&message).show_and_wait();
}

/// Display a single screen with a message,
/// and exit the function with 'true'
/// if the user validated 'message'
/// or false if the user aborted
pub struct Validator<'a> {
    message: &'a str,
}

impl<'a> Validator<'a> {
    pub fn new(message: &'a str) -> Self {
        Validator { message }
    }

    pub fn ask(&self) -> bool {
        clear_screen();
        let mut buttons = ButtonsState::new();

        let mut lines = [Label::from_const("Cancel"), Label::from(self.message)];

        lines[0].bold = true;

        let redraw = |lines_list: &[Label; 2]| {
            clear_screen();
            lines_list.place(Location::Middle, Layout::Centered, false);

            UP_ARROW.display();
            DOWN_ARROW.display();

            crate::ui::screen_util::screen_update();
        };
        redraw(&lines);

        let mut response = false;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    UP_S_ARROW.erase();
                    response = false;
                    lines[0].bold = true;
                    lines[1].bold = false;
                    lines.place(Location::Middle, Layout::Centered, false);
                    redraw(&lines);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    DOWN_S_ARROW.erase();
                    response = true;
                    lines[0].bold = false;
                    lines[1].bold = true;
                    redraw(&lines);
                }
                Some(ButtonEvent::BothButtonsPress) => {
                    UP_ARROW.erase();
                    DOWN_ARROW.erase();
                }
                Some(ButtonEvent::BothButtonsRelease) => return response,
                _ => (),
            }
        }
    }
}

pub struct MessageValidator<'a> {
    /// Strings displayed in the pages. One string per page. Can be empty.
    message: &'a [&'a str],
    /// Strings displayed in the confirmation page.
    /// 0 element: only the icon is displayed, in center of the screen.
    /// 1 element: icon and one line of text displayed.
    /// 2 elements: icon and two lines of text displayed.
    confirm: &'a [&'a str],
    /// Strings displayed in the cancel page.
    /// 0 element: only the icon is displayed, in center of the screen.
    /// 1 element: icon and one line of text displayed.
    /// 2 elements: icon and two lines of text displayed.
    cancel: &'a [&'a str],
}

use crate::ui::layout::*;

impl<'a> MessageValidator<'a> {
    pub const fn new(
        message: &'a [&'a str],
        confirm: &'a [&'a str],
        cancel: &'a [&'a str],
    ) -> Self {
        MessageValidator {
            message,
            confirm,
            cancel,
        }
    }

    pub fn ask(&self) -> bool {
        clear_screen();
        let page_count = &self.message.len() + 2;
        let mut cur_page = 0;

        let draw_icon_and_text = |icon: Icon, strings: &[&str]| {
            // Draw icon on the center if there is no text.
            let x = match strings.len() {
                0 => 60,
                _ => 18,
            };
            icon.set_x(x).display();
            match strings.len() {
                0 => {}
                1 => {
                    strings[0].place(Location::Middle, Layout::Centered, false);
                }
                _ => {
                    strings[..2].place(Location::Middle, Layout::Centered, false);
                }
            }
        };

        let draw = |page: usize| {
            clear_screen();
            if page == page_count - 2 {
                draw_icon_and_text(CHECKMARK_ICON, &self.confirm);
                RIGHT_ARROW.display();
            } else if page == page_count - 1 {
                draw_icon_and_text(CROSS_ICON, &self.cancel);
            } else {
                self.message[page].place(Location::Middle, Layout::Centered, false);
                RIGHT_ARROW.display();
            }
            if page > 0 {
                LEFT_ARROW.display();
            }
            crate::ui::screen_util::screen_update();
        };

        draw(cur_page);

        let mut buttons = ButtonsState::new();
        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    LEFT_S_ARROW.erase();
                    if cur_page > 0 {
                        cur_page -= 1;
                    }
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    RIGHT_S_ARROW.erase();
                    if cur_page < page_count - 1 {
                        cur_page += 1;
                    }
                    draw(cur_page);
                }
                Some(ButtonEvent::BothButtonsRelease) => {
                    if cur_page == page_count - 2 {
                        // Confirm
                        return true;
                    } else if cur_page == page_count - 1 {
                        // Abort
                        return false;
                    }
                    draw(cur_page);
                }
                _ => (),
            }
        }
    }
}

pub struct Menu<'a> {
    panels: &'a [&'a str],
}

impl<'a> Menu<'a> {
    pub fn new(panels: &'a [&'a str]) -> Self {
        Menu { panels }
    }

    pub fn show(&self) -> usize {
        clear_screen();
        let mut buttons = ButtonsState::new();

        let mut items: [Label; layout::MAX_LINES] =
            core::array::from_fn(|i| Label::from(*self.panels.get(i).unwrap_or(&"")));

        items[0].bold = true;
        items.place(Location::Middle, Layout::Centered, false);

        UP_ARROW.display();
        DOWN_ARROW.display();

        crate::ui::screen_util::screen_update();

        let mut index = 0;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_S_ARROW.instant_display();
                }
                Some(ButtonEvent::BothButtonsRelease) => return index,
                Some(x) => {
                    match x {
                        ButtonEvent::LeftButtonRelease => {
                            index = index.saturating_sub(1);
                        }
                        ButtonEvent::RightButtonRelease => {
                            if index < self.panels.len() - 1 {
                                index += 1;
                            }
                        }
                        _ => (),
                    }
                    clear_screen();
                    UP_ARROW.display();
                    DOWN_ARROW.display();

                    let chunk = (index / layout::MAX_LINES) * layout::MAX_LINES;
                    for (i, item) in items.iter_mut().enumerate() {
                        item.text = self.panels.get(chunk + i).unwrap_or(&"");
                        item.bold = false;
                    }
                    items[index - chunk].bold = true;
                    items.place(Location::Middle, Layout::Centered, false);
                    crate::ui::screen_util::screen_update();
                }
                _ => (),
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PageStyle {
    PictureNormal, // Picture (should be 16x16) with two lines of text (page layout depends on device).
    PictureBold,   // Icon on top with one line of text on the bottom.
    BoldNormal,    // One line of bold text and one line of normal text.
    Normal,        // 2 lines of centered text.
}

#[derive(Copy, Clone)]
pub struct Page<'a> {
    style: PageStyle,
    label: [&'a str; 2],
    glyph: Option<&'a Glyph<'a>>,
    chunk_count: u8,
    chunk_idx: u8,
}

// new_picture_normal
impl<'a> From<([&'a str; 2], &'a Glyph<'a>)> for Page<'a> {
    fn from((label, glyph): ([&'a str; 2], &'a Glyph<'a>)) -> Page<'a> {
        Page::new(PageStyle::PictureNormal, [label[0], label[1]], Some(glyph))
    }
}

// new bold normal or new normal
impl<'a> From<([&'a str; 2], bool)> for Page<'a> {
    fn from((label, bold): ([&'a str; 2], bool)) -> Page<'a> {
        if bold {
            Page::new(PageStyle::BoldNormal, [label[0], label[1]], None)
        } else {
            Page::new(PageStyle::Normal, [label[0], label[1]], None)
        }
    }
}

// new picture bold
impl<'a> From<(&'a str, &'a Glyph<'a>)> for Page<'a> {
    fn from((label, glyph): (&'a str, &'a Glyph<'a>)) -> Page<'a> {
        let label = [label, ""];
        Page::new(PageStyle::PictureBold, label, Some(glyph))
    }
}

impl<'a> Page<'a> {
    pub fn new(style: PageStyle, label: [&'a str; 2], glyph: Option<&'a Glyph<'a>>) -> Self {
        let chunk_count = 0;
        let chunk_idx = 0;
        Page {
            style,
            label,
            glyph,
            chunk_count,
            chunk_idx,
        }
    }

    pub fn place(&self) {
        match self.style {
            PageStyle::Normal => {
                self.label.place(Location::Middle, Layout::Centered, false);
            }
            PageStyle::PictureNormal => {
                let mut icon_x = 16;
                let mut icon_y = 8;
                if cfg!(target_os = "nanos") {
                    self.label
                        .place(Location::Middle, Layout::Custom(41), false);
                } else {
                    icon_x = 57;
                    icon_y = 10;
                    self.label
                        .place(Location::Custom(28), Layout::Centered, false);
                }
                match self.glyph {
                    Some(glyph) => {
                        let icon = Icon::from(glyph);
                        icon.set_x(icon_x).set_y(icon_y).display();
                    }
                    None => {}
                }
            }
            PageStyle::PictureBold => {
                let mut icon_x = 56;
                let mut icon_y = 2;
                if cfg!(target_os = "nanos") {
                    self.label[0].place(Location::Bottom, Layout::Centered, true);
                } else {
                    icon_x = 57;
                    icon_y = 17;
                    self.label[0].place(Location::Custom(35), Layout::Centered, true);
                }
                match self.glyph {
                    Some(glyph) => {
                        let icon = Icon::from(glyph);
                        icon.set_x(icon_x).set_y(icon_y).display();
                    }
                    None => {}
                }
            }
            PageStyle::BoldNormal => {
                let padding = 1;
                let mut max_text_lines = 3;
                if cfg!(target_os = "nanos") {
                    max_text_lines = 1;
                }
                let total_height = (OPEN_SANS[0].height * max_text_lines) as usize
                    + OPEN_SANS[1].height as usize
                    + 2 * padding as usize;
                let mut cur_y = Location::Middle.get_y(total_height);

                // Display the chunk count and index if needed
                if self.chunk_count > 1 {
                    let mut label_bytes = [0u8; MAX_CHAR_PER_LINE];
                    // Convert the chunk count to a string
                    let mut chunk_count_buf = [0u8; 3];
                    let chunk_count_str = self.chunk_count.numtoa_str(10, &mut chunk_count_buf);
                    // Convert the chunk index to a string
                    let mut chunk_idx_buf = [0u8; 3];
                    let chunk_idx_str = self.chunk_idx.numtoa_str(10, &mut chunk_idx_buf);
                    // Add the chunk count and index to the label
                    concatenate(
                        &[
                            self.label[0],
                            " (",
                            chunk_idx_str,
                            "/",
                            chunk_count_str,
                            ")",
                        ],
                        &mut label_bytes,
                    );
                    from_utf8(&mut label_bytes)
                        .unwrap()
                        .trim_matches(char::from(0))
                        .place(Location::Custom(cur_y), Layout::Centered, true);
                } else {
                    self.label[0].place(Location::Custom(cur_y), Layout::Centered, true);
                }
                cur_y += OPEN_SANS[0].height as usize + 2 * padding as usize;

                // If the device is a Nano S, display the second label as
                // a single line of text
                if cfg!(target_os = "nanos") {
                    self.label[1].place(Location::Custom(cur_y), Layout::Centered, false);
                }
                // Otherwise, display the second label as up to 3 lines of text
                else {
                    let mut indices = [(0, 0); 3];
                    let len = self.label[1].len();
                    for i in 0..3 {
                        let start = (i * MAX_CHAR_PER_LINE).min(len);
                        if start >= len {
                            break; // Break if we reach the end of the string
                        }
                        let end = (start + MAX_CHAR_PER_LINE).min(len);
                        indices[i] = (start, end);
                        (&self.label[1][start..end]).place(
                            Location::Custom(cur_y),
                            Layout::Centered,
                            false,
                        );
                        cur_y += OPEN_SANS[0].height as usize + 2 * padding as usize;
                    }
                }
            }
        }
    }

    pub fn place_and_wait(&self) {
        let mut buttons = ButtonsState::new();

        self.place();

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonRelease)
                | Some(ButtonEvent::RightButtonRelease)
                | Some(ButtonEvent::BothButtonsRelease) => return,
                _ => (),
            }
        }
    }
}

pub enum EventOrPageIndex {
    Event(io::Event<io::ApduHeader>),
    Index(usize),
}

pub struct MultiPageMenu<'a> {
    comm: &'a mut io::Comm,
    pages: &'a [&'a Page<'a>],
}

impl<'a> MultiPageMenu<'a> {
    pub fn new(comm: &'a mut io::Comm, pages: &'a [&'a Page]) -> Self {
        MultiPageMenu { comm, pages }
    }

    pub fn show(&mut self) -> EventOrPageIndex {
        clear_screen();

        self.pages[0].place();

        LEFT_ARROW.display();
        RIGHT_ARROW.display();

        crate::ui::screen_util::screen_update();

        let mut index = 0;

        loop {
            match self.comm.next_event() {
                io::Event::Button(button) => match button {
                    BothButtonsRelease => return EventOrPageIndex::Index(index),
                    b => {
                        match b {
                            LeftButtonRelease => {
                                if index as i16 - 1 < 0 {
                                    index = self.pages.len() - 1;
                                } else {
                                    index = index.saturating_sub(1);
                                }
                            }
                            RightButtonRelease => {
                                if index < self.pages.len() - 1 {
                                    index += 1;
                                } else {
                                    index = 0;
                                }
                            }
                            _ => (),
                        }
                        clear_screen();
                        self.pages[index].place();
                        LEFT_ARROW.display();
                        RIGHT_ARROW.display();
                        crate::ui::screen_util::screen_update();
                    }
                },
                io::Event::Command(ins) => return EventOrPageIndex::Event(io::Event::Command(ins)),
                _ => (),
            };
        }
    }
}

/// A gadget that displays
/// a short message in the
/// middle of the screen and
/// waits for a button press
pub struct SingleMessage<'a> {
    message: &'a str,
}

impl<'a> SingleMessage<'a> {
    pub fn new(message: &'a str) -> Self {
        SingleMessage { message }
    }

    pub fn show(&self) {
        clear_screen();
        self.message
            .place(Location::Middle, Layout::Centered, false);
    }
    /// Display the message and wait
    /// for any kind of button release
    pub fn show_and_wait(&self) {
        let mut buttons = ButtonsState::new();

        self.show();

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonRelease)
                | Some(ButtonEvent::RightButtonRelease)
                | Some(ButtonEvent::BothButtonsRelease) => return,
                _ => (),
            }
        }
    }
}

/// A horizontal scroller that
/// splits any given message
/// over several panes in chunks
/// of MAX_CHAR_PER_LINE characters.
/// Press both buttons to exit.
pub struct MessageScroller<'a> {
    message: &'a str,
}

impl<'a> MessageScroller<'a> {
    pub fn new(message: &'a str) -> Self {
        MessageScroller { message }
    }

    pub fn event_loop(&self) {
        clear_screen();
        let mut buttons = ButtonsState::new();
        let page_count = (self.message.len() - 1) / MAX_CHAR_PER_LINE + 1;
        if page_count == 0 {
            return;
        }
        let mut label = Label::from("");
        let mut cur_page = 0;

        // A closure to draw common elements of the screen
        // cur_page passed as parameter to prevent borrowing
        let mut draw = |page: usize| {
            let start = page * MAX_CHAR_PER_LINE;
            let end = (start + MAX_CHAR_PER_LINE).min(self.message.len());
            let chunk = &self.message[start..end];
            label.erase();
            label.text = &chunk;
            LEFT_ARROW.erase();
            RIGHT_ARROW.erase();
            if page > 0 {
                LEFT_ARROW.display();
            }
            if page + 1 < page_count {
                RIGHT_ARROW.display();
            }
            label.instant_display();
        };

        draw(cur_page);

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    LEFT_S_ARROW.erase();
                    if cur_page > 0 {
                        cur_page -= 1;
                    }
                    // We need to draw anyway to clear button press arrow
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    RIGHT_S_ARROW.erase();
                    if cur_page + 1 < page_count {
                        cur_page += 1;
                    }
                    // We need to draw anyway to clear button press arrow
                    draw(cur_page);
                }
                Some(ButtonEvent::BothButtonsRelease) => break,
                Some(_) | None => (),
            }
        }
    }
}

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}
pub struct MultiFieldReview<'a> {
    fields: &'a [Field<'a>],
    review_message: &'a [&'a str],
    review_glyph: Option<&'a Glyph<'a>>,
    validation_message: &'a str,
    validation_glyph: Option<&'a Glyph<'a>>,
    cancel_message: &'a str,
    cancel_glyph: Option<&'a Glyph<'a>>,
}

// Function to concatenate multiple strings into a fixed-size array
fn concatenate(strings: &[&str], output: &mut [u8]) {
    let mut offset = 0;

    for s in strings {
        let s_len = s.len();
        let copy_len = core::cmp::min(s_len, output.len() - offset);

        if copy_len > 0 {
            output[offset..offset + copy_len].copy_from_slice(&s.as_bytes()[..copy_len]);
            offset += copy_len;
        } else {
            // If the output buffer is full, stop concatenating.
            break;
        }
    }
}

const MAX_REVIEW_PAGES: usize = 48;

impl<'a> MultiFieldReview<'a> {
    pub fn new(
        fields: &'a [Field<'a>],
        review_message: &'a [&'a str],
        review_glyph: Option<&'a Glyph<'a>>,
        validation_message: &'a str,
        validation_glyph: Option<&'a Glyph<'a>>,
        cancel_message: &'a str,
        cancel_glyph: Option<&'a Glyph<'a>>,
    ) -> Self {
        MultiFieldReview {
            fields,
            review_message,
            review_glyph,
            validation_message,
            validation_glyph,
            cancel_message,
            cancel_glyph,
        }
    }

    pub fn show(&self) -> bool {
        let mut buttons = ButtonsState::new();

        let first_page = match self.review_message.len() {
            0 => Page::new(PageStyle::PictureNormal, ["", ""], self.review_glyph),
            1 => Page::new(
                PageStyle::PictureBold,
                [self.review_message[0], ""],
                self.review_glyph,
            ),
            _ => Page::new(
                PageStyle::PictureNormal,
                [self.review_message[0], self.review_message[1]],
                self.review_glyph,
            ),
        };

        let validation_page = Page::new(
            PageStyle::PictureBold,
            [self.validation_message, ""],
            self.validation_glyph,
        );
        let cancel_page = Page::new(
            PageStyle::PictureBold,
            [self.cancel_message, ""],
            self.cancel_glyph,
        );
        let mut review_pages: [Page; MAX_REVIEW_PAGES] =
            [Page::new(PageStyle::Normal, ["", ""], None); MAX_REVIEW_PAGES];
        let mut total_page_count = 0;

        let mut max_chars_per_page = MAX_CHAR_PER_LINE * 3;
        if cfg!(target_os = "nanos") {
            max_chars_per_page = MAX_CHAR_PER_LINE;
        }

        // Determine each field page count
        for field in self.fields {
            let field_page_count = (field.value.len() - 1) / max_chars_per_page + 1;
            // Create pages for each chunk of the field
            for i in 0..field_page_count {
                let start = i * max_chars_per_page;
                let end = (start + max_chars_per_page).min(field.value.len());
                let chunk = &field.value[start..end];

                review_pages[total_page_count] =
                    Page::new(PageStyle::BoldNormal, [field.name, chunk], None);
                review_pages[total_page_count].chunk_count = field_page_count as u8;
                review_pages[total_page_count].chunk_idx = (i + 1) as u8;
                // Check if we have reached the maximum number of pages
                // We need to keep 2 pages for the validation and cancel pages
                total_page_count = if total_page_count < MAX_REVIEW_PAGES - 2 {
                    total_page_count + 1
                } else {
                    break;
                };
            }
        }

        review_pages[total_page_count] = validation_page;
        total_page_count += 1;
        review_pages[total_page_count] = cancel_page;

        clear_screen();
        first_page.place_and_wait();
        crate::ui::screen_util::screen_update();
        clear_screen();
        RIGHT_ARROW.display();

        let mut cur_page = 0;
        let mut refresh: bool = true;
        review_pages[cur_page].place();

        loop {
            match get_event(&mut buttons) {
                Some(b) => {
                    match b {
                        ButtonEvent::LeftButtonRelease => {
                            if cur_page > 0 {
                                cur_page -= 1;
                            }
                            refresh = true;
                        }
                        ButtonEvent::RightButtonRelease => {
                            if cur_page < total_page_count {
                                cur_page += 1;
                            }
                            refresh = true;
                        }
                        ButtonEvent::BothButtonsRelease => {
                            if cur_page == total_page_count {
                                // Cancel
                                return false;
                            } else if cur_page == total_page_count - 1 {
                                // Validate
                                return true;
                            }
                        }
                        _ => refresh = false,
                    }
                    if refresh {
                        clear_screen();
                        review_pages[cur_page].place();
                        if cur_page > 0 {
                            LEFT_ARROW.display();
                        }
                        if cur_page < total_page_count {
                            RIGHT_ARROW.display();
                        }
                        crate::ui::screen_util::screen_update();
                    }
                }
                _ => (),
            }
        }
    }
}