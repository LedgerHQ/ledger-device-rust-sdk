#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, Field, NbglAddressReview, NbglGlyph, NbglReviewStatus, StatusType,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    let addr_hex = "0x1234567890ABCDEF1234567890ABCDEF12345678";

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    // Display the address confirmation screen.
    let success = NbglAddressReview::new()
        .glyph(&FERRIS)
        .review_title("Verify Address")
        .review_subtitle("Check carefully")
        .show(addr_hex);

    NbglReviewStatus::new()
        .status_type(StatusType::Address)
        .show(success);

    let my_fields = [
        Field {
            name: "Type address",
            value: "dummy type",
        },
        Field {
            name: "Sub address",
            value: "dummy sub address",
        },
    ];

    let success = NbglAddressReview::new()
        .glyph(&FERRIS)
        .review_title("Verify Address")
        .review_subtitle("Check carefully")
        .set_tag_value_list(&my_fields)
        .show(addr_hex);

    NbglReviewStatus::new()
        .status_type(StatusType::Address)
        .show(success);

    ledger_device_sdk::exit_app(0);
}
