//#![warn(missing_docs)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(testing::sdk_test_runner)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]

#[cfg(target_os = "nanox")]
mod aeabi_unaligned;
mod app_info;
#[cfg(feature = "app_storage")]
pub mod app_storage;
pub mod bn;
pub mod ecc;
pub mod hash;
pub mod hmac;
pub(crate) mod io_callbacks;
pub(crate) mod io_legacy;
#[cfg(feature = "io_new")]
pub(crate) mod io_new;

// Only re-export the selected module as `io`
pub mod io {
    #[cfg(not(feature = "io_new"))]
    pub use super::io_legacy::*;
    #[cfg(feature = "io_new")]
    pub use super::io_new::*;
}

pub mod libcall;
pub mod log;
pub mod math;
#[cfg(feature = "mldsa")]
pub mod mldsa;
#[cfg(feature = "mlkem")]
pub mod mlkem;
pub mod nvm;
pub mod pki;
pub mod random;
pub mod screen;
pub mod seph;
pub mod testing;
pub mod tlv;

#[cfg(any(
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p",
    feature = "nano_nbgl"
))]
pub mod nbgl;
#[cfg(all(
    any(target_os = "nanosplus", target_os = "nanox"),
    not(feature = "nano_nbgl")
))]
pub mod ui;

pub mod uxapp;

use core::panic::PanicInfo;

/// In case of runtime problems, return an internal error and exit the app
#[inline]
pub fn exiting_panic(_info: &PanicInfo) -> ! {
    #[cfg(not(feature = "io_new"))]
    {
        let mut comm = crate::io_legacy::Comm::new();
        comm.reply(crate::io_legacy::StatusWords::Panic);
    }
    #[cfg(feature = "io_new")]
    {
        crate::io_new::callbacks::send_panic_reply(crate::io_new::Reply::from(
            crate::io_new::StatusWords::Panic,
        ));
    }
    ledger_secure_sdk_sys::exit_app(0);
}

// Re-export exit_app
pub use ledger_secure_sdk_sys::buttons;
pub use ledger_secure_sdk_sys::exit_app;

// Re-export include_gif macro
pub use include_gif::include_gif;

// Re-export the underlying Ledger C SDK bindings when the `sys` feature is enabled.
// This provides direct (unsafe) FFI access. Prefer the safe abstractions in this crate
// when possible.
#[cfg(feature = "sys")]
pub use ledger_secure_sdk_sys as sys;

use ledger_secure_sdk_sys::{pic_rs, pic_rs_mut};

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

unsafe extern "C" {
    // `int c_main(int arg0)` in ledger_secure_sdk_sys/src/c/src.c
    fn c_main(arg0: core::ffi::c_int) -> core::ffi::c_int;
}

/// Application entry point, called by the OS with `arg0` in `r0`: 0 for a
/// normal start, or a pointer to the `libargs_t` of a library call (swap).
/// `c_main` uses it to decide whether to reset BSS and start I/O, and passes
/// it on to `sample_main`.
#[unsafe(link_section = ".boot")]
#[unsafe(no_mangle)]
pub extern "C" fn _start(arg0: core::ffi::c_int) -> ! {
    // Main is in C until the try_context can be set properly from Rust
    unsafe { c_main(arg0) };
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
        pic_rs(&self.data)
    }

    /// Returns translated mutable reference to the wrapped data.
    pub fn get_mut(&mut self) -> &mut T {
        pic_rs_mut(&mut self.data)
    }
}

// Needed for `NVMData<T>` to function properly
unsafe extern "C" {
    // This is a linker script symbol defining the beginning of
    // the .nvm_data section. Declaring it as a static u32
    // (as is usually done) will result in a r9-indirect memory
    // access, as if it were a RAM access.
    // To force the compiler out of this assumption, we define
    // it as a function instead, but it is _not_ a function at all
    fn _nvm_data_start();
    // End of the data held in .nvm_data: the relocation marker that follows
    // it (see link.ld) is not application data.
    fn _nvram_prev_run();
}

/// Size of the data area of `.nvm_data`, from `_nvm_data_start` to the
/// relocation marker.
fn nvm_data_len() -> usize {
    (_nvram_prev_run as *const u8 as usize).wrapping_sub(_nvm_data_start as *const u8 as usize)
}

/// Whether the `size` bytes at `ptr` lie in the data area of `.nvm_data`, at
/// the address the app runs it from: where [`NVMData::get_ref`] and
/// [`NVMData::get_mut`] point.
#[cfg(any(
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p",
    feature = "nano_nbgl",
    test
))]
pub(crate) fn in_nvm_data(ptr: *const u8, size: usize) -> bool {
    // SAFETY: only translates the address of the section start, as `get_addr`
    // does.
    let start = unsafe {
        ledger_secure_sdk_sys::pic(_nvm_data_start as *const u8 as *mut core::ffi::c_void)
    } as usize;
    (ptr as usize)
        .checked_sub(start)
        .and_then(|offset| offset.checked_add(size))
        .is_some_and(|end| end <= nvm_data_len())
}

/// The following is a means to correctly access data stored in NVM
/// through the `#[link_section = ".nvm_data"]` attribute
///
/// Declare it with [`nvm_data!`], which places it in that section.
pub struct NVMData<T> {
    data: T,
}

