#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(sdk_test_runner)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod bindings;

#[cfg(target_os = "nanox")]
pub mod ble;

pub mod buttons;
pub mod ecc;
pub mod io;
pub mod nvm;
pub mod random;
pub mod screen;
pub mod seph;
pub mod usbbindings;

use bindings::os_sched_exit;

use core::{ffi::c_void, panic::PanicInfo};

/// In case of runtime problems, return an internal error and exit the app
#[inline]
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
use core::arch::asm;

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

#[cfg(feature = "speculos")]
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

#[cfg(test)]
#[cfg_attr(test, panic_handler)]
pub fn test_panic(info: &PanicInfo) -> ! {
    debug_print("Panic! ");
    let loc = info.location().unwrap();
    debug_print(loc.file());
    debug_print("\n");
    debug_print(core::str::from_utf8(&to_hex(loc.line())).unwrap());
    debug_print("\n");
    exit_app(0);
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

// Needed for `NVMData<T>` to function properly
extern "C" {
    // This is a linker script symbol defining the beginning of
    // the .nvm_data section. Declaring it as a static u32
    // (as is usually done) will result in a r9-indirect memory
    // access, as if it were a RAM access.
    // To force the compiler out of this assumption, we define
    // it as a function instead, but it is _not_ a function at all
    fn _nvram_data();
}

/// The following is a means to correctly access data stored in NVM
/// through the `#[link_section = ".nvm_data"]` attribute
pub struct NVMData<T> {
    data: T,
}

impl<T> NVMData<T> {
    pub const fn new(data: T) -> NVMData<T> {
        NVMData { data }
    }

    #[cfg(target_os = "nanos")]
    pub fn get_mut(&mut self) -> &mut T {
        crate::pic_rs_mut(&mut self.data)
    }

    /// This will return a mutable access by casting the pointer
    /// to the correct offset in `.nvm_data` manually.
    /// This is necessary when using the `rwpi` relocation model,
    /// because a static mutable will be assumed to be located in
    /// RAM, and be accessed through the static base (r9)
    #[cfg(not(target_os = "nanos"))]
    pub fn get_mut(&mut self) -> &mut T {
        use core::arch::asm;
        unsafe {
            // Compute offset in .nvm_data by taking the reference to
            // self.data and subtracting r9
            let addr = &self.data as *const T as u32;
            let static_base: u32;
            asm!( "mov {}, r9", out(reg) static_base);
            let offset = (addr - static_base) as isize;
            let data_addr = (_nvram_data as *const u8).offset(offset);
            let pic_addr = crate::bindings::pic(data_addr as *mut c_void) as *mut T;
            &mut *pic_addr.cast()
        }
    }
}

#[cfg(test)]
#[no_mangle]
fn sample_main() {
    test_main();
    exit_app(0);
}
