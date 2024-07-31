#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglGlyph, NbglReviewStatus, NbglStreamingReview, StatusType, TransactionType,
};
use ledger_secure_sdk_sys::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit_app(1);
}

#[no_mangle]
extern "C" fn sample_main() {
    unsafe {
        nbgl_refreshReset();
    }

    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));

    let mut review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .tx_type(TransactionType::Message);

    if !review.start("Streaming example", "Example Subtitle") {
        NbglReviewStatus::new().show(false);
        return;
    }

    let fields = [
        Field {
            name: "Name1",
            value: "Value1",
        },
        Field {
            name: "Name2",
            value: "Value2",
        },
        Field {
            name: "Name3",
            value: "Value3",
        },
        Field {
            name: "Name4",
            value: "Value4",
        },
        Field {
            name: "Name5",
            value: "Value5",
        },
    ];

    for i in 0..fields.len() {
        if !review.continue_review(&fields[i..i + 1]) {
            NbglReviewStatus::new().show(false);
            return;
        }
    }

    let success = review.finish("Sign to send token\n");
    NbglReviewStatus::new().show(success);
}
