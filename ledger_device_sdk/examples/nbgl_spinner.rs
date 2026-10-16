#![no_std]
#![no_main]

use ledger_device_sdk::io::{Event, StatusWords};
use ledger_device_sdk::nbgl::{NbglReviewStatus, NbglSpinner, init_comm};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

    NbglSpinner::new().show("Please wait...");

    // Simulate an idle state of the app where it just
    // waits for some event to happen (such as APDU reception), going through
    // the event loop to process TickerEvents so that the spinner can be animated
    // every 800ms.
    let mut ticks = 50;
    while ticks > 0 {
        match comm.next_event() {
            // Commands must be answered: this app is not ready to process any.
            Event::Command(cmd) => {
                let _ = cmd.reply(&[], StatusWords::CmdNotAccepted);
            }
            Event::Ticker => ticks -= 1,
            _ => {}
        }
    }
    NbglReviewStatus::new().show(comm, true);
    ledger_device_sdk::exit_app(0);
}
