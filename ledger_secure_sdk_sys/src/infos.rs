const fn make_c_string<const N: usize>(s: &str) -> [u8; N] {
    let mut result = [0u8; N];
    let mut i = 0;
    while i != s.len() {
        result[i] = s.as_bytes()[i];
        i += 1;
    }
    result[i] = '\n' as u8;
    result
}

macro_rules! const_cstr {
    ($name: ident, $section: literal, $in_str: expr) => {
        #[used]
        #[link_section = $section]
        static $name: [u8; $in_str.len() + 1] = make_c_string($in_str);
    };
}

const fn const_parse_api_level(x: &str) -> u8 {
    let a = x.as_bytes();
    let mut res = a[0] - b'0';
    let mut i = 1;
    while i < a.len() {
        res *= 10;
        res += a[i] - b'0';
        i += 1;
    }
    res
}

/// Expose the API_LEVEL
#[used]
static API_LEVEL: u8 = const_parse_api_level(env!("API_LEVEL"));

// Store metadata in the ELF file
const_cstr!(ELF_API_LEVEL, "ledger.api_level", env!("API_LEVEL_STR"));
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
