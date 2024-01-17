#![no_std]
#![no_main]

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    ledger_device_sdk::exit_app(1);
}

use ledger_device_sdk::ui::bitmaps::*;
use ledger_device_sdk::ui::gadgets::{Field, MultiFieldReview};

#[no_mangle]
extern "C" fn sample_main() {
    let fields = [
            Field {
                name: "Amount",
                value: "CRAB 666",
            },
            Field {
                name: "Destination",
                value: "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae",
            },
            Field {
                name: "Memo",
                value: "This is a very long memo. It will force the app client to send the serialized transaction to be sent in chunk. As the maximum chunk size is 255 bytes we will make this memo greater than 255 characters. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor. Cras elementum ultrices diam.",
            },
    ];
    let review = MultiFieldReview::new(
        &fields,
        &["Review ", "Transaction"],
        Some(&EYE),
        "Approve",
        Some(&VALIDATE_14),
        "Reject",
        Some(&CROSSMARK),
    );
    review.show();
    ledger_device_sdk::exit_app(0);
}
