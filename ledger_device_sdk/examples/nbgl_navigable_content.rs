#![no_std]
#![no_main]

//! A flow of navigable pages with a touchable header.
//!
//! Unlike a review there is no confirm step: the flow ends when the user leaves
//! through the header. Each page carries one content type.

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{
    BarsList, CenteredInfo, CenteredInfoStyle, ChoicesList, Field, InfosList, NbglGlyph,
    NbglNavigableContent, NbglPageContent, NbglStatus, SwitchesList, TagValueList, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

/// Runs when a control on one of the pages is touched.
///
/// The pages here are static, so this only records that the touch arrived;
/// a real settings screen would update its state and redraw.
fn on_control(_token: u8, _index: u8) {}

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

    let fields = [
        Field {
            name: "Derivation path",
            value: "m/44'/1'/0'/0/0",
        },
        Field {
            name: "Chain ID",
            value: "1",
        },
    ];

    let mut content = NbglNavigableContent::new()
        .title("Settings")
        .on_control(on_control)
        // A plain centered page, with its own touchable title.
        .add_titled_page(
            NbglPageContent::CenteredInfo(CenteredInfo::new(
                "Navigable content",
                "Swipe to walk through the pages",
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                "",
                Some(&FERRIS),
                false,
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                CenteredInfoStyle::NormalInfo,
                #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
                CenteredInfoStyle::RegularInfo,
                #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
                0,
            )),
            "Overview",
            true,
            None,
        )
        .add_page(NbglPageContent::SwitchesList(SwitchesList::new(&[
            ("Expert mode", "Show raw values", false),
            ("Blind signing", "Allow unverified contracts", true),
        ])))
        .add_page(NbglPageContent::ChoicesList(ChoicesList::new(
            &["Mainnet", "Testnet", "Devnet"],
            0,
        )))
        .add_page(NbglPageContent::BarsList(BarsList::new(&[
            "Network details",
            "Fee breakdown",
        ])))
        .add_page(NbglPageContent::TagValueList(TagValueList::new(
            &fields, 2, false, false,
        )))
        .add_page(NbglPageContent::InfosList(InfosList::new(&fields)));

    // Returns once the user leaves through the header.
    content.show();

    NbglStatus::new().text("Settings closed").show(comm, true);

    ledger_device_sdk::exit_app(0);
}
