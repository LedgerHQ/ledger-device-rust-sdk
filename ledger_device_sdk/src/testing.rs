use core::panic::PanicInfo;

#[cfg(feature = "debug")]
use core::arch::asm;

/// Debug 'print' function that uses ARM semihosting
/// Prints only strings with no formatting
#[cfg(feature = "debug")]
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

#[cfg_attr(test, panic_handler)]
pub fn test_panic(info: &PanicInfo) -> ! {
    debug_print("Panic! ");
    let loc = info.location().unwrap();
    debug_print(loc.file());
    debug_print("\n");
    debug_print(core::str::from_utf8(&to_hex(loc.line())).unwrap());
    debug_print("\n");
    ledger_secure_sdk_sys::exit_app(1);
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
    use core::ffi::c_void;
    use ledger_secure_sdk_sys::{pic, pic_rs};
    let mut failures = 0;
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
            Err(()) => {
                failures += 1;
                debug_print("\x1b[1;31m  fail  \x1b[0m")
            }
        }
        debug_print(modname);
        debug_print("::");
        debug_print(name);
        debug_print("\n");
    }
    if failures > 0 {
        ledger_secure_sdk_sys::exit_app(1);
    }
    ledger_secure_sdk_sys::exit_app(0);
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
                    $crate::testing::debug_print("assertion failed: `(left == right)`\n");
                    return Err(());
                }
            }
        }
    }};
}
