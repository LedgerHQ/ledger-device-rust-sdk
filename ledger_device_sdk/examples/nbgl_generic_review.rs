#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{
    init_comm, CenteredInfo, CenteredInfoStyle, Field, InfoButton, InfoLongPress, InfosList,
    NbglGenericReview, NbglGlyph, NbglPageContent, TagValueConfirm, TagValueList, TuneIndex,
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

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));

    let centered_info = CenteredInfo::new(
        "Sample centered info",
        "Generic text",
        "More generic text",
        Some(&FERRIS),
        true,
        CenteredInfoStyle::LargeCaseBoldInfo,
        0,
    );

    let info_button = InfoButton::new(
        "Validate info : abc",
        Some(&FERRIS),
        "Approve",
        TuneIndex::Success,
    );

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

    let mut review: NbglGenericReview = NbglGenericReview::new()
        .add_content(NbglPageContent::CenteredInfo(centered_info))
        .add_content(NbglPageContent::InfoButton(info_button))
        .add_content(NbglPageContent::InfoLongPress(info_long_press))
        .add_content(NbglPageContent::TagValueList(tag_values_list))
        .add_content(NbglPageContent::TagValueConfirm(tag_value_confirm))
        .add_content(NbglPageContent::InfosList(infos_list));

    review.show("Reject Example", "Example Confirmed", "Example Rejected");
}
