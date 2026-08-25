#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    Field, NbglGlyph, NbglReview, NbglReviewStatus, TagValue, init_comm,
};

/// Runs when a value icon is touched, receiving the index of its pair.
///
/// The log macros are no-ops unless the app is built with the `debug` feature
/// and a `log_*` level, so this costs nothing in a release build.
fn on_value_icon(pair_index: u8) {
    ledger_device_sdk::log::info!("value icon touched on pair {}", pair_index);
}

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
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

    // A review whose pairs carry a touchable icon at the right of the value.
    // A list uses either icons or extensions throughout, never both.
    let icon_pairs = [
        TagValue {
            name: "Amount",
            value: "111 CRAB",
            value_icon: Some(&FERRIS),
            ..Default::default()
        },
        TagValue {
            name: "Destination",
            value: "0x1234567890ABCDEF1234567890ABCDEF12345678",
            value_icon: Some(&FERRIS),
            ..Default::default()
        },
    ];

    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "Value icons",
            "Sign transaction\nto send CRAB",
        )
        .glyph(&FERRIS)
        .on_value_icon(on_value_icon)
        .show_ext(comm, &icon_pairs);
    NbglReviewStatus::new().show(comm, success);

    ledger_device_sdk::exit_app(0);
}
