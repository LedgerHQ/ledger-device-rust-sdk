#![no_std]
#![no_main]

use ledger_device_sdk::nbgl::{NbglKeypad, NbglStatus, init_comm};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

    let res = NbglKeypad::new()
        .title("Enter PIN")
        .min_digits(4)
        .max_digits(8)
        .ask(comm, [0x31, 0x32, 0x33, 0x34].as_slice()); // Set PIN to "1234"
    if res {
        NbglStatus::new().text("PIN OK").show(comm, true);
    } else {
        NbglStatus::new().text("PIN KO").show(comm, false);
    }

    let res = NbglKeypad::new()
        .title("Enter PIN")
        .min_digits(4)
        .max_digits(8)
        .hide(false)
        .ask(comm, [0x31, 0x32, 0x33, 0x34].as_slice()); // Set PIN to "1234"
    if res {
        NbglStatus::new().text("PIN OK").show(comm, true);
    } else {
        NbglStatus::new().text("PIN KO").show(comm, false);
    }

    ledger_device_sdk::exit_app(0);
}
