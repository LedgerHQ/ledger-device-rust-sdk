#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, NbglAddressReview, NbglGlyph, NbglReviewStatus, StatusType,
};
use ledger_secure_sdk_sys::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit_app(1);
}

#[no_mangle]
extern "C" fn sample_main() {
    unsafe {
        nbgl_refreshReset();
    }

    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    let addr_hex = "0x1234567890ABCDEF1234567890ABCDEF12345678";

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    // Display the address confirmation screen.
    let success = NbglAddressReview::new()
        .glyph(&FERRIS)
        .verify_str("Verify Address")
        .show(addr_hex);
    NbglReviewStatus::new()
        .status_type(StatusType::Address)
        .show(success);
    exit_app(0);
}
