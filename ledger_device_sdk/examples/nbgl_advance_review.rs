#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    Field, NbglAdvanceReview, NbglGlyph, NbglReviewStatus, StatusType, TransactionType, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[no_mangle]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    let my_fields = [
        Field {
            name: "Field 1",
            value: "Value 1",
        },
        Field {
            name: "Field 2",
            value: "Value 2",
        },
        Field {
            name: "Field 3",
            value: "Value 3",
        },
    ];

    let advance_review = NbglAdvanceReview::new(TransactionType::Transaction)
        .glyph(&FERRIS)
        .review_title("Review Title")
        .review_subtitle("Review Subtitle")
        .finish_title("Finish Title")
        .warning_details(
            Some("DApp Provider"),
            Some("https://report.url"),
            Some("Report Provider"),
            Some("Provider Message"),
        );

    let success = advance_review.show(comm, &my_fields);

    match success {
        Ok(true) => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(comm, true);
        }
        Ok(false) => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(comm, false);
        }
        _ => {}
    }

    ledger_device_sdk::exit_app(0);
}
