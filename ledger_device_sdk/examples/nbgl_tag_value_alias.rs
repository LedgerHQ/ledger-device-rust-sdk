#![no_std]
#![no_main]

//! Tag/value pairs carrying value extensions (aliases).
//!
//! Each pair whose value is an alias gets a `>` affordance; tapping it opens a
//! modal built from the [`FieldExtension`]. This example walks through every
//! alias kind the C SDK supports.

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    Field, FieldExtension, NbglGlyph, NbglReview, NbglReviewStatus, TagValue, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
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

    // Nested content for the two list-shaped aliases. These must outlive the
    // review call, so they are bound here rather than inline.
    let token_details = [
        Field {
            name: "Contract",
            value: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        },
        Field {
            name: "Decimals",
            value: "6",
        },
    ];
    let raw_calldata = [
        Field {
            name: "Selector",
            value: "0xa9059cbb",
        },
        Field {
            name: "Argument 1",
            value: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        },
    ];

    let pairs = [
        // A plain pair: Field lifts into TagValue with no extension.
        TagValue::from(&Field {
            name: "Amount",
            value: "1.5 ETH",
        }),
        // ENS: NBGL adds its own "resolved by Ledger backend" note.
        TagValue {
            name: "To",
            value: "vitalik.eth",
            extension: Some(FieldExtension::ens(
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            )),
        },
        // Address book: alias_sub_name is the saved name.
        TagValue {
            name: "From",
            value: "My hardware wallet",
            extension: Some(
                FieldExtension::address_book("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")
                    .alias_sub_name("Saved 2026-01-14"),
            ),
        },
        // QR code: title captions the code in the modal.
        TagValue {
            name: "Refund address",
            value: "bc1qar0s...wf5mdq",
            extension: Some(
                FieldExtension::qr_code("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq")
                    .title("Scan to verify"),
            ),
        },
        // Info list: expands into [type, content] rows.
        TagValue {
            name: "Token",
            value: "USDC",
            extension: Some(FieldExtension::info_list(&token_details).back_text("Token details")),
        },
        // Tag/value list: expands into a nested review-style list.
        TagValue {
            name: "Calldata",
            value: "transfer(...)",
            extension: Some(FieldExtension::tag_value_list(&raw_calldata)),
        },
        // Plain full-value expansion for a value too long to fit.
        TagValue {
            name: "Memo",
            value: "Payment for invoice...",
            extension: Some(
                FieldExtension::full_value(
                    "Payment for invoice #2026-0042, settled under contract A-19.",
                )
                .explanation("Full memo as submitted"),
            ),
        },
    ];

    let success = NbglReview::new()
        .titles(
            "Please review transaction",
            "Value aliases",
            "Sign transaction\nto send ETH",
        )
        .glyph(&FERRIS)
        .show_ext(comm, &pairs);
    NbglReviewStatus::new().show(comm, success);

    ledger_device_sdk::exit_app(0);
}
