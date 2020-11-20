#![no_std]
#![feature(min_const_generics)]
#![feature(const_fn)]
#![feature(const_panic)]

pub mod bindings;
pub mod buttons;
pub mod syscalls_bindings;
pub mod ecc;
pub mod io;
pub mod seph;
pub mod random;
pub mod usbbindings;
pub mod usbcorebindings;
pub mod nvm;

use syscalls_bindings::*;

use core::panic::PanicInfo;

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
    unsafe { os_sched_exit( status) };
    loop {}
}

// The Rust version of PIC()
// hopefully there are ways to avoid that
extern "C" {
    fn pic(link_address: u32) -> u32; 
}

/// Performs code address translation for reading data located in the program
/// and relocated during application installation.
pub fn pic_rs<T>(x: &T) -> &T {
    let ptr = unsafe { pic(x as *const T as u32) as *const T };
    unsafe{ & *ptr }
}

/// Performs code address translation for reading mutable data located in the
/// program and relocated during application installation.
///
/// Warning: this is for corner cases as it is not directly possible to write
/// data stored in the code as it resides in Flash memory. This is needed in
/// particular when using the `nvm` module.
pub fn pic_rs_mut<T>(x: &mut T) -> &mut T {
    let ptr = unsafe { pic(x as *mut T as u32) as *mut T };
    unsafe{ &mut *ptr }
}

/// Data wrapper to force access through address translation with [`pic_rs`] or
/// [`pic_rs_mut`]. This can help preventing mistakes when accessing data which
/// has been relocated.
///
/// # Examples
///
/// ```
/// // This constant data is stored in Code space, which is relocated.
/// static DATA: PIC<u32> = PIC::new(42);
/// ...
/// // Access with address translation is enforced thanks to PIC wrapper
/// let x: u32 = *DATA.get_ref();
/// ```
pub struct PIC<T> {
    data: T
}

impl<T> PIC<T> {
    pub const fn new(data: T) -> PIC<T> {
        PIC { data }
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
