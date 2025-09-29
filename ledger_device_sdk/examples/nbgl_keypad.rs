#![no_std]
#![no_main]

use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{NbglKeypad, NbglStatus, SyncNbgl};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let _comm = Comm::new();

    let res = NbglKeypad::new()
        .title("Enter PIN")
        .min_digits(4)
        .max_digits(8)
        .ask([0x31, 0x32, 0x33, 0x34].as_slice()); // Set PIN to "1234"
    if res == SyncNbgl::UxSyncRetPinValidated {
        NbglStatus::new().text("PIN OK").show(true);
    } else {
        NbglStatus::new().text("PIN KO").show(false);
    }

    let res = NbglKeypad::new()
        .title("Enter PIN")
        .min_digits(4)
        .max_digits(8)
        .hide(false)
        .ask([0x31, 0x32, 0x33, 0x34].as_slice()); // Set PIN to "1234"
    if res == SyncNbgl::UxSyncRetPinValidated {
        NbglStatus::new().text("PIN OK").show(true);
    } else {
        NbglStatus::new().text("PIN KO").show(false);
    }

    ledger_secure_sdk_sys::exit_app(0);
}
