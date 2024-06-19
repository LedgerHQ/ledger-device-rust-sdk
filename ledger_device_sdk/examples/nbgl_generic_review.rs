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

    let centered_info = CenteredInfo {
        text1: "Sample centered info",
        text2: "Generic text",
        text3: "More generic text",
        icon: Some(&FERRIS),
        on_top: true,
        style: CenteredInfoStyle::LargeCaseBoldInfo,
        offset_y: 0,
    };

    let info_button = InfoButton {
        text: "Validate info : abc",
        icon: Some(&FERRIS),
        button_text: "Approve",
        tune_id: TuneIndex::Success,
    };

    let info_long_press = InfoLongPress {
        text: "Validate to send token",
        icon: Some(&FERRIS),
        long_press_text: "Hold to validate",
        tune_id: TuneIndex::Success,
    };

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

    let tag_values_list = TagValueList {
        pairs: &my_example_fields,
        nb_max_lines_for_value: 2,
        small_case_for_value: false,
        wrapping: false,
    };

    let tag_values_list_2 = TagValueList {
        pairs: &my_example_fields,
        nb_max_lines_for_value: 2,
        small_case_for_value: false,
        wrapping: false,
    };

    let tag_value_confirm = TagValueConfirm {
        tag_value_list: tag_values_list_2,
        tune_id: TuneIndex::Success,
        confirmation_text: "Confirm hash",
        cancel_text: "Reject hash",
    };

    let infos_list = InfosList {
        infos: &my_example_fields,
    };

    let mut review: NbglGenericReview = NbglGenericReview::new()
        .add_content(NbglPageContent::CenteredInfo(centered_info))
        .add_content(NbglPageContent::InfoButton(info_button))
        .add_content(NbglPageContent::InfoLongPress(info_long_press))
        .add_content(NbglPageContent::TagValueList(tag_values_list))
        .add_content(NbglPageContent::TagValueConfirm(tag_value_confirm))
        .add_content(NbglPageContent::InfosList(infos_list));

    review.show("Reject Example", "Example Confirmed", "Example Rejected");
}
