#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglChoice, NbglGlyph, NbglStatus};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    
    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const WARNING: NbglGlyph =
        NbglGlyph::from_include(include_gif!("./icons/Warning_64px.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const WARNING: NbglGlyph =
        NbglGlyph::from_include(include_gif!("./icons/icon_warning.gif", NBGL));

    let back_to_safety = NbglChoice::new().glyph(&WARNING).show(
        "Security risk detected",
        "It may not be safe to sign this transaction. To continue, you'll need to review the risk.",
        "Back to safety",
        "Review risk",
    );

    if back_to_safety {
        NbglStatus::new().text("Transaction rejected").show(false);
    } else {
        let confirmed = NbglChoice::new()
            .show(
                "The transaction cannot be trusted",
                "Your Ledger cannot decode this transaction. If you sign it, you could be authorizing malicious actions that can drain your wallet.\n\nLearn more: ledger.com/e8",
                "I accept the risk",
                "Reject transaction"
            );

        NbglStatus::new()
            .text(if confirmed {
                "Transaction confirmed"
            } else {
                "Transaction rejected"
            })
            .show(confirmed);
    }
}
