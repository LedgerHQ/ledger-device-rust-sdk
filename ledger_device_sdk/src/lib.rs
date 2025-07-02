#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(testing::sdk_test_runner)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(cfg_version)]

pub mod ecc;
pub mod hash;
pub mod hmac;
pub mod io;
pub mod libcall;
pub mod nvm;
pub mod random;
pub mod screen;
pub mod seph;

pub mod testing;

#[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
pub mod nbgl;
#[cfg(not(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl")))]
pub mod ui;

pub mod uxapp;

use core::panic::PanicInfo;

/// In case of runtime problems, return an internal error and exit the app
#[inline]
pub fn exiting_panic(_info: &PanicInfo) -> ! {
    let mut comm = io::Comm::new();
    comm.reply(io::StatusWords::Panic);
    ledger_secure_sdk_sys::exit_app(0);
}

// re-export exit_app
pub use ledger_secure_sdk_sys::buttons;
pub use ledger_secure_sdk_sys::exit_app;

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

extern "C" {
    fn c_main();
}

#[link_section = ".boot"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Main is in C until the try_context can be set properly from Rust
    unsafe { c_main() };
    ledger_secure_sdk_sys::exit_app(1);
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
        ledger_secure_sdk_sys::pic_rs(&self.data)
    }

    /// Returns translated mutable reference to the wrapped data.
    pub fn get_mut(&mut self) -> &mut T {
        ledger_secure_sdk_sys::pic_rs_mut(&mut self.data)
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
    fn _nvm_data_start();
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

    /// This will return a mutable access by casting the pointer
    /// to the correct offset in `.nvm_data` manually.
    /// This is necessary when using the `rwpi` relocation model,
    /// because a static mutable will be assumed to be located in
    /// RAM, and be accessed through the static base (r9)
    fn get_addr(&self) -> *mut T {
        use core::arch::asm;
        unsafe {
            // Compute offset in .nvm_data by taking the reference to
            // self.data and subtracting r9
            let addr = &self.data as *const T as u32;
            let static_base: u32;
            asm!( "mov {}, r9", out(reg) static_base);
            let offset = (addr - static_base) as isize;
            let data_addr = (_nvm_data_start as *const u8).offset(offset);
            ledger_secure_sdk_sys::pic(data_addr as *mut core::ffi::c_void) as *mut T
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            let pic_addr = self.get_addr();
            &mut *pic_addr.cast()
        }
    }

    pub fn get_ref(&self) -> &T {
        unsafe {
            let pic_addr = self.get_addr();
            &*pic_addr.cast()
        }
    }
}

#[cfg(test)]
#[no_mangle]
fn sample_main() {
    test_main();
}
