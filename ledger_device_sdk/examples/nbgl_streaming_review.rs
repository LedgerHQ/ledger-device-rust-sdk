#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, Field, NbglGlyph, NbglStreamingReview, TransactionType};
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

    review.start("Example Title", "Example Subtitle");

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
        review.continue_review(&fields[i..i + 1]);
    }

    review.finish("Sign to send token\n");
}
