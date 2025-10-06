#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglAdvanceReview, NbglGlyph, NbglReviewStatus, StatusType, SyncNbgl,
    TransactionType,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

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

    let success = advance_review.show(&my_fields);

    match success {
        SyncNbgl::UxSyncRetApproved => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(true);
        }
        SyncNbgl::UxSyncRetRejected => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(false);
        }
        _ => {}
    }

    ledger_device_sdk::exit_app(0);
}
