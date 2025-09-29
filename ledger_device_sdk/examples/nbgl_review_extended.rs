#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReviewExtended, NbglReviewStatus, SyncNbgl};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let _comm = Comm::new();

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

    // Create NBGL review
    let review = NbglReviewExtended::new()
        .first_page(
            "Please review transaction",
            "Standard use case",
            "Reject transaction",
            &FERRIS,
        )
        .last_page(
            "Confirm transaction",
            "Press and hold to confirm",
            &FERRIS,
            false, // Light mode for the last page
        );

    let next = review.start();
    match next {
        SyncNbgl::UxSyncRetContinue => {
            let success = review.show(&my_fields);
            match success {
                SyncNbgl::UxSyncRetApproved => {
                    // The user approved the transaction
                    NbglReviewStatus::new().show(true);
                }
                _ => {
                    // The user rejected the transaction or an error occurred
                    NbglReviewStatus::new().show(false);
                }
            }
        }
        _ => {
            NbglReviewStatus::new().show(false); // The user rejected the transaction or an error occurred
        }
    }

    let review = review.last_page(
        "Confirm transaction",
        "Press to confirm",
        &FERRIS,
        true, // Light mode for the last page
    );
    let next = review.start();
    match next {
        SyncNbgl::UxSyncRetContinue => {
            let success = review.show(&my_fields);
            match success {
                SyncNbgl::UxSyncRetApproved => {
                    // The user approved the transaction
                    NbglReviewStatus::new().show(true);
                }
                _ => {
                    // The user rejected the transaction or an error occurred
                    NbglReviewStatus::new().show(false);
                }
            }
        }
        _ => {
            NbglReviewStatus::new().show(false); // The user rejected the transaction or an error occurred
        }
    }

    ledger_device_sdk::exit_app(0);
}
