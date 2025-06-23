#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglGlyph, NbglReview, NbglReviewStatus, StatusType,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    let my_fields = [
        Field {
            name: "Amount",
            value: "111 CRAB",
        },
        Field {
            name: "Destination",
            value: "0x1234567890ABCDEF1234567890ABCDEF12345678",
        },
        Field {
            name: "Memo",
            value: "This is a test transaction.",
        },
    ];

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
        .blind()
        .show(&my_fields);
    NbglReviewStatus::new().show(success);
}
