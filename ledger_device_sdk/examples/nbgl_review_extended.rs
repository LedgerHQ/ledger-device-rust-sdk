#![no_std]
#![no_main]

//! `NbglReviewExtended` is touchscreen-only: the C use cases it wraps —
//! `nbgl_useCaseReviewStart`, `nbgl_useCaseStaticReview` and
//! `nbgl_useCaseStaticReviewLight` — are declared inside `#ifdef HAVE_SE_TOUCH`
//! and do not exist on Nano. `NbglReview` is the Nano equivalent.
//!
//! On Nano this example therefore builds as an empty app, so that
//! `cargo build --examples` succeeds on every target.

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
use include_gif::include_gif;
use ledger_device_sdk::nbgl::init_comm;
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReviewExtended, NbglReviewStatus};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let _comm = init_comm(&COMM);
    ledger_device_sdk::exit_app(0);
}

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));

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

    let next = review.start(comm);
    match next {
        Ok(true) => {
            let success = review.show(comm, &my_fields);
            match success {
                Ok(true) => {
                    // The user approved the transaction
                    NbglReviewStatus::new().show(comm, true);
                }
                _ => {
                    // The user rejected the transaction or an error occurred
                    NbglReviewStatus::new().show(comm, false);
                }
            }
        }
        _ => {
            NbglReviewStatus::new().show(comm, false); // The user rejected the transaction or an error occurred
        }
    }

    let review = review.last_page(
        "Confirm transaction",
        "Press to confirm",
        &FERRIS,
        true, // Light mode for the last page
    );
    let next = review.start(comm);
    match next {
        Ok(true) => {
            let success = review.show(comm, &my_fields);
            match success {
                Ok(true) => {
                    // The user approved the transaction
                    NbglReviewStatus::new().show(comm, true);
                }
                _ => {
                    // The user rejected the transaction or an error occurred
                    NbglReviewStatus::new().show(comm, false);
                }
            }
        }
        _ => {
            NbglReviewStatus::new().show(comm, false); // The user rejected the transaction or an error occurred
        }
    }

    ledger_device_sdk::exit_app(0);
}
