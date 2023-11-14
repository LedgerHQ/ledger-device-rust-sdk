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
#[link_section = ".ledger.api_level"]
#[used]
static API_LEVEL: u8 = const_parse_api_level(env!("API_LEVEL"));
