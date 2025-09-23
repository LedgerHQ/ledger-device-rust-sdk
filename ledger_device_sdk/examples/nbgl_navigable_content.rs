#![no_std]
#![no_main]

use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglNavigableContent};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    NbglNavigableContent::new()
        .title("Navigable Content")
        .init_page(0)
        .nb_pages(1)
        .show();

    ledger_secure_sdk_sys::exit_app(0);
}
