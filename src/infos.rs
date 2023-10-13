const MAX_METADATA_STRING_SIZE: usize = 32;

const fn copy_str_to_u8_array(input: &str) -> [u8; MAX_METADATA_STRING_SIZE] {
    let mut result = [0u8; MAX_METADATA_STRING_SIZE];
    let bytes = input.as_bytes();
    let mut count = 0u32;

    // Ensure we don't copy more than MAX_METADATA_STRING_SIZE
    let min_length = core::cmp::min(bytes.len(), MAX_METADATA_STRING_SIZE - 1);
    // Copy bytes to array
    loop {
        if count >= min_length as u32 {
            result[count as usize] = '\n' as u8;
            break;
        }
        result[count as usize] = bytes[count as usize];
        count += 1;
    }
    result
}

/// Expose the API_LEVEL to the app
#[used]
static API_LEVEL: u8 = if cfg!(target_os = "nanos") {
    0
} else if cfg!(target_os = "nanox") {
    5
} else if cfg!(target_os = "nanosplus") {
    1
} else {
    0xff
};

#[used]
#[link_section = "ledger.api_level"]
static ELF_API_LEVEL: [u8; 4] = if cfg!(target_os = "nanos") {
    *b"0\n\0\0"
} else if cfg!(target_os = "nanox") {
    *b"5\n\0\0"
} else if cfg!(target_os = "nanosplus") {
    *b"1\n\0\0"
} else {
    *b"255\n"
};

#[used]
#[link_section = "ledger.target"]
static ELF_TARGET: [u8; 8] = if cfg!(target_os = "nanos") {
    *b"nanos\n\0\0"
} else if cfg!(target_os = "nanox") {
    *b"nanox\n\0\0"
} else if cfg!(target_os = "nanosplus") {
    *b"nanos2\n\0"
} else {
    *b"unknown\n"
};

#[used]
#[link_section = "ledger.target_id"]
static ELF_TARGET_ID: [u8; 11] = if cfg!(target_os = "nanos") {
    *b"0x31100004\n"
} else if cfg!(target_os = "nanox") {
    *b"0x33000004\n"
} else if cfg!(target_os = "nanosplus") {
    *b"0x33100004\n"
} else {
    *b"0xffffffff\n"
};

#[used]
#[link_section = "ledger.target_name"]
static ELF_TARGET_NAME: [u8; 14] = if cfg!(target_os = "nanos") {
    *b"TARGET_NANOS\n\0"
} else if cfg!(target_os = "nanox") {
    *b"TARGET_NANOX\n\0"
} else if cfg!(target_os = "nanosplus") {
    *b"TARGET_NANOS2\n"
} else {
    *b"WRONG_TARGET\n\0"
};

#[used]
#[link_section = "ledger.sdk_hash"]
static ELF_SDK_HASH: [u8; 5] = *b"None\n";

#[used]
#[link_section = "ledger.sdk_name"]
static ELF_SDK_NAME: [u8; 18] = *b"ledger-secure-sdk\n";

#[used]
#[link_section = "ledger.sdk_version"]
static ELF_SDK_VERSION: [u8; 5] = *b"None\n";

#[used]
#[link_section = "ledger.rust_sdk_name"]
static ELF_RUST_SDK_NAME: [u8; MAX_METADATA_STRING_SIZE] =
    copy_str_to_u8_array(env!("CARGO_PKG_NAME"));

#[used]
#[link_section = "ledger.rust_sdk_version"]
static ELF_RUST_SDK_VERSION: [u8; MAX_METADATA_STRING_SIZE] =
    copy_str_to_u8_array(env!("CARGO_PKG_VERSION"));
