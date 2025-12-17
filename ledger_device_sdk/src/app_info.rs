use ledger_secure_sdk_sys::{const_cstr, infos::str_to_bytes};

const_cstr!(ELF_APP_NAME, "ledger.app_name", env!("APP_NAME"));
const_cstr!(ELF_APP_VERSION, "ledger.app_version", env!("APP_VERSION"));
const_cstr!(ELF_APP_FLAGS, "ledger.app_flags", env!("APP_FLAGS"));

#[used]
#[no_mangle]
#[link_section = ".install_parameters"]
#[allow(non_upper_case_globals)]
static install_parameters: [u8; include!(concat!(env!("OUT_DIR"), "/install_params_len.txt"))] =
    include!(concat!(env!("OUT_DIR"), "/install_params.txt"));
