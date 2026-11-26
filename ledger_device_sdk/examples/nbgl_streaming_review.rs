#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    Field, NbglGlyph, NbglReviewStatus, NbglStreamingReview, NbglStreamingReviewStatus,
    TransactionType, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

    #[cfg(ledger_device = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(ledger_device = "stax", ledger_device = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(ledger_device = "nanosplus", ledger_device = "nanox"))]
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

    // Standard streaming review example
    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .tx_type(TransactionType::Transaction);
    if !review.start(comm, "Streaming example", Some("Standard")) {
        NbglReviewStatus::new().show(comm, false);
        ledger_secure_sdk_sys::exit_app(0);
    }
    let mut success: Option<bool> = None;
    for c in fields.chunks(1) {
        match review.next(comm, c) {
            NbglStreamingReviewStatus::Next => {}
            NbglStreamingReviewStatus::Skipped => {
                success = Some(review.finish(comm, "Sign to send token\n"));
                break;
            }
            NbglStreamingReviewStatus::Rejected => {
                success = Some(false);
                break;
            }
        }
    }
    match success {
        Some(b) => {
            NbglReviewStatus::new().show(comm, b);
        }
        None => {
            let success = review.finish(comm, "Sign to send token\n");
            NbglReviewStatus::new().show(comm, success);
        }
    }

    // Skippable streaming review example
    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .skippable()
        .tx_type(TransactionType::Transaction);
    if !review.start(comm, "Streaming example", Some("Skippable")) {
        NbglReviewStatus::new().show(comm, false);
        ledger_secure_sdk_sys::exit_app(0);
    }
    let mut success: Option<bool> = None;
    for c in fields.chunks(1) {
        match review.next(comm, c) {
            NbglStreamingReviewStatus::Next => {}
            NbglStreamingReviewStatus::Skipped => {
                success = Some(review.finish(comm, "Sign to send token\n"));
                break;
            }
            NbglStreamingReviewStatus::Rejected => {
                success = Some(false);
                break;
            }
        }
    }
    match success {
        Some(b) => {
            NbglReviewStatus::new().show(comm, b);
        }
        None => {
            let success = review.finish(comm, "Sign to send token\n");
            NbglReviewStatus::new().show(comm, success);
        }
    }

    // Blind signing example
    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .skippable()
        .blind()
        .tx_type(TransactionType::Transaction);
    if !review.start(comm, "Streaming example", Some("Blind Signing")) {
        NbglReviewStatus::new().show(comm, false);
        ledger_secure_sdk_sys::exit_app(0);
    }
    let mut success: Option<bool> = None;
    for c in fields.chunks(1) {
        match review.next(comm, c) {
            NbglStreamingReviewStatus::Next => {}
            NbglStreamingReviewStatus::Skipped => {
                success = Some(review.finish(comm, "Sign to send token\n"));
                break;
            }
            NbglStreamingReviewStatus::Rejected => {
                success = Some(false);
                break;
            }
        }
    }
    match success {
        Some(b) => {
            NbglReviewStatus::new().show(comm, b);
        }
        None => {
            let success = review.finish(comm, "Sign to send token\n");
            NbglReviewStatus::new().show(comm, success);
        }
    }

    // Blind signing + warning example
    let review: NbglStreamingReview = NbglStreamingReview::new()
        .glyph(&FERRIS)
        .skippable()
        .blind()
        .tx_type(TransactionType::Transaction)
        .warning_details(
            Some("DApp Provider"),
            Some("https://report.url"),
            Some("Report Provider"),
            Some("Provider Message"),
        );
    if !review.start(comm, "Streaming example", Some("Blind Signing")) {
        NbglReviewStatus::new().show(comm, false);
        ledger_secure_sdk_sys::exit_app(0);
    }
    let mut success: Option<bool> = None;
    for c in fields.chunks(1) {
        match review.next(comm, c) {
            NbglStreamingReviewStatus::Next => {}
            NbglStreamingReviewStatus::Skipped => {
                success = Some(review.finish(comm, "Sign to send token\n"));
                break;
            }
            NbglStreamingReviewStatus::Rejected => {
                success = Some(false);
                break;
            }
        }
    }
    match success {
        Some(b) => {
            NbglReviewStatus::new().show(comm, b);
        }
        None => {
            let success = review.finish(comm, "Sign to send token\n");
            NbglReviewStatus::new().show(comm, success);
        }
    }

    // Review started by a builder that is dropped right after `start`, then
    // continued and finished through another one: the last page shows the icon
    // given to `start`.
    if !start_with_temporary_builder(comm, &FERRIS) {
        NbglReviewStatus::new().show(comm, false);
        ledger_secure_sdk_sys::exit_app(0);
    }
    overwrite_stack();
    let review = NbglStreamingReview::new();
    let mut success: Option<bool> = None;
    for c in fields.chunks(1) {
        match review.next(comm, c) {
            NbglStreamingReviewStatus::Next => {}
            NbglStreamingReviewStatus::Skipped => {
                success = Some(review.finish(comm, "Sign to send token\n"));
                break;
            }
            NbglStreamingReviewStatus::Rejected => {
                success = Some(false);
                break;
            }
        }
    }
    match success {
        Some(b) => {
            NbglReviewStatus::new().show(comm, b);
        }
        None => {
            let success = review.finish(comm, "Sign to send token\n");
            NbglReviewStatus::new().show(comm, success);
        }
    }

    ledger_device_sdk::exit_app(0);
}

/// Starts a streaming review with a builder that is dropped on return.
#[inline(never)]
fn start_with_temporary_builder<const N: usize>(
    comm: &mut ledger_device_sdk::io::Comm<N>,
    glyph: &NbglGlyph<'static>,
) -> bool {
    NbglStreamingReview::new()
        .glyph(glyph)
        .tx_type(TransactionType::Transaction)
        .start(comm, "Streaming example", Some("Temporary builder"))
}

/// Fills the stack the dropped builder was on.
#[inline(never)]
fn overwrite_stack() {
    let filler = core::hint::black_box([0xa5u8; 512]);
    core::hint::black_box(&filler);
}
