#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    CenterInfo, NbglChoice, NbglGlyph, NbglStatus, WarningDetails, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

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
        comm,
        "Security risk detected",
        "It may not be safe to sign this transaction. To continue, you'll need to review the risk.",
        "Back to safety",
        "Review risk",
    );

    if back_to_safety {
        NbglStatus::new()
            .text("Transaction rejected")
            .show(comm, false);
    } else {
        let confirmed = NbglChoice::new()
            .ask_confirmation(Some("Are you sure to accept ?"), Some("Accept case"), Some("Yes"), Some("No"), true)
            .ask_confirmation(Some("Are you sure to reject ?"), Some("Reject case"), Some("Yes"), Some("No"), false)
            .glyph(&WARNING)
            .show(
                comm,
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
            .show(comm, confirmed);
    }

    // A choice carrying a details page, reachable before deciding.
    let details = WarningDetails::CenteredInfo {
        title: "About this risk",
        info: CenterInfo::new()
            .title("Unverified contract")
            .description("This contract is not in Ledger's registry.")
            .sub_text("Proceed only if you trust the source."),
    };

    let accepted = NbglChoice::new().glyph(&WARNING).show_with_details(
        comm,
        "Unverified contract",
        "Read the details before deciding.",
        "Accept",
        "Reject",
        &details,
    );
    NbglStatus::new()
        .text(if accepted { "Accepted" } else { "Rejected" })
        .show(comm, accepted);

    // The same, with a header icon and title above the message.
    let accepted = NbglChoice::new()
        .glyph(&WARNING)
        .show_advanced_with_details(
            comm,
            Some(&WARNING),
            "Review risk",
            "Unverified contract",
            "Read the details before deciding.",
            "Accept",
            "Reject",
            &details,
        );
    NbglStatus::new()
        .text(if accepted { "Accepted" } else { "Rejected" })
        .show(comm, accepted);

    ledger_device_sdk::exit_app(0);
}
