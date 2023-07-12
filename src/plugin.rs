use crate::string::String;
pub enum PluginInteractionType {
    Check,
    Init,
    Feed,
    Finalize,
    QueryUi,
    GetUi,
    Unknown
}

impl From<u16> for PluginInteractionType {
    fn from(v: u16) -> Self {
        match v {
            0x0A00 => PluginInteractionType::Check,
            0x0A01 => PluginInteractionType::Init,
            0x0A02 => PluginInteractionType::Feed,
            0x0A03 => PluginInteractionType::Finalize,
            0x0A04 => PluginInteractionType::QueryUi,
            0x0A05 => PluginInteractionType::GetUi,
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
            PluginInteractionType::QueryUi => 0x0A04,
            PluginInteractionType::GetUi => 0x0A05,
            PluginInteractionType::Unknown => 0x0AFF
        }
    }
}

pub enum PluginResult {
    Ok,
    NeedInfo,
    Err
}

impl From<PluginResult> for u16 {
    fn from(res: PluginResult) -> Self {
        match res {
            PluginResult::Ok => 0x0000,
            PluginResult::NeedInfo => 0x0001,
            PluginResult::Err => 0xFF00,
        }
    }
}

pub struct PluginParam {
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub data_in: *const u8,
    pub data_out: *mut u8,
    pub result: PluginResult
}

use crate::bindings::os_lib_call;

pub fn plugin_call_v2(plugin_name: &str, plugin_params: &mut PluginParam, op: PluginInteractionType) {

    let name: &[u8] = plugin_name.as_bytes();
    let mut arg: [u32; 3] = [0x00; 3];
    
    arg[0] = name.as_ptr() as u32;

    let operation: u16 = u16::from(op);
    arg[1] = operation as u32;

    arg[2] = plugin_params as *mut PluginParam as u32;

    unsafe {
        os_lib_call(arg.as_mut_ptr());
    }
}