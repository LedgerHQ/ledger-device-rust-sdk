#![no_std]
#![no_main]

//! A configuration screen built from generic content.
//!
//! The general form of `NbglGenericSettings`, which is fixed to one switch list
//! backed by NVM. There is nothing to approve: the flow ends when the user
//! leaves through the header.

use ledger_device_sdk::nbgl::{
    BarsList, ChoicesList, Field, InfosList, NbglGenericConfiguration, NbglPageContent, NbglStatus,
    SwitchesList, TagValueList, init_comm,
};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

/// Runs when a switch, choice or bar is touched.
///
/// `token` identifies the element (`FIRST_USER_TOKEN + i` for the `i`th, except
/// a choices list which uses one token and reports the pick in `index`). A real
/// app would record the new value here; the contents in this example are static.
fn on_action(_token: u8, _index: u8) {}

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

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

    let mut config = NbglGenericConfiguration::new()
        .title("Configuration")
        .on_action(on_action)
        .add_content(NbglPageContent::SwitchesList(SwitchesList::new(&[
            ("Expert mode", "Show raw values", false),
            ("Blind signing", "Allow unverified contracts", true),
        ])))
        .add_content(NbglPageContent::ChoicesList(ChoicesList::new(
            &["Slow", "Standard", "Fast"],
            1,
        )))
        .add_content(NbglPageContent::BarsList(BarsList::new(&[
            "Network details",
            "Fee breakdown",
        ])))
        .add_content(NbglPageContent::TagValueList(TagValueList::new(
            &fields, 2, false, false,
        )))
        .add_content(NbglPageContent::InfosList(InfosList::new(&fields)));

    // Returns once the user leaves through the header.
    config.show();

    NbglStatus::new()
        .text("Configuration closed")
        .show(comm, true);

    ledger_device_sdk::exit_app(0);
}
