pub enum PluginInteractionType {
    Check,
    Init,
    Feed,
    Finalize,
    ProvideData,
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
            0x0A04 => PluginInteractionType::ProvideData,
            0x0A05 => PluginInteractionType::QueryUi,
            0x0A06 => PluginInteractionType::GetUi,
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
            PluginInteractionType::QueryUi => 0x0A05,
            PluginInteractionType::GetUi => 0x0A06,
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

pub struct PluginCoreParams {
    pub app_data: *const u8,
    pub app_data_len: usize,
    pub plugin_internal_ctx: *mut u8,
    pub plugin_internal_ctx_len: usize,
    pub plugin_result: PluginResult
}

pub struct PluginCheckParams {
    pub core_params: PluginCoreParams
}

pub struct PluginInitParams {
    pub core_params: PluginCoreParams
}

pub struct PluginFeedParams {
    pub core_params: PluginCoreParams
}

pub struct PluginFinalizeParams {
    pub core_params: PluginCoreParams,
    pub num_ui_screens: u8,
}

pub struct PluginProvideDataParams {
    pub core_params: PluginCoreParams
}

pub struct PluginQueryUiParams {
    pub core_params: PluginCoreParams,

    pub title: [u8; 32],
    pub title_len: usize,
}

pub struct PluginGetUiParams {
    pub core_params: PluginCoreParams,

    pub ui_screen_idx: usize,
    pub title: [u8; 32],
    pub title_len: usize,
    pub msg: [u8; 64],
    pub msg_len: usize,
}

pub enum PluginParams<'a> {
    Check(&'a mut PluginCheckParams),
    Init(&'a mut PluginInitParams),
    Feed(&'a mut PluginFeedParams),
    Finalize(&'a mut PluginFinalizeParams),
    ProvideData(&'a mut PluginProvideDataParams),
    QueryUi(&'a mut PluginQueryUiParams),
    GetUi(&'a mut PluginGetUiParams)
}

use crate::bindings::{
    os_lib_call
};

pub fn plugin_call(plugin_name: &str, plugin_params: PluginParams, op: PluginInteractionType) {
    
    let name: &[u8] = plugin_name.as_bytes();
    let mut arg: [u32; 3] = [0x00; 3];
    
    arg[0] = name.as_ptr() as u32;

    let operation: u16 = u16::from(op);
    arg[1] = operation as u32;

    match plugin_params {
        PluginParams::Check(p) => {
            arg[2] = p as *mut PluginCheckParams as u32;
        }
        PluginParams::Init(p) => {
            arg[2] = p as *mut PluginInitParams as u32;
        }
        PluginParams::Feed(p) => {
            arg[2] = p as *mut PluginFeedParams as u32;
        }
        PluginParams::Finalize(p) => {
            arg[2] = p as *mut PluginFinalizeParams as u32;
        }
        PluginParams::ProvideData(p) => {
            arg[2] = p as *mut PluginProvideDataParams as u32;
        }
        PluginParams::QueryUi(p) => {
            arg[2] = p as *mut PluginQueryUiParams as u32;
        }
        PluginParams::GetUi(p) => {
            arg[2] = p as *mut PluginGetUiParams as u32;
        }
    }
    unsafe {
        os_lib_call(arg.as_mut_ptr());
    }
}