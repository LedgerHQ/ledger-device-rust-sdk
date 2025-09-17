#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglGlyph, NbglReviewStatus, NbglStreamingReview, StatusType, TransactionType,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
    NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

   
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

    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .tx_type(TransactionType::Message);
    if !review.start("Streaming example", Some("Example Subtitle")) {
        NbglReviewStatus::new().show(false);
        return;
    }
    for i in 0..fields.len() {
        if !review.continue_review(&fields[i..i + 1]) {
            NbglReviewStatus::new().show(false);
            return;
        }
    }
    let success = review.finish("Sign to send token\n");
    NbglReviewStatus::new().show(success);

    // Blind signing example
    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .blind()
        .tx_type(TransactionType::Message);
    if !review.start("Streaming example", Some("Example Subtitle")) {
        NbglReviewStatus::new().show(false);
        return;
    }
    for i in 0..fields.len() {
        if !review.continue_review(&fields[i..i + 1]) {
            NbglReviewStatus::new().show(false);
            return;
        }
    }
    let success = review.finish("Sign to send token\n");
    NbglReviewStatus::new().show(success);

    ledger_secure_sdk_sys::exit_app(0);

}
