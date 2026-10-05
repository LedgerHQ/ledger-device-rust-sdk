#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    BarsList, CenterInfo, CenteredInfo, CenteredInfoStyle, ChoicesList, ExtendedCenter, Field,
    InfoButton, InfoLongPress, InfosList, NbglChoice, NbglGenericReview, NbglGlyph,
    NbglPageContent, NbglStatus, SwitchesList, TagValueConfirm, TagValueList, TuneIndex, init_comm,
};

use core::ops::Not;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

/// Runs when a switch, choice or bar in one of the contents is touched.
fn on_action(_token: u8, _index: u8) {}

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

    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    let centered_info = CenteredInfo::new(
        "Sample centered info",
        "Generic text",
        "More generic text",
        Some(&FERRIS),
        true,
        CenteredInfoStyle::LargeCaseBoldInfo,
        0,
    );

    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    let centered_info = CenteredInfo::new(
        "Centered info",
        "Generic text",
        Some(&FERRIS),
        true,
        CenteredInfoStyle::RegularInfo,
    );

    let info_button = InfoButton::new(
        "Validate info : abc",
        Some(&FERRIS),
        "Approve",
        TuneIndex::Success,
    );

    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    let info_long_press = InfoLongPress::new(
        "Validate to send token",
        Some(&FERRIS),
        "Hold to validate",
        TuneIndex::Success,
    );

    let my_example_fields = [
        Field {
            name: "Field 1",
            value: "0x1234567890abcdef",
        },
        Field {
            name: "Field 2",
            value: "0xdeafbeefdeadbeef",
        },
    ];

    let tag_values_list = TagValueList::new(&my_example_fields, 2, false, false);

    let tag_value_confirm = TagValueConfirm::new(
        &tag_values_list,
        TuneIndex::Success,
        "Confirm hash",
        "Reject hash",
    );

    let infos_list = InfosList::new(&my_example_fields);

    let mut review = NbglGenericReview::new()
        .on_action(on_action)
        .add_content(NbglPageContent::CenteredInfo(centered_info))
        .add_content(NbglPageContent::InfoButton(info_button));

    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    {
        review = review.add_content(NbglPageContent::InfoLongPress(info_long_press));
    }

    // The five content types added below complete coverage of the C union.
    let switches_list = SwitchesList::new(&[
        ("Expert mode", "Show raw values", false),
        ("Blind signing", "Allow unverified contracts", true),
    ]);
    let choices_list = ChoicesList::new(&["Slow", "Standard", "Fast"], 1);
    let bars_list = BarsList::new(&["Network details", "Fee breakdown"]);
    let extended_center = ExtendedCenter::new(
        &CenterInfo::new()
            .title("Extended center")
            .description("A centered block with a tip box under it."),
        Some("Tip box text"),
        Some(&FERRIS),
    );

    review = review
        .add_content(NbglPageContent::TagValueList(tag_values_list))
        .add_content(NbglPageContent::InfosList(infos_list))
        .add_content(NbglPageContent::SwitchesList(switches_list))
        .add_content(NbglPageContent::ChoicesList(choices_list))
        .add_content(NbglPageContent::BarsList(bars_list))
        .add_content(NbglPageContent::ExtendedCenter(extended_center))
        .add_content(NbglPageContent::TagValueConfirm(tag_value_confirm));

    #[cfg(target_os = "apex_p")]
    const IMPORTANT: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const IMPORTANT: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const IMPORTANT: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    let mut show_tx = true;
    let mut status_text = "Example rejected";
    while show_tx {
        let confirm = review.show(comm, "Reject");
        if confirm {
            status_text = "Example confirmed";
            show_tx = false;
        } else {
            show_tx = NbglChoice::new()
                .glyph(&IMPORTANT)
                .show(
                    comm,
                    "Reject transaction?",
                    "",
                    "Yes, reject",
                    "Go back to transaction",
                )
                // not() is used to invert the boolean value returned
                // by the choice (since we want to return to showing the
                // transaction if the user selects "Go back to transaction"
                // which returns false).
                .not();
        }
    }
    NbglStatus::new()
        .text(status_text)
        .show(comm, status_text == "Example confirmed");

    ledger_device_sdk::exit_app(0);
}
