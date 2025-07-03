use crate::{
    buttons::ButtonEvent::*,
    io::{self, ApduHeader, Comm, Event, Reply},
    uxapp::{UxEvent, BOLOS_UX_OK},
};
use ledger_secure_sdk_sys::{
    buttons::{get_button_event, ButtonEvent, ButtonsState},
    seph,
};

use crate::ui::bitmaps::{Glyph, WARNING};

use crate::ui::{bagls::*, fonts::OPEN_SANS};

use crate::ui::layout;
use crate::ui::layout::{Draw, Location, StringPlace};

use numtoa::NumToA;

const MAX_CHAR_PER_LINE: usize = 17;

/// Handles communication to filter
/// out actual events, and converts key
/// events into presses/releases
pub fn get_event(buttons: &mut ButtonsState) -> Option<ButtonEvent> {
    let mut io_buffer = [0u8; 273];
    let status = seph::io_rx(&mut io_buffer, true);
    if status > 0 {
        let packet_type = io_buffer[0];
        match packet_type {
            0x01 | 0x02 => {
                // SE or SEPH event
                if io_buffer[1] == 0x05 {
                    let button_info = io_buffer[4] >> 1;
                    return get_button_event(buttons, button_info);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn clear_screen() {
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

/// Display a developer mode / pending review popup, cleared with user interaction.
///
/// This method must be called by an application at the very beginning until it has been reviewed
/// and approved by Ledger.
///
/// # Arguments
///
/// * `comm` - Communication manager used to get device events.
///
/// # Examples
///
/// Following is an application example main function calling the pending review popup at the very
/// beginning, before doing any other application logic.
///
/// ```
/// #[no_mangle]
/// extern "C" fn sample_main() {
///     let mut comm = Comm::new();
///     ledger_device_sdk::ui::gadgets::display_pending_review(&mut comm);
///     ...
/// }
/// `
pub fn display_pending_review(comm: &mut Comm) {
    clear_screen();

    // Add icon and text to match the C SDK equivalent.
    WARNING.draw(57, 10);
    "Pending".place(Location::Custom(28), Layout::Centered, true);
    "Ledger review".place(Location::Custom(42), Layout::Centered, true);

    crate::ui::screen_util::screen_update();

    // Process events until a double button press release.
    loop {
        if let Event::Button(BothButtonsRelease) = comm.next_event::<ApduHeader>() {
            break;
        }
    }
}

/// Shorthand to display a single message
/// and wait for button action
pub fn popup(message: &str) {
    clear_screen();
    SingleMessage::new(message).show_and_wait();
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
                draw_icon_and_text(CHECKMARK_ICON, self.confirm);
                RIGHT_ARROW.display();
            } else if page == page_count - 1 {
                draw_icon_and_text(CROSS_ICON, self.cancel);
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
                    cur_page = cur_page.saturating_sub(1);
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

#[derive(Copy, Clone, PartialEq, Default)]
pub enum PageStyle {
    #[default]
    PictureNormal, // Picture (should be 16x16) with two lines of text (page layout depends on device).
    PictureBold,        // Icon on top with one line of text on the bottom.
    BoldNormal,         // One line of bold text and one line of normal text.
    Normal,             // 2 lines of centered text.
    BoldCenteredNormal, // 2 lines of centered text, where the first one is bold
}

#[derive(Copy, Clone, Default)]
pub struct Page<'a> {
    style: PageStyle,
    label: [&'a str; 2],
    glyph: Option<&'a Glyph<'a>>,
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

// new bold normal or new normal
impl<'a> From<([&'a str; 2], bool, bool)> for Page<'a> {
    fn from((label, bold, centered): ([&'a str; 2], bool, bool)) -> Page<'a> {
        if centered {
            if bold {
                Page::new(PageStyle::BoldCenteredNormal, [label[0], label[1]], None)
            } else {
                Page::new(PageStyle::Normal, [label[0], label[1]], None)
            }
        } else {
            Page::new(PageStyle::BoldNormal, [label[0], label[1]], None)
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
    pub const fn new(style: PageStyle, label: [&'a str; 2], glyph: Option<&'a Glyph<'a>>) -> Self {
        Page {
            style,
            label,
            glyph,
        }
    }

    pub fn place(&self) {
        match self.style {
            PageStyle::Normal => {
                self.label.place(Location::Middle, Layout::Centered, false);
            }
            PageStyle::BoldCenteredNormal => {
                self.label.place(Location::Middle, Layout::Centered, true);
            }
            PageStyle::PictureNormal => {
                let icon_x = 57;
                let icon_y = 10;
                self.label
                    .place(Location::Custom(28), Layout::Centered, false);
                if let Some(glyph) = self.glyph {
                    let icon = Icon::from(glyph);
                    icon.set_x(icon_x).set_y(icon_y).display();
                }
            }
            PageStyle::PictureBold => {
                let icon_x = 57;
                let icon_y = 17;
                self.label[0].place(Location::Custom(35), Layout::Centered, true);
                self.label[1].place(Location::Custom(49), Layout::Centered, true);
                if let Some(glyph) = self.glyph {
                    let icon = Icon::from(glyph);
                    icon.set_x(icon_x).set_y(icon_y).display();
                }
            }
            PageStyle::BoldNormal => {
                let padding = 1;
                let max_text_lines = 3;
                let total_height = (OPEN_SANS[0].height * max_text_lines) as usize
                    + OPEN_SANS[1].height as usize
                    + 2 * padding as usize;
                let mut cur_y = Location::Middle.get_y(total_height);

                self.label[0].place(Location::Custom(cur_y), Layout::Centered, true);
                cur_y += OPEN_SANS[0].height as usize + 2 * padding as usize;

                let mut indices = [(0, 0); 3];
                let len = self.label[1].len();
                for (i, indice) in indices.iter_mut().enumerate() {
                    let start = (i * MAX_CHAR_PER_LINE).min(len);
                    if start >= len {
                        break; // Break if we reach the end of the string
                    }
                    let end = (start + MAX_CHAR_PER_LINE).min(len);
                    *indice = (start, end);
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

    pub fn place_and_wait(&self) {
        let mut buttons = ButtonsState::new();

        self.place();
        crate::ui::screen_util::screen_update();

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

pub enum EventOrPageIndex<T> {
    Event(io::Event<T>),
    Index(usize),
}

// Trick to manage pin code
struct Temp {}
impl TryFrom<io::ApduHeader> for Temp {
    type Error = io::StatusWords;
    fn try_from(_header: io::ApduHeader) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

pub struct MultiPageMenu<'a> {
    comm: &'a mut io::Comm,
    pages: &'a [&'a Page<'a>],
}

impl<'a> MultiPageMenu<'a> {
    pub fn new(comm: &'a mut io::Comm, pages: &'a [&'a Page]) -> Self {
        MultiPageMenu { comm, pages }
    }

    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> EventOrPageIndex<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        self.show_from(0)
    }

    pub fn show_from<T: TryFrom<ApduHeader>>(&mut self, page_index: usize) -> EventOrPageIndex<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        clear_screen();

        self.pages[page_index].place();

        LEFT_ARROW.display();
        RIGHT_ARROW.display();

        crate::ui::screen_util::screen_update();

        let mut index = page_index;

        loop {
            match self.comm.next_event() {
                io::Event::Button(button) => {
                    if UxEvent::Event.request() == BOLOS_UX_OK {
                        match button {
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
                        }
                    }
                }
                io::Event::Command(ins) => return EventOrPageIndex::Event(io::Event::Command(ins)),
                io::Event::Ticker => {
                    if UxEvent::Event.request() != BOLOS_UX_OK {
                        // pin lock management
                        UxEvent::block_and_get_event::<Temp>(self.comm);
                        // notify Ticker event only when redisplay is required
                        return EventOrPageIndex::Event(io::Event::Ticker);
                    }
                }
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
            label.text = chunk;
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
                    cur_page = cur_page.saturating_sub(1);
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

impl<'a> Field<'a> {
    pub fn event_loop(&self, incoming_direction: ButtonEvent, is_first_field: bool) -> ButtonEvent {
        let mut buttons = ButtonsState::new();
        let chunk_max_lines = layout::MAX_LINES - 1;
        let page_count = 1 + self.value.len() / (chunk_max_lines * MAX_CHAR_PER_LINE);

        let mut cur_page = match incoming_direction {
            ButtonEvent::LeftButtonRelease => page_count - 1,
            ButtonEvent::RightButtonRelease => 0,
            _ => 0,
        };

        // A closure to draw common elements of the screen
        // cur_page passed as parameter to prevent borrowing
        let draw = |page: usize| {
            clear_screen();
            let mut chunks = [Label::default(); layout::MAX_LINES];
            for (i, chunk) in self
                .value
                .as_bytes()
                .chunks(MAX_CHAR_PER_LINE)
                .skip(page * chunk_max_lines)
                .take(chunk_max_lines)
                .enumerate()
            {
                chunks[1 + i] = Label::from(core::str::from_utf8(chunk).unwrap_or(""));
            }

            let mut header_buf = [b' '; MAX_CHAR_PER_LINE + 4];

            if page == 0 && MAX_CHAR_PER_LINE * chunk_max_lines > self.value.len() {
                // There is a single page. Do not display counter `( x / n )`
                header_buf[..self.name.len()].copy_from_slice(self.name.as_bytes());
            } else {
                let mut buf_page = [0u8; 3];
                let mut buf_count = [0u8; 3];
                let page_str = (page + 1).numtoa_str(10, &mut buf_page);
                let count_str = page_count.numtoa_str(10, &mut buf_count);

                concatenate(
                    &[&self.name, " (", &page_str, "/", &count_str, ")"],
                    &mut header_buf,
                );
            }
            let header = core::str::from_utf8(&header_buf)
                .unwrap_or("")
                .trim_end_matches(' ');
            chunks[0] = Label::from(header).bold();

            if !is_first_field {
                LEFT_ARROW.display();
            }
            RIGHT_ARROW.display();

            chunks.place(Location::Middle, Layout::Centered, false);

            crate::ui::screen_util::screen_update();
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
                    if cur_page == 0 {
                        return ButtonEvent::LeftButtonRelease;
                    }
                    cur_page = cur_page.saturating_sub(1);
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    RIGHT_S_ARROW.erase();
                    if cur_page + 1 == page_count {
                        return ButtonEvent::RightButtonRelease;
                    }
                    if cur_page + 1 < page_count {
                        cur_page += 1;
                    }
                    draw(cur_page);
                }
                Some(_) | None => (),
            }
        }
    }
}

pub struct MultiFieldReview<'a> {
    fields: &'a [Field<'a>],
    review_message: &'a [&'a str],
    review_glyph: Option<&'a Glyph<'a>>,
    validation_message: [&'a str; 2],
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
        Self::new_with_validation_messages(
            fields,
            review_message,
            review_glyph,
            [validation_message, ""],
            validation_glyph,
            cancel_message,
            cancel_glyph,
        )
    }

    pub fn new_with_validation_messages(
        fields: &'a [Field<'a>],
        review_message: &'a [&'a str],
        review_glyph: Option<&'a Glyph<'a>>,
        validation_message: [&'a str; 2],
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
        let first_page_opt = match self.review_message.len() {
            0 => None,
            1 => Some(Page::new(
                PageStyle::PictureBold,
                [self.review_message[0], ""],
                self.review_glyph,
            )),
            _ => Some(Page::new(
                PageStyle::PictureNormal,
                [self.review_message[0], self.review_message[1]],
                self.review_glyph,
            )),
        };

        display_first_page(&first_page_opt);

        let validation_page = Page::new(
            PageStyle::PictureBold,
            self.validation_message,
            self.validation_glyph,
        );
        let cancel_page = Page::new(
            PageStyle::PictureBold,
            [self.cancel_message, ""],
            self.cancel_glyph,
        );

        let mut cur_page = 0usize;
        let mut direction = ButtonEvent::RightButtonRelease;

        loop {
            match cur_page {
                cancel if cancel == self.fields.len() + 1 => {
                    let mut buttons = ButtonsState::new();
                    clear_screen();
                    LEFT_ARROW.display();
                    cancel_page.place();
                    crate::ui::screen_util::screen_update();
                    loop {
                        match get_event(&mut buttons) {
                            Some(ButtonEvent::LeftButtonRelease) => {
                                cur_page = cur_page.saturating_sub(1);
                                break;
                            }
                            Some(ButtonEvent::BothButtonsRelease) => return false,
                            _ => (),
                        }
                    }
                }
                validation if validation == self.fields.len() => {
                    let mut buttons = ButtonsState::new();
                    clear_screen();
                    LEFT_ARROW.display();
                    RIGHT_ARROW.display();
                    validation_page.place();
                    crate::ui::screen_util::screen_update();
                    loop {
                        match get_event(&mut buttons) {
                            Some(ButtonEvent::LeftButtonRelease) => {
                                cur_page = cur_page.saturating_sub(1);
                                if cur_page == 0 && self.fields.is_empty() {
                                    display_first_page(&first_page_opt);
                                } else {
                                    direction = ButtonEvent::LeftButtonRelease;
                                }
                                break;
                            }
                            Some(ButtonEvent::RightButtonRelease) => {
                                cur_page += 1;
                                break;
                            }
                            Some(ButtonEvent::BothButtonsRelease) => return true,
                            _ => (),
                        }
                    }
                }
                _ => {
                    direction = self.fields[cur_page]
                        .event_loop(direction, cur_page == 0 && first_page_opt.is_none());
                    match direction {
                        ButtonEvent::LeftButtonRelease => {
                            if cur_page == 0 {
                                display_first_page(&first_page_opt);
                                direction = ButtonEvent::RightButtonRelease;
                            } else {
                                cur_page -= 1;
                            }
                        }
                        ButtonEvent::RightButtonRelease => {
                            cur_page += 1;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn display_first_page(page_opt: &Option<Page>) {
    match page_opt {
        Some(page) => {
            clear_screen();
            RIGHT_ARROW.display();
            page.place();
            crate::ui::screen_util::screen_update();

            let mut buttons = ButtonsState::new();
            loop {
                if let Some(ButtonEvent::RightButtonRelease) = get_event(&mut buttons) {
                    return;
                }
            }
        }
        None => (),
    }
}
