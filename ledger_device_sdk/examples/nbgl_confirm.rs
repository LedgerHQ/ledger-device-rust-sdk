#![no_std]
#![no_main]

use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglConfirm};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    NbglConfirm::new().show(
        "Do you confirm this action?",
        Some("This action is irreversible."),
        "Confirm",
        "Cancel",
    );
    
    ledger_secure_sdk_sys::exit_app(0);
}

