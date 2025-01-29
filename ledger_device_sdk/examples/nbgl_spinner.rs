#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglGlyph, NbglReview, NbglReviewStatus, NbglSpinner, StatusType,
};
use ledger_device_sdk::testing::debug_print;
use ledger_secure_sdk_sys::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit_app(1);
}

// static spin_end: bool = false;

#[no_mangle]
extern "C" fn sample_main() {
    unsafe {
        nbgl_refreshReset();
    }

    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    let my_field = [Field {
        name: "Amount",
        value: "111 CRAB",
    }];

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    // Create NBGL review. Maximum number of fields and string buffer length can be customised
    // with constant generic parameters of NbglReview. Default values are 32 and 1024 respectively.
    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "To send CRAB",
            "Sign transaction\nto send CRAB",
        )
        .glyph(&FERRIS)
        .show(&my_field);

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
    NbglReviewStatus::new().show(success);
}
