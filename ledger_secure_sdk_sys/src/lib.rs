#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use core::mem::MaybeUninit;

pub mod buttons;
mod infos;
pub mod seph;

/// Wrapper for 'os_sched_exit'
/// Exit application with status
pub fn exit_app(status: u8) -> ! {
    unsafe { os_sched_exit(status) }
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

#[cfg(feature = "heap")]
use critical_section::RawRestoreState;
#[cfg(feature = "heap")]
use embedded_alloc::Heap;

#[cfg(feature = "heap")]
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[cfg(feature = "heap")]
struct CriticalSection;
#[cfg(feature = "heap")]
critical_section::set_impl!(CriticalSection);

/// Default empty implementation as we don't have concurrency.
#[cfg(feature = "heap")]
unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() -> RawRestoreState {}
    unsafe fn release(restore_state: RawRestoreState) {}
}

/// Initializes the heap memory for the global allocator.
///
/// The heap is stored in the stack, and has a fixed size.
/// This method is called just before [sample_main].
#[no_mangle]
#[cfg(feature = "heap")]
extern "C" fn heap_init() {
    const HEAP_SIZE: usize = 8192;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

#[no_mangle]
#[cfg(not(feature = "heap"))]
extern "C" fn heap_init() {}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
