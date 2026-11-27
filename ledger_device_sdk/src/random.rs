//! Random number generation functions

use core::{ffi::c_void, fmt, ops::Range};

use ledger_secure_sdk_sys::{CX_OK, cx_err_t};
use num_traits::{Bounded, PrimInt, Unsigned};
use rand_core::{TryCryptoRng, TryRng};
use zeroize::Zeroize;

/// Re-export of the `rand_core` version used by [`LedgerRng`], so apps can
/// name its traits (and [`rand_core::UnwrapErr`]) without a version mismatch.
pub use rand_core;

/// Error returned when the device random number generator fails.
///
/// Wraps the `cx_err_t` code returned by the C SDK.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RngError(pub cx_err_t);

impl fmt::Display for RngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RNG failure: 0x{:x}", self.0)
    }
}

impl core::error::Error for RngError {}

/// Fills a byte array with random bytes, reporting RNG failures.
///
/// On failure `out` is zeroized and the C SDK error code is returned.
///
/// # Arguments
///
/// * `out` - Destination array.
#[inline]
pub fn try_rand_bytes(out: &mut [u8]) -> Result<(), RngError> {
    let res = unsafe {
        ledger_secure_sdk_sys::cx_get_random_bytes(out.as_mut_ptr() as *mut c_void, out.len())
    };
    if res != CX_OK {
        out.zeroize();
        return Err(RngError(res));
    }
    Ok(())
}

/// Fills a byte array with random bytes.
///
/// # Arguments
///
/// * `out` - Destination array.
///
/// # Panics
///
/// Panics if the random number generation fails. Use [`try_rand_bytes`] to
/// handle the error instead.
#[inline]
pub fn rand_bytes(out: &mut [u8]) {
    try_rand_bytes(out).expect("Failed to generate random bytes");
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

/// Device RNG implementing `rand_core@0.10` [`TryRng`] and [`TryCryptoRng`]
/// via the fallible [`try_rand_bytes`] syscall.
///
/// RNG failures are reported as [`RngError`] rather than hidden, so
/// `LedgerRng` does not implement the infallible [`rand_core::Rng`] /
/// [`rand_core::CryptoRng`] traits directly. Where those are required (e.g.
/// `rand::RngExt` or crates bounded on `CryptoRng`), wrap it in
/// [`rand_core::UnwrapErr`], which panics on failure:
///
/// ```ignore
/// use ledger_device_sdk::random::{LedgerRng, rand_core::{Rng, UnwrapErr}};
/// let x = UnwrapErr(LedgerRng).next_u64();
/// ```
#[derive(Copy, Clone, Debug)]
pub struct LedgerRng;

impl TryRng for LedgerRng {
    type Error = RngError;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, RngError> {
        let mut b = [0u8; 4];
        try_rand_bytes(&mut b)?;
        Ok(u32::from_be_bytes(b))
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, RngError> {
        let mut b = [0u8; 8];
        try_rand_bytes(&mut b)?;
        Ok(u64::from_be_bytes(b))
    }

    #[inline]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), RngError> {
        try_rand_bytes(dst)
    }
}

/// Mark LedgerRng as safe for cryptographic use
impl TryCryptoRng for LedgerRng {}

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

    #[test]
    fn try_rng() {
        let mut b = [0u8; 16];
        assert_eq!(LedgerRng.try_fill_bytes(&mut b).is_ok(), true);
        assert_eq!(u128::from_be_bytes(b) != 0, true);

        use rand_core::{Rng, UnwrapErr};
        let mut rng = UnwrapErr(LedgerRng);
        let x = [rng.next_u64(), rng.next_u64()];
        assert_eq!(x != [0, 0], true);
    }
}
