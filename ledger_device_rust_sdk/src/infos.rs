/// Expose the API_LEVEL
#[link_section = ".ledger.api_level"]
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
