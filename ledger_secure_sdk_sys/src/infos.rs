pub const fn str_to_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut result = [0u8; N];
    let mut i = 0;
    while i != s.len() {
        result[i] = s.as_bytes()[i];
        i += 1;
    }
    // Used to detect the end of the string when
    // reading the bytes from the ELF section
    result[i] = b'\n';
    result
}

#[macro_export]
macro_rules! const_cstr {
    ($name: ident, $section: literal, $in_str: expr) => {
        #[used]
        #[link_section = $section]
        static $name: [u8; $in_str.len() + 1] = str_to_bytes($in_str);
    };
}

// Store metadata in the ELF file
const_cstr!(ELF_API_LEVEL, "ledger.api_level", env!("API_LEVEL"));
const_cstr!(ELF_TARGET, "ledger.target", env!("TARGET"));
const_cstr!(ELF_TARGET_ID, "ledger.target_id", env!("TARGET_ID"));
const_cstr!(ELF_TARGET_NAME, "ledger.target_name", env!("TARGET_NAME"));
const_cstr!(
    ELF_RUST_SDK_NAME,
    "ledger.rust_sdk_name",
    env!("CARGO_PKG_NAME")
);
const_cstr!(
    ELF_RUST_SDK_VERSION,
    "ledger.rust_sdk_version",
    env!("CARGO_PKG_VERSION")
);
const_cstr!(ELF_C_SDK_NAME, "ledger.sdk_name", env!("C_SDK_NAME"));
const_cstr!(ELF_C_SDK_HASH, "ledger.sdk_hash", env!("C_SDK_HASH"));
const_cstr!(
    ELF_C_SDK_VERSION,
    "ledger.sdk_version",
    env!("C_SDK_VERSION")
);
const_cstr!(
    ELF_C_SDK_GRAPHICS,
    "ledger.sdk_graphics",
    env!("C_SDK_GRAPHICS")
);
