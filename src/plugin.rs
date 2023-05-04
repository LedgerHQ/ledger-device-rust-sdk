pub enum PluginInteractionType {
    Check,
    Init,
    Feed,
    Finalize,
    ProvideData,
    QueryUI,
    GetUI,
    Unknown
}

impl From<u16> for PluginInteractionType {
    fn from(v: u16) -> Self {
        match v {
            0x0A00 => PluginInteractionType::Check,
            0x0A01 => PluginInteractionType::Init,
            0x0A02 => PluginInteractionType::Feed,
            0x0A03 => PluginInteractionType::Finalize,
            0x0A04 => PluginInteractionType::ProvideData,
            0x0A05 => PluginInteractionType::QueryUI,
            0x0A06 => PluginInteractionType::GetUI,
            _ => PluginInteractionType::Unknown
        }
    }
}

impl From<PluginInteractionType> for u16 {
    fn from(t: PluginInteractionType) -> Self {
        match t {
            PluginInteractionType::Check => 0x0A00,
            PluginInteractionType::Init => 0x0A01,
            PluginInteractionType::Feed => 0x0A02,
            PluginInteractionType::Finalize => 0x0A03,
            PluginInteractionType::ProvideData => 0x0A04,
            PluginInteractionType::QueryUI => 0x0A05,
            PluginInteractionType::GetUI => 0x0A06,
            PluginInteractionType::Unknown => 0x0AFF
        }
    }
}

pub struct PluginInitParams {
    pub app_data: *const u8,
    pub app_data_len: usize,
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub operation: u8,
    pub name: [u8; 100],
}

pub struct PluginFeedParams {
    pub app_data: *const u8,
    pub app_data_len: usize,
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
}

pub struct PluginFinalizeParams {
    pub app_data: *const u8,
    pub app_data_len: usize,
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub num_ui_screens: u8,
    pub need_info: bool
}

pub struct PluginQueryUiParams {
    pub title: [u8; 32],
    pub title_len: usize,
}

pub struct PluginGetUiParams {
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub ui_screen_idx: usize,
    pub title: [u8; 32],
    pub title_len: usize,
    pub msg: [u8; 64],
    pub msg_len: usize,
}


fn uint256_to_decimal(value: &[u8; 32], out: &mut [u8], out_len: &mut usize) -> bool {

    // Special case when value is 0
    if *value == [0u8; 32] {
        if out.len() < 2 {
            return false;
        }
        out[0] = b'0';
        *out_len = 1;
        return true;
    }

    let mut n: [u16; 16] = [0u16; 16];
    for idx in 0..16 {
        n[idx] = u16::from_be_bytes([value[2 * idx], value[2 * idx + 1]]);
    }

    let mut pos: usize = out.len();
    while n != [0u16; 16] {
        if pos == 0 {
            return false;
        }
        pos -= 1;
        let mut carry = 0u32;
        let mut rem: u32;
        for i in 0..16 {
            rem = ((carry << 16) | u32::from(n[i])) % 10;
            n[i] = (((carry << 16) | u32::from(n[i])) / 10) as u16;
            carry = rem;
        }
        out[pos] = u8::try_from(char::from_digit(carry, 10).unwrap()).unwrap(); 
    }
    out.copy_within(pos.., 0);
    *out_len = out.len() - pos;

    return true;
}

pub fn value_to_decimal_string(value: &[u8;32], decimals: usize, s: &mut [u8], s_len: &mut usize ) {
    
    let mut tmp: [u8; 100] = [0; 100];
    let mut len: usize = 0;
    uint256_to_decimal(value, &mut tmp[..], &mut len);

    s.fill(b'0');
    if len <= decimals {
        s[1] = b'.';
        s[2 + decimals - len..2 + decimals].copy_from_slice(&tmp[..len]);
        *s_len = 2 + decimals;
    }
    else {
        let delta = len - decimals;

        let part = &tmp[0..len];
        let (ipart, dpart) = part.split_at(delta);

        s[0..delta].copy_from_slice(ipart);
        s[delta] = b'.';
        s[delta + 1..delta + 1 + dpart.len()].copy_from_slice(dpart);
        *s_len = ipart.len() + dpart.len() + 1;
    }
}

