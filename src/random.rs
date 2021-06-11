//! Random number generation functions

// use crate::bindings::{cx_rng_u32, cx_rng_u8};
use core::ops::Range;
use num_traits::{Bounded, PrimInt, Unsigned};

extern "C" {
    pub fn cx_rng_no_throw(buffer: *mut u8, len: u32);
}

/// Fills a byte array with random bytes.
///
/// # Arguments
///
/// * `out` - Destination array.
pub fn rand_bytes(out: &mut [u8]) {
    unsafe {
        cx_rng_no_throw(out.as_mut_ptr(), out.len() as u32);
    }
}

/// In-house random trait for generating random numbers.
pub trait Random
where
    Self: PrimInt + Unsigned + Bounded,
{
    /// Generates a random value.
    fn random() -> Self;

    /// Generates and returns a random number in the given range
    ///
    /// # Arguments
    ///
    /// * `range` - range bounded inclusively below and exclusively above. Empty
    ///   ranges are not allowed and will cause panic.
    ///
    /// # Example
    ///
    /// ```
    /// // Roll a dice
    /// let r = random_from_range::<u8>(1..7);
    /// ```
    ///
    fn random_from_range(range: Range<Self>) -> Self {
        assert!(range.end > range.start, "Invalid range");
        let width = range.end - range.start;

        if width & (width - Self::one()) == Self::zero() {
            // Special case: range is a power of 2
            // Result is very fast to calculate.
            range.start + Self::random() % width
        } else {
            let chunk_size = Self::max_value() / width;
            let last_chunk_value = chunk_size * width;
            let mut r = Self::random();
            while r >= last_chunk_value {
                r = Self::random();
            }
            range.start + r / chunk_size
        }
    }
}

impl Random for u8 {
    fn random() -> Self {
        let mut r = [0u8; 1];
        unsafe {
            cx_rng_no_throw(r.as_mut_ptr(), 1);
        }
        r[0]
    }
}

impl Random for u32 {
    fn random() -> Self {
        let mut r = [0u8; 4];
        unsafe {
            cx_rng_no_throw(r.as_mut_ptr(), 4);
        }
        u32::from_be_bytes(r)
    }
}
