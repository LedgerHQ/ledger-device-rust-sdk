use core::panic::PanicInfo;

#[cfg(feature = "debug")]
use core::arch::asm;

/// Stack consumption measurement utility.
///
/// Implements the same paint/measure mechanism as the C SDK's
/// `get_stack_consumption` (behind `DEBUG_OS_STACK_CONSUMPTION`).
///
/// Usage:
/// 1. Call [`StackTracker::init`] early in your app (e.g. right after `Comm::new()`).
///    This paints the unused portion of the stack with a known pattern.
/// 2. After the operation you want to measure, call [`StackTracker::get_usage`]
///    to get the high-water mark in bytes.
///
/// # Safety
/// These functions use inline assembly to read the current stack pointer (`sp`)
/// and directly write to the stack memory region. They must only be called
/// from the app's main execution context (not from interrupts).
pub struct StackTracker;

const STACK_INIT_VALUE: u8 = 0xFF;

unsafe extern "C" {
    // Section-relative BSS symbols from the linker script.
    // Their addresses are properly relocated at load time via PIC.
    unsafe static _stack: u8;
    unsafe static _estack: u8;
}

impl StackTracker {
    /// Paint the unused portion of the application stack with a known pattern.
    ///
    /// This writes `0xFF` from `_stack` (bottom of stack region) up to the
    /// current stack pointer, preserving the currently active stack frames.
    /// The canary (just below `_stack`) is not touched.
    pub fn init() {
        let stack_lowest = &raw const _stack as *mut u8;
        let stack_current: *mut u8;
        unsafe {
            core::arch::asm!("mov {}, sp", out(reg) stack_current);
        }

        // Paint from the bottom of the stack region up to the current SP
        let mut ptr = stack_lowest;
        while ptr < stack_current {
            unsafe {
                core::ptr::write_volatile(ptr, STACK_INIT_VALUE);
                ptr = ptr.add(1);
            }
        }
    }

    /// Measure the stack high-water mark since the last [`init`](Self::init) call.
    ///
    /// Returns the number of bytes of stack that have been used (overwritten
    /// since initialization). Scans from `_stack` upward to find the first
    /// byte that is no longer `0xFF`.
    pub fn get_usage() -> usize {
        let stack_lowest = &raw const _stack as usize;
        let stack_top = &raw const _estack as usize;
        let stack_current: usize;
        unsafe {
            core::arch::asm!("mov {}, sp", out(reg) stack_current);
        }

        // Scan from the bottom up to find the first non-0xFF byte
        let mut ptr = stack_lowest as *const u8;
        let limit = stack_current as *const u8;
        while ptr < limit {
            if unsafe { core::ptr::read_volatile(ptr) } != STACK_INIT_VALUE {
                break;
            }
            ptr = unsafe { ptr.add(1) };
        }

        // Usage = from the high-water mark to the top of the stack
        stack_top - (ptr as usize)
    }
}

// C SDK protocol constants (from os_debug.h)
const MODE_INITIALIZATION: u8 = 0x00;
const MODE_RETRIEVAL: u8 = 0x01;
#[allow(dead_code)]
const SYSCALL_STACK_TYPE: u8 = 0x00;
const APP_STACK_TYPE: u8 = 0x01;

/// Handle the stack consumption BOLOS APDU (INS=0x57) for io_legacy.
///
/// Matches the C SDK's `get_stack_consumption` protocol:
/// - P1=0x00: Initialize (paint stack), response = 4 bytes big-endian 0
/// - P1=0x01: Retrieve usage, response = 4 bytes big-endian usage in bytes
/// - P2=0x01: App stack (the only type supported in Rust apps)
#[cfg(feature = "stack_usage")]
pub(crate) fn handle_stack_consumption_apdu(p1: u8, p2: u8, com: &mut crate::io_legacy::Comm) {
    use crate::io_legacy::StatusWords;

    if p1 > MODE_RETRIEVAL || p2 > APP_STACK_TYPE {
        com.reply(StatusWords::BadP1P2);
        return;
    }

    if p2 != APP_STACK_TYPE {
        // Syscall stack monitoring is not available in Rust apps
        com.reply(StatusWords::BadP1P2);
        return;
    }

    let status: u32 = match p1 {
        MODE_INITIALIZATION => {
            StackTracker::init();
            0u32
        }
        MODE_RETRIEVAL => u32::try_from(StackTracker::get_usage()).unwrap_or(u32::MAX),
        _ => {
            com.reply(StatusWords::BadP1P2);
            return;
        }
    };

    // Encode as 4-byte big-endian, matching C SDK's U4BE_ENCODE (unsigned 32-bit)
    let bytes = status.to_be_bytes();
    com.append(&bytes);
    com.reply_ok();
}

