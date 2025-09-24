#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglChoice, NbglGlyph, NbglStatus};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    #[cfg(target_os = "apex_p")]
    const WARNING: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const WARNING: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const WARNING: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

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
            .ask_confirmation_when_accept(Some("Are you sure to accept ?"), Some("Accept case"), Some("Yes"), Some("No"))
            .ask_confirmation_when_reject(Some("Are you sure to reject ?"), Some("Reject case"), Some("Yes"), Some("No"))
            .glyph(&WARNING)
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

    ledger_secure_sdk_sys::exit_app(0);
}
