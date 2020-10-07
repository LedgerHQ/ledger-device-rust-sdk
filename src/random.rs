//! Random number generation functions

use crate::bindings::*;

/// Generates and returns a random byte value
pub fn rand_u8() -> u8 {
    unsafe { cx_rng_u8() }
}

/// Generates and returns a random unsigned 32-bits value
pub fn rand_u32() -> u32 {
    unsafe { cx_rng_u32() }
}

/// Generates and returns a random number in the given range
///
/// # Arguments
///
/// * `range` - range bounded inclusively below and exclusively above. Empty
///   ranges are not allowed and will cause panic.
///
/// # Examples
///
/// ```
/// // Roll a dice
/// let r = rand_u32_range(1..7)
/// ```
pub fn rand_u32_range(range: core::ops::Range<u32>) -> u32 {
    assert!(range.end > range.start, "Invalid range");
    let width = range.end - range.start;
    if width & (width - 1) == 0 {
        // Special case: range is a power of 2
        // Result is very fast to calculate.
        range.start + rand_u32() % width
    } else {
        let chunk_size = u32::MAX / width;
        let last_chunk_value = chunk_size * width;
        let mut r = rand_u32();
        while r >= last_chunk_value {
            r = rand_u32();
        }
        range.start + r / chunk_size
    }
}
