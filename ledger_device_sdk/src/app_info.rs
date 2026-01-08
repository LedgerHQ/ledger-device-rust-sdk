use ledger_secure_sdk_sys::{const_cstr, infos::str_to_bytes};

const_cstr!(
    ELF_APP_NAME,
    "ledger.app_name",
    option_env!("APP_NAME").unwrap_or("SDK Rust App")
);
const_cstr!(
    ELF_APP_VERSION,
    "ledger.app_version",
    option_env!("APP_VERSION").unwrap_or("0.0.0")
);
const_cstr!(
    ELF_APP_FLAGS,
    "ledger.app_flags",
    option_env!("APP_FLAGS").unwrap_or("0")
);

#[used]
#[no_mangle]
#[link_section = ".install_parameters"]
#[allow(non_upper_case_globals)]
static install_parameters: [u8; include!(concat!(env!("OUT_DIR"), "/install_params_len.txt"))] =
    include!(concat!(env!("OUT_DIR"), "/install_params.txt"));
