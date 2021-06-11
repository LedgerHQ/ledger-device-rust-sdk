#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(sdk_test_runner)]
#![feature(asm)]
#![feature(const_panic)]
#![cfg_attr(not(feature = "pre1_54"), feature(const_fn_trait_bound))]

pub mod bindings;
pub mod buttons;
pub mod ecc;
pub mod io;
pub mod nvm;
pub mod random;
pub mod seph;
pub mod usbbindings;

use bindings::os_sched_exit;

use core::{ffi::c_void, panic::PanicInfo};

/// In case of runtime problems, return an internal error and exit the app
#[inline]
#[cfg_attr(test, panic_handler)]
pub fn exiting_panic(_info: &PanicInfo) -> ! {
    let mut comm = io::Comm::new();
    comm.reply(io::StatusWords::Panic);
    exit_app(0);
}

/// Helper macro that sets an external panic handler
/// as the project's current panic handler
#[macro_export]
macro_rules! set_panic {
    ($f:expr) => {
        use core::panic::PanicInfo;
        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            $f(info)
        }
    };
}

/// Debug 'print' function that uses ARM semihosting
/// Prints only strings with no formatting
#[cfg(feature = "speculos")]
pub fn debug_print(s: &str) {
    let p = s.as_bytes().as_ptr();
    for i in 0..s.len() {
        let m = unsafe { p.offset(i as isize) };
        unsafe {
            asm!(
                "svc #0xab",
                in("r1") m,
                inout("r0") 3 => _,
            );
        }
    }
}

/// Custom type used to implement tests
#[cfg(feature = "speculos")]
pub struct TestType {
    pub modname: &'static str,
    pub name: &'static str,
    pub f: fn() -> Result<(), ()>,
}

/// Custom test runner that uses non-formatting print functions
/// using semihosting. Only reports 'Ok' or 'fail'.
#[cfg(feature = "speculos")]
pub fn sdk_test_runner(tests: &[&TestType]) {
    debug_print("--- Tests ---\n");
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
        match res {
            Ok(()) => debug_print("\x1b[1;32m   ok   \x1b[0m"),
            Err(()) => debug_print("\x1b[1;31m  fail  \x1b[0m"),
        }
        debug_print(modname);
        debug_print("::");
        debug_print(name);
        debug_print("\n");
    }
}

/// This variant of `assert_eq!()` returns an error
/// `Err(())` instead of panicking, to prevent tests
/// from exiting on first failure
#[cfg(feature = "speculos")]
#[macro_export]
macro_rules! assert_eq_err {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    $crate::debug_print("assertion failed: `(left == right)`\n");
                    return Err(());
                }
            }
        }
    }};
}

extern "C" {
    fn c_main();
}

#[link_section = ".boot"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Main is in C until the try_context can be set properly from Rust
    unsafe { c_main() };
    exit_app(1);
}

/// Wrapper for 'os_sched_exit'
/// Exit application with status
pub fn exit_app(status: u8) -> ! {
    unsafe { os_sched_exit(status) };
    unreachable!("Did not exit properly");
}

// The Rust version of Pic()
// hopefully there are ways to avoid that
extern "C" {
    fn pic(link_address: *mut c_void) -> *mut c_void;
}

/// Performs code address translation for reading data located in the program
/// and relocated during application installation.
pub fn pic_rs<T>(x: &T) -> &T {
    let ptr = unsafe { pic(x as *const T as *mut c_void) as *const T };
    unsafe { &*ptr }
}

/// Performs code address translation for reading mutable data located in the
/// program and relocated during application installation.
///
/// Warning: this is for corner cases as it is not directly possible to write
/// data stored in the code as it resides in Flash memory. This is needed in
/// particular when using the `nvm` module.
pub fn pic_rs_mut<T>(x: &mut T) -> &mut T {
    let ptr = unsafe { pic(x as *mut T as *mut c_void) as *mut T };
    unsafe { &mut *ptr }
}

/// Data wrapper to force access through address translation with [`pic_rs`] or
/// [`pic_rs_mut`]. This can help preventing mistakes when accessing data which
/// has been relocated.
///
/// # Examples
///
/// ```
/// // This constant data is stored in Code space, which is relocated.
/// static DATA: Pic<u32> = Pic::new(42);
/// ...
/// // Access with address translation is enforced thanks to Pic wrapper
/// let x: u32 = *DATA.get_ref();
/// ```
pub struct Pic<T> {
    data: T,
}

impl<T> Pic<T> {
    pub const fn new(data: T) -> Pic<T> {
        Pic { data }
    }

    /// Returns translated reference to the wrapped data.
    pub fn get_ref(&self) -> &T {
        pic_rs(&self.data)
    }

    /// Returns translated mutable reference to the wrapped data.
    pub fn get_mut(&mut self) -> &mut T {
        pic_rs_mut(&mut self.data)
    }
}

#[cfg(test)]
#[no_mangle]
fn sample_main() {
    test_main();
    exit_app(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use testmacro::test_item as test;

    #[test]
    fn test1() {
        assert_eq!(2, 2);
    }

    #[test]
    fn test2() {
        assert_eq!(3, 2);
    }
}
