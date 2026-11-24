//! Unaligned memory access helpers of the ARM run-time ABI for Nano X.
//!
//! Since LLVM 23 (from nightly-2026-09-15), the ARM backend lowers unaligned
//! 4- and 8-byte loads and stores on ARMv6-M to calls to `__aeabi_uread4`,
//! `__aeabi_uwrite4`, `__aeabi_uread8` and `__aeabi_uwrite8` when optimizing
//! for size (`opt-level = "z"`, used by apps). `compiler_builtins` does not
//! provide them for the `nanox` target: it either lacks them, or (from
//! nightly-2026-09-16) builds Thumb-2 versions that do not assemble for
//! ARMv6-M because it detects Thumb-1-only targets by their name
//! (rust-lang/compiler-builtins#1374).
//!
//! They are written in assembly so that the compiler cannot turn the byte
//! accesses back into a call to themselves, and defined weak so that a
//! working `compiler_builtins` definition takes precedence.

core::arch::global_asm!(
    ".syntax unified",
    ".thumb",
    // int __aeabi_uread4(void *address)
    ".section .text.__aeabi_uread4,\"ax\",%progbits",
    ".weak __aeabi_uread4",
    ".type __aeabi_uread4,%function",
    ".thumb_func",
    "__aeabi_uread4:",
    "ldrb r1, [r0, #0]",
    "ldrb r2, [r0, #1]",
    "lsls r2, r2, #8",
    "orrs r1, r2",
    "ldrb r2, [r0, #2]",
    "lsls r2, r2, #16",
    "orrs r1, r2",
    "ldrb r2, [r0, #3]",
    "lsls r2, r2, #24",
    "orrs r1, r2",
    "movs r0, r1",
    "bx lr",
    ".size __aeabi_uread4, . - __aeabi_uread4",
    // int __aeabi_uwrite4(int value, void *address), returns value
    ".section .text.__aeabi_uwrite4,\"ax\",%progbits",
    ".weak __aeabi_uwrite4",
    ".type __aeabi_uwrite4,%function",
    ".thumb_func",
    "__aeabi_uwrite4:",
    "strb r0, [r1, #0]",
    "lsrs r2, r0, #8",
    "strb r2, [r1, #1]",
    "lsrs r2, r0, #16",
    "strb r2, [r1, #2]",
    "lsrs r2, r0, #24",
    "strb r2, [r1, #3]",
    "bx lr",
    ".size __aeabi_uwrite4, . - __aeabi_uwrite4",
    // long long __aeabi_uread8(void *address): low word in r0, high in r1
    ".section .text.__aeabi_uread8,\"ax\",%progbits",
    ".weak __aeabi_uread8",
    ".type __aeabi_uread8,%function",
    ".thumb_func",
    "__aeabi_uread8:",
    "mov r3, r0",
    "ldrb r1, [r3, #4]",
    "ldrb r2, [r3, #5]",
    "lsls r2, r2, #8",
    "orrs r1, r2",
    "ldrb r2, [r3, #6]",
    "lsls r2, r2, #16",
    "orrs r1, r2",
    "ldrb r2, [r3, #7]",
    "lsls r2, r2, #24",
    "orrs r1, r2",
    "ldrb r0, [r3, #0]",
    "ldrb r2, [r3, #1]",
    "lsls r2, r2, #8",
    "orrs r0, r2",
    "ldrb r2, [r3, #2]",
    "lsls r2, r2, #16",
    "orrs r0, r2",
    "ldrb r2, [r3, #3]",
    "lsls r2, r2, #24",
    "orrs r0, r2",
    "bx lr",
    ".size __aeabi_uread8, . - __aeabi_uread8",
    // long long __aeabi_uwrite8(long long value, void *address), returns value
    ".section .text.__aeabi_uwrite8,\"ax\",%progbits",
    ".weak __aeabi_uwrite8",
    ".type __aeabi_uwrite8,%function",
    ".thumb_func",
    "__aeabi_uwrite8:",
    "strb r0, [r2, #0]",
    "lsrs r3, r0, #8",
    "strb r3, [r2, #1]",
    "lsrs r3, r0, #16",
    "strb r3, [r2, #2]",
    "lsrs r3, r0, #24",
    "strb r3, [r2, #3]",
    "strb r1, [r2, #4]",
    "lsrs r3, r1, #8",
    "strb r3, [r2, #5]",
    "lsrs r3, r1, #16",
    "strb r3, [r2, #6]",
    "lsrs r3, r1, #24",
    "strb r3, [r2, #7]",
    "bx lr",
    ".size __aeabi_uwrite8, . - __aeabi_uwrite8",
    ".text",
);

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    unsafe extern "C" {
        fn __aeabi_uread4(address: *const u8) -> u32;
        fn __aeabi_uwrite4(value: u32, address: *mut u8) -> u32;
        fn __aeabi_uread8(address: *const u8) -> u64;
        fn __aeabi_uwrite8(value: u64, address: *mut u8) -> u64;
    }

    #[test]
    fn unaligned_read_write() {
        let mut buf = [0u8; 12];
        // Odd offset: never 4-byte aligned.
        let p = unsafe { buf.as_mut_ptr().add(1) };
        unsafe {
            assert_eq!(__aeabi_uwrite4(0x1234_5678, p), 0x1234_5678);
            assert_eq!(buf[1..5], [0x78, 0x56, 0x34, 0x12]);
            assert_eq!(__aeabi_uread4(p), 0x1234_5678);
            let v = 0x0123_4567_89ab_cdef_u64;
            assert_eq!(__aeabi_uwrite8(v, p), v);
            assert_eq!(buf[1..9], v.to_le_bytes());
            assert_eq!(__aeabi_uread8(p), v);
        }
    }
}
