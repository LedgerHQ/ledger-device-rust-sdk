#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, NbglAddressReview, NbglGlyph, NbglReviewStatus, StatusType,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    let addr_hex = "0x1234567890ABCDEF1234567890ABCDEF12345678";

     // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("./examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("./examples/crab_16x16.gif", NBGL));
    
    // Display the address confirmation screen.
    let success = NbglAddressReview::new()
        .glyph(&FERRIS)
        .verify_str("Verify Address")
        .show(addr_hex);
    NbglReviewStatus::new()
        .status_type(StatusType::Address)
        .show(success);
}
