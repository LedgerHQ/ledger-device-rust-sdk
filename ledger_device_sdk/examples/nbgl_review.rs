#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview, NbglReviewStatus, init_comm};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[no_mangle]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

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
            value: "It is a test transaction.",
        },
    ];

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    // Create NBGL review
    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "Standard use case",
            "Sign transaction\nto send CRAB",
        )
        .glyph(&FERRIS)
        .show(comm, &my_fields);
    NbglReviewStatus::new().show(comm, success);

    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "Light use case",
            "Sign transaction\nto send CRAB",
        )
        .glyph(&FERRIS)
        .light()
        .show(comm, &my_fields);
    NbglReviewStatus::new().show(comm, success);

    let my_fields = [Field {
        name: "Hash",
        value: "0x1234567890ABCDEF1234567890ABCDEF12345678",
    }];

    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "Blind signing use case",
            "Sign transaction\nto send CRAB",
        )
        .glyph(&FERRIS)
        .blind()
        .show(comm, &my_fields);
    NbglReviewStatus::new().show(comm, success);

    ledger_device_sdk::exit_app(0);
}
