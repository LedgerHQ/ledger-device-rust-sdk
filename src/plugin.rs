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
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub operation: u8,
    pub name: [u8; 100],
}

pub struct PluginFeedParams {
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
}