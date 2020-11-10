#![no_std]

pub mod bindings;
pub mod buttons;
pub mod syscalls_bindings;
pub mod ecc;
pub mod io;
pub mod seph;
pub mod random;
pub mod usbbindings;
pub mod usbcorebindings;

use syscalls_bindings::*;

use core::panic::PanicInfo;

/// In case of runtime problems, return an internal error and exit the app
#[inline]
pub fn exiting_panic(_info: &PanicInfo) -> ! {
    let mut comm = io::Comm::new();
    comm.set_status_word(io::StatusWords::Panic);
    comm.apdu_send();
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

pub fn pic_rs<T>(x: &T) -> &T {
    let ptr = unsafe { pic(x as *const T as u32) as *const T };
    unsafe{ & *ptr }
}