/// Declares statics in the `.nvm_data` section, each holding an [`NVMData`]
/// of the declared type.
///
/// ```ignore
/// use ledger_device_sdk::nvm::AtomicStorage;
///
/// ledger_device_sdk::nvm_data! {
///     static mut COUNTER: AtomicStorage<i32> = AtomicStorage::new(&3);
/// }
/// // declares
/// // static mut COUNTER: NVMData<AtomicStorage<i32>>
/// ```
///
/// The declared type must be [`NvmStorable`](crate::nvm::NvmStorable): one
/// of the storages of [`nvm`] holding an [`NvmValue`](crate::nvm::NvmValue),
/// or an `NvmValue` the app writes itself.
///
/// ```compile_fail
/// use ledger_device_sdk::nvm::AtomicStorage;
///
/// ledger_device_sdk::nvm_data! {
///     static mut FLAG: AtomicStorage<bool> = AtomicStorage::new(&false);
/// }
/// ```
#[macro_export]
macro_rules! nvm_data {
    ($($(#[$attr:meta])* $vis:vis static mut $name:ident: $ty:ty = $init:expr;)*) => {
        $(
            $(#[$attr])*
            #[unsafe(link_section = ".nvm_data")]
            $vis static mut $name: $crate::NVMData<$ty> = {
                $crate::nvm::require_nvm_storable::<$ty>();
                // Evaluated outside the `unsafe` block below, so that the initializer gets no
                // unsafe permission from the macro.
                let data: $ty = $init;
                // SAFETY: this is a static in `.nvm_data`.
                unsafe { $crate::NVMData::new_unchecked(data) }
            };
        )*
    };
}

impl<T> NVMData<T> {
    /// Wraps `data`.
    ///
    /// The accessors are only valid for a static in the `.nvm_data` section.
    #[deprecated(
        since = "1.44.0",
        note = "declare the static with `nvm_data!`, which places it in `.nvm_data`"
    )]
    pub const fn new(data: T) -> NVMData<T> {
        NVMData { data }
    }

    /// Wraps `data`, as [`nvm_data!`] does.
    ///
    /// # Safety
    /// The `NVMData` must be a static declared with
    /// `#[link_section = ".nvm_data"]`: its accessors translate its address
    /// into that section, and would otherwise refer to other data of it.
    pub const unsafe fn new_unchecked(data: T) -> NVMData<T> {
        NVMData { data }
    }

    /// Offset of the wrapped data in `.nvm_data`, or `None` if `self` is not
    /// at an offset where a value of type `T` fits in the section.
    ///
    /// A static placed in `.nvm_data` is addressed through the static base
    /// (r9) like a RAM static, so its offset from r9 is its offset in the
    /// section. An `NVMData` on the stack or past the section size is rejected;
    /// one in `.bss` or on the heap cannot be told apart from a static by its
    /// offset, and would alias data of the section.
    fn nvm_offset(&self) -> Option<usize> {
        let addr = &self.data as *const T as u32;
        let static_base: u32;
        unsafe { core::arch::asm!("mov {}, r9", out(reg) static_base) };
        let offset = usize::try_from(addr.wrapping_sub(static_base)).ok()?;
        let end = offset.checked_add(core::mem::size_of::<T>())?;
        (end <= nvm_data_len()).then_some(offset)
    }

    /// This will return a mutable access by casting the pointer
    /// to the correct offset in `.nvm_data` manually.
    /// This is necessary when using the `rwpi` relocation model,
    /// because a static mutable will be assumed to be located in
    /// RAM, and be accessed through the static base (r9)
    ///
    /// # Panics
    /// Panics if `self` is not at an offset of the `.nvm_data` section (see
    /// [`Self::nvm_offset`]).
    fn get_addr(&self) -> *mut T {
        let offset = self
            .nvm_offset()
            .expect("NVMData must be a static in the .nvm_data section");
        unsafe {
            let data_addr = (_nvm_data_start as *const u8).add(offset);
            ledger_secure_sdk_sys::pic(data_addr as *mut core::ffi::c_void) as *mut T
        }
    }

    /// Mutable access to the data in `.nvm_data`.
    ///
    /// # Panics
    /// Panics if `self` is not a static in the `.nvm_data` section.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe {
            let pic_addr = self.get_addr();
            &mut *pic_addr.cast()
        }
    }

    /// Shared access to the data in `.nvm_data`.
    ///
    /// # Panics
    /// Panics if `self` is not a static in the `.nvm_data` section.
    pub fn get_ref(&self) -> &T {
        unsafe {
            let pic_addr = self.get_addr();
            &*pic_addr.cast()
        }
    }
}

#[cfg(test)]
#[unsafe(no_mangle)]
fn sample_main() {
    test_main();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    crate::nvm_data! {
        static mut IN_NVM: [u32; 4] = [0; 4];
    }

    #[test]
    fn nvm_offset_accepts_statics_in_nvm_data() {
        let data = unsafe { &*&raw const IN_NVM };
        assert_eq!(data.nvm_offset().is_some(), true);
        assert_eq!(data.get_ref(), &[0u32; 4]);
    }

    #[test]
    fn in_nvm_data_accepts_data_of_nvm_data_statics_only() {
        let data = unsafe { &*&raw const IN_NVM }.get_ref();
        let ptr = data as *const [u32; 4] as *const u8;
        assert_eq!(in_nvm_data(ptr, core::mem::size_of::<[u32; 4]>()), true);
        // Past the end of the section.
        assert_eq!(in_nvm_data(ptr, usize::MAX), false);
        // On the stack.
        let local = [0u32; 4];
        assert_eq!(in_nvm_data(local.as_ptr() as *const u8, 16), false);
    }

    #[test]
    fn nvm_offset_rejects_values_on_the_stack() {
        // The stack is far past the start of RAM, so past the section size.
        #[allow(deprecated)]
        let local = NVMData::new([0u32; 4]);
        assert_eq!(local.nvm_offset(), None);
    }
}
