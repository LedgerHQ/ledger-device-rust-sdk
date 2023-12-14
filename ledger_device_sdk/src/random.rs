//! Random number generation functions

use core::ops::Range;

use num_traits::{Bounded, PrimInt, Unsigned};
use rand_core::{CryptoRng, RngCore};

/// Fills a byte array with random bytes.
///
/// # Arguments
///
/// * `out` - Destination array.
#[inline]
pub fn rand_bytes(out: &mut [u8]) {
    unsafe {
        ledger_secure_sdk_sys::cx_rng_no_throw(out.as_mut_ptr(), out.len());
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
        rand_bytes(&mut r);
        r[0]
    }
}

impl Random for u32 {
    fn random() -> Self {
        let mut r = [0u8; 4];
        rand_bytes(&mut r);
        u32::from_be_bytes(r)
    }
}

/// [`RngCore`] implementation via the [`rand_bytes`] syscall
#[derive(Copy, Clone, Debug)]
pub struct LedgerRng;

/// Implement [`RngCore`] (for `rand_core@0.6.x`) using ledger syscalls
///
/// For backwards compatibility with `rand_core@0.5.x` see [rand_compat](https://docs.rs/rand-compat/latest/rand_compat/)
impl RngCore for LedgerRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        rand_bytes(&mut b);
        u32::from_be_bytes(b)
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        rand_bytes(&mut b);
        u64::from_be_bytes(b)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_bytes(dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// Mark LedgerRng as safe for cryptographic use
impl CryptoRng for LedgerRng {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn rng() {
        // Test that the bindings are not broken by checking a random u128
        // isn't 0 (has 1/2^128 of happening)
        let r: [u8; 16] = core::array::from_fn(|_| u8::random());
        assert_eq!(u128::from_be_bytes(r) != 0, true);
    }
}