/// Handle the stack consumption BOLOS APDU (INS=0x57) for io_new.
#[cfg(all(feature = "stack_usage", feature = "io_new"))]
pub(crate) fn handle_stack_consumption_apdu_new<const N: usize>(
    p1: u8,
    p2: u8,
    comm: &mut crate::io_new::Comm<N>,
) {
    use crate::io_new::StatusWords;

    if p1 > MODE_RETRIEVAL || p2 > APP_STACK_TYPE {
        let _ = comm.begin_response().send(StatusWords::BadP1P2);
        return;
    }

    if p2 != APP_STACK_TYPE {
        let _ = comm.begin_response().send(StatusWords::BadP1P2);
        return;
    }

    let status: u32 = match p1 {
        MODE_INITIALIZATION => {
            StackTracker::init();
            0u32
        }
        MODE_RETRIEVAL => u32::try_from(StackTracker::get_usage()).unwrap_or(u32::MAX),
        _ => {
            let _ = comm.begin_response().send(StatusWords::BadP1P2);
            return;
        }
    };

    let bytes = status.to_be_bytes();
    let mut response = comm.begin_response();
    let _ = response.append(&bytes);
    let _ = response.send(StatusWords::Ok);
}

/// Debug 'print' function that uses ARM semihosting
/// Prints only strings with no formatting
#[cfg(feature = "debug")]
#[deprecated(note = "Use the logging macros from log module instead")]
pub fn debug_print(s: &str) {
    let p = s.as_bytes().as_ptr();
    for i in 0..s.len() {
        let m = unsafe { p.add(i) };
        unsafe {
            asm!(
                "svc #0xab",
                in("r1") m,
                inout("r0") 3 => _,
            );
        }
    }
}
#[cfg(not(feature = "debug"))]
#[deprecated(note = "Use the logging macros from log module instead")]
pub fn debug_print(_s: &str) {}

pub fn to_hex(m: u32) -> [u8; 8] {
    let mut hex = [0u8; 8];
    let mut i = 0;
    for c in m.to_be_bytes().iter() {
        let c0 = char::from_digit((c >> 4).into(), 16).unwrap();
        let c1 = char::from_digit((c & 0xf).into(), 16).unwrap();
        hex[i] = c0 as u8;
        hex[i + 1] = c1 as u8;
        i += 2;
    }
    hex
}

fn to_dec(v: u32) -> [u8; 10] {
    let mut dec = [0u8; 10];
    let mut val = v;
    let mut fact = 1_000_000_000;
    let mut i = 0;
    while fact != 0 {
        let d = val / fact;
        let c = char::from_digit(d.into(), 10).unwrap();
        dec[i] = c as u8;
        i += 1;
        val -= d * fact;
        fact /= 10;
    }
    dec
}

#[cfg_attr(test, panic_handler)]
pub fn test_panic(info: &PanicInfo) -> ! {
    let loc = info.location().unwrap();
    let bytes = to_dec(loc.line());
    let s = core::str::from_utf8(&bytes)
        .unwrap()
        .trim_start_matches('0');
    crate::log::error!(
        "Panic in {} at line {}: {}",
        loc.file(),
        s,
        info.message().as_str().unwrap()
    );
    ledger_secure_sdk_sys::exit_app(1);
}

/// Custom type used to implement tests
#[cfg(feature = "unit_test")]
pub struct TestType {
    pub modname: &'static str,
    pub name: &'static str,
    pub f: fn() -> Result<(), ()>,
}

/// Custom test runner that uses non-formatting print functions
/// using semihosting. Only reports 'Ok' or 'fail'.
#[cfg(feature = "unit_test")]
pub fn sdk_test_runner(tests: &[&TestType]) {
    use core::ffi::c_void;
    use ledger_secure_sdk_sys::{pic, pic_rs};
    let mut failures = 0;
    crate::log::info!("--- Tests ---");
    for test_ in tests {
        // (ノಠ益ಠ)ノ彡ꓛIꓒ
        let test = pic_rs(*test_);
        let modname;
        let name;
        unsafe {
            let t = pic(test.modname.as_ptr() as *mut c_void) as *const u8;
            let t = core::ptr::slice_from_raw_parts(t, test.modname.len());
            let t: &[u8] = core::mem::transmute(t);
            modname = core::str::from_utf8_unchecked(t);

            let t = pic(test.name.as_ptr() as *mut c_void) as *const u8;
            let t = core::ptr::slice_from_raw_parts(t, test.name.len());
            let t: &[u8] = core::mem::transmute(t);
            name = core::str::from_utf8_unchecked(t);
        }
        let fp = unsafe { pic(test.f as *mut c_void) };
        let fp: fn() -> Result<(), ()> = unsafe { core::mem::transmute(fp) };
        let res = fp();
        let res_out = match res {
            Ok(()) => "\x1b[1;32m   ok   \x1b[0m",
            Err(()) => {
                failures += 1;
                "\x1b[1;31m  fail  \x1b[0m"
            }
        };
        crate::log::info!("{} {}::{}", res_out, modname, name);
    }
    if failures > 0 {
        ledger_secure_sdk_sys::exit_app(1);
    }
    ledger_secure_sdk_sys::exit_app(0);
}

/// This variant of `assert_eq!()` returns an error
/// `Err(())` instead of panicking, to prevent tests
/// from exiting on first failure
#[cfg(feature = "unit_test")]
#[macro_export]
macro_rules! assert_eq_err {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    crate::log::error!("assertion failed: `(left == right)`");
                    return Err(());
                }
            }
        }
    }};
}
