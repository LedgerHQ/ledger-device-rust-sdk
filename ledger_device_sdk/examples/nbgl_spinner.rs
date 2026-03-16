#![no_std]
#![no_main]

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
    let mut loop_count = 50;
    while loop_count > 0 {
        comm.next_event();
        loop_count -= 1;
    }
    NbglReviewStatus::new().show(comm, true);
    ledger_device_sdk::exit_app(0);
}
