#![no_std]
#![no_main]

use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglReviewStatus, NbglSpinner};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    NbglSpinner::new().show("Please wait...");

    // Simulate an idle state of the app where it just
    // waits for some event to happen (such as APDU reception), going through
    // the event loop to process TickerEvents so that the spinner can be animated
    // every 800ms.
    let mut loop_count = 50;
    while loop_count > 0 {
        comm.next_event::<ApduHeader>();
        loop_count -= 1;
    }
    NbglReviewStatus::new().show(true);
    ledger_secure_sdk_sys::exit_app(0);
}
