#![no_std]
#![no_main]

//! Warning pages preceding an advanced review.
//!
//! Shows both ways to configure `NbglWarning`: the pre-defined set, where NBGL
//! builds the pages itself from `WarningType`s, and the manual path, where the
//! intro page and the details reachable from the top-right button are
//! described explicitly.

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    CenterInfo, Field, NbglAdvanceReview, NbglGlyph, NbglReviewStatus, NbglWarning, Prelude,
    StatusType, TransactionType, WarningBar, WarningDetails, WarningType, init_comm,
};
// QR code pages need NBGL_QRCODE, which only the touchscreen devices define.
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
use ledger_device_sdk::nbgl::QrCode;

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

    let my_fields = [
        Field {
            name: "Amount",
            value: "111 CRAB",
        },
        Field {
            name: "Destination",
            value: "0x1234567890ABCDEF1234567890ABCDEF12345678",
        },
    ];

    // --- 1. Pre-defined warnings -------------------------------------------
    // Two at once, which the old four-argument `warning_details` could not
    // express: it always raised W3cRiskDetected and nothing else.
    let predefined = NbglWarning::new()
        .predefined(&[WarningType::BlindSigning, WarningType::W3cRiskDetected])
        .dapp_provider("Example DApp")
        .report_provider("Example Scanner")
        .report_url("https://report.example/tx/0x1234")
        .provider_message("This contract was flagged as risky.");

    let success = NbglAdvanceReview::new(TransactionType::Transaction)
        .glyph(&FERRIS)
        .review_title("Review transaction")
        .review_subtitle("Pre-defined warnings")
        .finish_title("Sign transaction")
        .warning(&predefined)
        .show(comm, &my_fields);
    report(comm, success);

    // --- 2. Manual configuration -------------------------------------------
    // Sub-pages must outlive the review call, so they are bound here.
    let contract_info = WarningDetails::CenteredInfo {
        title: "Unverified contract",
        info: CenterInfo::new()
            .title("Not verified")
            .description("This contract is not in Ledger's registry.")
            .sub_text("Proceed only if you trust the source."),
    };
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    let report_details = WarningDetails::QrCode {
        title: "Full report",
        qr: QrCode::new("https://report.example/tx/0x1234")
            .text1("Scan for the full report")
            .text2("report.example"),
    };
    // Nano has no QR code support, so the same bar leads to plain text there.
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    let report_details = WarningDetails::CenteredInfo {
        title: "Full report",
        info: CenterInfo::new().description("report.example/tx/0x1234"),
    };

    let bars = [
        WarningBar {
            text: "Unverified contract",
            sub_text: Some("Not in Ledger's registry"),
            icon: None,
            details: Some(&contract_info),
        },
        WarningBar {
            text: "Read the full report",
            sub_text: Some("Opens the report"),
            icon: None,
            details: Some(&report_details),
        },
    ];
    let bar_list = WarningDetails::BarList {
        title: "Risks found",
        bars: &bars,
    };

    let intro = CenterInfo::new()
        .icon(&FERRIS)
        .title("2 risks found")
        .description("Review them before signing.");

    let prelude_details = WarningDetails::CenteredInfo {
        title: "Before you sign",
        info: CenterInfo::new().description("This transaction spends from a shared vault."),
    };
    let prelude = Prelude::new()
        .title("Shared vault")
        .description("This transaction spends from a shared vault.")
        .button_text("Learn more")
        .footer_text("Continue to review")
        .details(&prelude_details);

    // `intro_top_right_icon` is what makes `intro_details` reachable: on the
    // manual path NBGL only draws the top-right button when this icon is set,
    // and that button is the only way into the details. Without it the bar
    // list below is built but never shown.
    let manual = NbglWarning::new()
        .info(&intro)
        .intro_top_right_icon(&FERRIS)
        .intro_details(&bar_list)
        .review_details(&bar_list)
        .prelude(&prelude);

    let success = NbglAdvanceReview::new(TransactionType::Transaction)
        .glyph(&FERRIS)
        .review_title("Review transaction")
        .review_subtitle("Manual warning pages")
        .finish_title("Sign transaction")
        .warning(&manual)
        .show(comm, &my_fields);
    report(comm, success);

    ledger_device_sdk::exit_app(0);
}

fn report<const N: usize>(comm: &mut ledger_device_sdk::io::Comm<N>, success: Result<bool, u8>) {
    match success {
        Ok(approved) => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(comm, approved);
        }
        Err(_) => {
            NbglReviewStatus::new()
                .status_type(StatusType::Transaction)
                .show(comm, false);
        }
    }
}
