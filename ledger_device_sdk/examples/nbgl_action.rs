#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{NbglAction, NbglGlyph, init_comm};

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

    // Create NBGL action
    let action = NbglAction::new()
        .message("Press continue to proceed")
        .action_text("Continue")
        .glyph(&FERRIS);

    let _ = action.show(comm);

    ledger_device_sdk::exit_app(0);
}
