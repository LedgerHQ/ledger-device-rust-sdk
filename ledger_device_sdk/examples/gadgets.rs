#![no_std]
#![no_main]

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    ledger_device_sdk::exit_app(1);
}

use ledger_device_sdk::buttons::*;
use ledger_device_sdk::ui::gadgets;
use ledger_device_sdk::ui::layout::{Layout, Location, StringPlace};

fn wait_any() {
    let mut buttons = ButtonsState::new();
    loop {
        match gadgets::get_event(&mut buttons) {
            Some(ButtonEvent::LeftButtonRelease)
            | Some(ButtonEvent::RightButtonRelease)
            | Some(ButtonEvent::BothButtonsRelease) => return,
            _ => (),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    gadgets::clear_screen();
    gadgets::popup("Hello");

    gadgets::clear_screen();

    ["First", "Second"].place(Location::Middle, Layout::Centered, false);
    wait_any();
    gadgets::clear_screen();

    ["First Line", "Second Line", "Third Line"].place(Location::Middle, Layout::Centered, false);
    wait_any();
    gadgets::clear_screen();

    ["First Line", "Second Line", "Third Line", "Fourth"].place(
        Location::Middle,
        Layout::Centered,
        false,
    );
    wait_any();
    gadgets::clear_screen();

    ["Monero &", "Ethereum &", "Zcash &", "NanoPass"].place(
        Location::Top,
        Layout::LeftAligned,
        false,
    );
    wait_any();
    gadgets::clear_screen();

    ["Monero &", "Ethereum &", "Zcash &", "NanoPass"].place(
        Location::Top,
        Layout::RightAligned,
        false,
    );
    wait_any();

    let scrolled_message = "Arbitrary long text goes here, with numbers -1234567890";
    gadgets::MessageScroller::new(scrolled_message).event_loop();

    loop {
        match gadgets::Menu::new(&[&"Top0", &"Top1", &"Top2", &"Top3", &"Next"]).show() {
            0 => loop {
                match gadgets::Menu::new(&[&"Top0_sub0", &"Back"]).show() {
                    0 => gadgets::popup("Top0_sub0_0"),
                    _ => break,
                }
            },
            1 => loop {
                match gadgets::Menu::new(&[&"Top1_sub0", &"Top1_sub1", &"Back"]).show() {
                    0 => gadgets::popup("Top1_sub0_0"),
                    1 => gadgets::popup("Top1_sub1_0"),
                    _ => break,
                }
            },
            2 => break,
            3 => break,
            4 => break,
            _ => (),
        }
    }

    let _ = gadgets::Validator::new("Confirm?").ask();
    let _ = gadgets::MessageValidator::new(
        &[&"Message Review"],
        &[&"Confirm", &"message?"],
        &[&"Cancel"],
    )
    .ask();

    gadgets::clear_screen();

    use ledger_device_sdk::ui::bagls::RectFull as Rect;
    use ledger_device_sdk::ui::layout::Draw;

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

    gadgets::clear_screen();

    let checkmark = ledger_device_sdk::ui::bagls::CHECKMARK_ICON
        .set_x(0)
        .set_y(4);
    checkmark.instant_display();
    ledger_device_sdk::ui::bagls::CROSS_ICON
        .set_x(20)
        .set_y(4)
        .instant_display();
    ledger_device_sdk::ui::bagls::COGGLE_ICON
        .set_x(40)
        .set_y(4)
        .instant_display();
    wait_any();
    checkmark.instant_erase();
    wait_any();

    ledger_device_sdk::exit_app(0);
}
