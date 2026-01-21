//! Logging system for Ledger device applications.
//!
//! This module provides hierarchical logging macros for debugging and monitoring application
//! behavior on Ledger devices. Logging output uses ARM semihosting when the `debug` feature
//! is enabled, allowing messages to appear in the Speculos emulator console.
//!
//! # Feature Flags
//!
//! Logging is controlled by two types of feature flags that must both be enabled:
//!
//! ## Debug Output Feature
//!
//! - **`debug`**: Enables actual output via ARM semihosting. Without this feature, all logging
//!   macros compile to no-ops regardless of log level features.
//!
//! ## Log Level Features
//!
//! Log levels follow a hierarchical structure where enabling a higher verbosity level
//! automatically enables all lower levels:
//!
//! - **`log_error`**: Enable only error messages (highest priority, least verbose)
//! - **`log_warn`**: Enable warnings and errors (includes `log_error`)
//! - **`log_info`**: Enable informational messages, warnings, and errors (includes `log_warn`)
//! - **`log_debug`**: Enable debug messages and all above (includes `log_info`)
//! - **`log_trace`**: Enable trace messages and all above (includes `log_debug`, most verbose)
//!
//! # Usage
//!
//! ## Basic Usage
//!
//! ```rust
//! use ledger_device_sdk::log;
//!
//! // Available macros (if corresponding features are enabled):
//! log::error!("Critical error: {}", error_code);
//! log::warn!("Warning: retry attempt {}", attempt);
//! log::info!("Transaction processed successfully");
//! log::debug!("Internal state: {:?}", state);
//! log::trace!("Entering function with args: {}, {}", arg1, arg2);
//! ```
//!
//! ## Output Format
//!
//! Log messages are formatted as:
//! ```text
//! [LEVEL] file.rs:line: your message here
//! ```
//!
//! For example:
//! ```text
//! [INFO] src/main.rs:42: Application started
//! [ERROR] src/handler.rs:156: Invalid APDU: 0x6a82
//! ```
//!
//! ## Cargo.toml Configuration
//!
//! For development with Speculos emulator:
//! ```toml
//! [dependencies]
//! ledger_device_sdk = { version = "1.31", features = ["debug", "log_info"] }
//! ```
//!
//! For production builds, omit both `debug` and log level features:
//! ```toml
//! [dependencies]
//! ledger_device_sdk = { version = "1.31" }
//! ```
//!
//! ## Build Examples
//!
//! ```bash
//! # Development build with info-level logging
//! cargo build --features debug,log_info
//!
//! # Development build with trace-level logging (most verbose)
//! cargo build --features debug,log_trace
//!
//! # Production build (no logging)
//! cargo build --release
//! ```
//!
//! # Implementation Details
//!
//! - **Zero-cost abstractions**: When features are disabled, macros compile to empty blocks
//!   with no runtime overhead.
//! - **ARM semihosting**: Debug output uses SVC (supervisor call) instruction 0xAB for
//!   character-by-character printing.
//! - **No heap allocation**: All formatting is done on the stack.
//! - **Macro re-exports**: Macros are available both at crate root (`ledger_device_sdk::info!()`)
//!   and in this module (`ledger_device_sdk::log::info!()`).

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
    ($lvl:expr, $fmt:literal $($arg:tt)*) => ($crate::log!(target: core::module_path!(), $lvl, $fmt $($arg)*))
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $fmt:literal $($arg:tt)*) => ({ });
    ($lvl:expr, $fmt:literal $($arg:tt)*) => ($crate::log!(target: core::module_path!(), $lvl, $fmt $($arg)*))
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
pub use crate::{debug, error, info, log, trace, warn};
