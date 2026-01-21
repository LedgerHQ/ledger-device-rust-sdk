use core::fmt::Write;

#[cfg(feature = "debug")]
use core::arch::asm;

/// Debug 'print' function that uses ARM semihosting
/// Prints only strings with no formatting
#[cfg(feature = "debug")]
fn print(s: &str) {
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
pub fn print(_s: &str) {}

pub struct DBG;

#[cfg(feature = "debug")]
impl Write for DBG {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s);
        Ok(())
    }
}

#[cfg(not(feature = "debug"))]
impl Write for DBG {
    fn write_str(&mut self, _s: &str) -> core::fmt::Result {
        Ok(()) // No-op for production builds
    }
}

#[cfg(feature = "debug")]
#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $fmt:literal $($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = core::write!($crate::log::DBG, concat!("[{}] {}:{}: ", $fmt, "\r\n"), $lvl, core::file!(), core::line!() $($arg)*);
    });
    ($lvl:expr, $fmt:literal $($arg:tt)*) => ($crate::log!(target: __log_module_path!(), $lvl, $fmt $($arg)*))
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $fmt:literal $($arg:tt)*) => ({ });
    ($lvl:expr, $fmt:literal $($arg:tt)*) => ($crate::log!(target: __log_module_path!(), $lvl, $fmt $($arg)*))
}

#[cfg(feature = "log_error")]
#[macro_export]
macro_rules! error {
    ($fmt:literal $($arg:tt)*) => ({$crate::log!("ERROR", $fmt $($arg)*)})
}
#[cfg(not(feature = "log_error"))]
#[macro_export]
macro_rules! error {
    ($fmt:literal $($arg:tt)*) => {{}};
}

#[cfg(feature = "log_warn")]
#[macro_export]
macro_rules! warn {
    ($fmt:literal $($arg:tt)*) => ({$crate::log!("WARN", $fmt $($arg)*)})
}
#[cfg(not(feature = "log_warn"))]
#[macro_export]
macro_rules! warn {
    ($fmt:literal $($arg:tt)*) => {{}};
}

#[cfg(feature = "log_info")]
#[macro_export]
macro_rules! info {
    ($fmt:literal $($arg:tt)*) => ({$crate::log!("INFO", $fmt $($arg)*)})
}
#[cfg(not(feature = "log_info"))]
#[macro_export]
macro_rules! info {
    ($fmt:literal $($arg:tt)*) => {{}};
}

#[cfg(feature = "log_debug")]
#[macro_export]
macro_rules! debug {
    ($fmt:literal $($arg:tt)*) => ({$crate::log!("DEBUG", $fmt $($arg)*)})
}
#[cfg(not(feature = "log_debug"))]
#[macro_export]
macro_rules! debug {
    ($fmt:literal $($arg:tt)*) => {{}};
}

#[cfg(feature = "log_trace")]
#[macro_export]
macro_rules! trace {
    ($fmt:literal $($arg:tt)*) => ({$crate::log!("TRACE", $fmt $($arg)*)})
}
#[cfg(not(feature = "log_trace"))]
#[macro_export]
macro_rules! trace {
    ($fmt:literal $($arg:tt)*) => {{}};
}

// Re-export macros in the log module so they can be accessed as ledger_device_sdk::log::info!()
pub use crate::{log, error, warn, info, debug, trace};