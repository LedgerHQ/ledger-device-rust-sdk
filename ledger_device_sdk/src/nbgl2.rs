use crate::io::{ApduHeader, Comm, Event, Reply};
use crate::nvm::*;
use const_zero::const_zero;
extern crate alloc;
use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{c_char, c_int};
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

pub mod nbgl2_home_and_settings;
pub use nbgl2_home_and_settings::*;
