#![no_std]
#![no_main]

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

use nanos_sdk::buttons::*;
use nanos_ui::layout::{Layout, Location, StringPlace};
use nanos_ui::ui;

fn wait_any() {
    let mut buttons = ButtonsState::new();
    loop {
        match ui::get_event(&mut buttons) {
            Some(ButtonEvent::LeftButtonRelease)
            | Some(ButtonEvent::RightButtonRelease)
            | Some(ButtonEvent::BothButtonsRelease) => return,
            _ => (),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    ui::clear_screen();
    ui::popup("Hello");

    ui::clear_screen();

    ["First", "Second"].place(Location::Middle, Layout::Centered, false);
    wait_any();
    ui::clear_screen();

    ["First Line", "Second Line", "Third Line"].place(Location::Middle, Layout::Centered, false);
    wait_any();
    ui::clear_screen();

    ["First Line", "Second Line", "Third Line", "Fourth"].place(
        Location::Middle,
        Layout::Centered,
        false,
    );
    wait_any();
    ui::clear_screen();

    ["Monero &", "Ethereum &", "Zcash &", "NanoPass"].place(
        Location::Top,
        Layout::LeftAligned,
        false,
    );
    wait_any();
    ui::clear_screen();

    ["Monero &", "Ethereum &", "Zcash &", "NanoPass"].place(
        Location::Top,
        Layout::RightAligned,
        false,
    );
    wait_any();

    let scrolled_message = "Arbitrary long text goes here, with numbers -1234567890";
    ui::MessageScroller::new(scrolled_message).event_loop();

    loop {
        match ui::Menu::new(&[&"Top0", &"Top1", &"Top2", &"Top3", &"Next"]).show() {
            0 => loop {
                match ui::Menu::new(&[&"Top0_sub0", &"Back"]).show() {
                    0 => ui::popup("Top0_sub0_0"),
                    _ => break,
                }
            },
            1 => loop {
                match ui::Menu::new(&[&"Top1_sub0", &"Top1_sub1", &"Back"]).show() {
                    0 => ui::popup("Top1_sub0_0"),
                    1 => ui::popup("Top1_sub1_0"),
                    _ => break,
                }
            },
            2 => break,
            3 => break,
            4 => break,
            _ => (),
        }
    }

    let _ = ui::Validator::new("Confirm?").ask();
    let _ = ui::MessageValidator::new(
        &[&"Message Review"],
        &[&"Confirm", &"message?"],
        &[&"Cancel"],
    )
    .ask();

    ui::clear_screen();

    use nanos_ui::bagls::RectFull as Rect;
    use nanos_ui::layout::Draw;

    Rect::new()
        .width(10)
        .height(10)
        .pos(16, 16)
        .instant_display();
    Rect::new()
        .width(10)
        .height(10)
        .pos(32, 16)
        .instant_display();
    Rect::new()
        .width(10)
        .height(10)
        .pos(48, 16)
        .instant_display();
    wait_any();

    ui::clear_screen();

    let checkmark = nanos_ui::bagls::CHECKMARK_ICON.set_x(0).set_y(4);
    checkmark.instant_display();
    nanos_ui::bagls::CROSS_ICON
        .set_x(20)
        .set_y(4)
        .instant_display();
    nanos_ui::bagls::COGGLE.set_x(40).set_y(4).instant_display();
    wait_any();
    checkmark.instant_erase();
    wait_any();

    nanos_sdk::exit_app(0);
}
