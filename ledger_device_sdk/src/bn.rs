//! Safe Rust wrappers for the Ledger C SDK big-number (BN) and Montgomery
//! arithmetic functions declared in `ox_bn.h`.
//!
//! The BN subsystem operates inside a **locked context**: call
//! [`BnLock::acquire`] before any BN operation and let the guard drop (or call
//! [`BnLock::release`]) when done.  All [`Bn`] values allocated inside a lock
//! are automatically destroyed when they are dropped.
//!
//! # Example
//!
//! ```ignore
//! use ledger_device_sdk::bn::{BnLock, Bn};
//!
//! let _lock = BnLock::acquire(32)?;
//! let mut a = Bn::alloc(32)?;
//! a.set_u32(42)?;
//! let mut b = Bn::alloc_init(&[0, 0, 0, 7])?;
//! let mut r = Bn::alloc(32)?;
//! r.add(&a, &b)?;
//! ```

use crate::check_cx_ok;
use crate::ecc::CxError;
use core::cmp::Ordering;
use core::ffi::c_int;
use ledger_secure_sdk_sys::*;

// =========================================================================
// BnLock – RAII guard for the big-number context
// =========================================================================

/// RAII guard for the BN lock context.
///
/// All BN / Montgomery operations require the context to be locked first.
/// The lock is automatically released when this guard is dropped.
///
/// # Example
///
/// ```ignore
/// let _lock = BnLock::acquire(32)?;
/// // … BN operations …
/// // lock released on drop
/// ```
pub struct BnLock;

impl BnLock {
    /// Lock the BN context.
    ///
    /// `word_nbytes` is the maximal byte-size of BN values that will be
    /// allocated inside this lock (e.g. 32 for 256-bit numbers).
    /// Wraps `cx_bn_lock`.
    pub fn acquire(word_nbytes: usize) -> Result<Self, CxError> {
        check_cx_ok!(cx_bn_lock(word_nbytes, 0));
        Ok(BnLock)
    }

    /// Explicitly release the lock (equivalent to dropping the guard).
    /// Wraps `cx_bn_unlock`.
    pub fn release(self) {
        drop(self);
    }

    /// Returns `true` if the BN context is currently locked.
    /// Wraps `cx_bn_locked`.
    pub fn is_locked() -> bool {
        unsafe { cx_bn_locked() == CX_OK }
    }

    /// Returns `true` if the BN context is currently locked (alternate API).
    /// Wraps `cx_bn_is_locked`.
    pub fn is_locked_bool() -> bool {
        unsafe { cx_bn_is_locked() }
    }
}

impl Drop for BnLock {
    fn drop(&mut self) {
        unsafe {
            cx_bn_unlock();
        }
    }
}

// =========================================================================
// Bn – RAII wrapper around cx_bn_t
// =========================================================================

/// Safe RAII wrapper around a single big-number handle (`cx_bn_t`).
///
/// The BN is allocated inside the locked BN context and automatically
/// destroyed when dropped.  A [`BnLock`] **must** be held for the entire
/// lifetime of every `Bn`.
#[derive(Debug)]
pub struct Bn {
    handle: cx_bn_t,
}

impl Bn {
    // ----- allocation / init ------------------------------------------------

    /// Allocate an uninitialised BN with room for `nbytes` bytes.
    /// Wraps `cx_bn_alloc`.
    pub fn alloc(nbytes: usize) -> Result<Self, CxError> {
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        check_cx_ok!(cx_bn_alloc(&mut handle, nbytes));
        Ok(Self { handle })
    }

    /// Allocate a BN and initialise it from a big-endian byte slice.
    /// The BN will have room for `nbytes` bytes where `nbytes` is at least
    /// `value.len()` rounded up to the BN word alignment.
    /// Wraps `cx_bn_alloc_init`.
    pub fn alloc_init(value: &[u8]) -> Result<Self, CxError> {
        let nbytes = align_bn_size(value.len());
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        check_cx_ok!(cx_bn_alloc_init(
            &mut handle,
            nbytes,
            value.as_ptr(),
            value.len(),
        ));
        Ok(Self { handle })
    }

    /// Allocate a BN with `nbytes` capacity and initialise from a byte slice.
    /// Wraps `cx_bn_alloc_init`.
    pub fn alloc_init_size(nbytes: usize, value: &[u8]) -> Result<Self, CxError> {
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        check_cx_ok!(cx_bn_alloc_init(
            &mut handle,
            nbytes,
            value.as_ptr(),
            value.len(),
        ));
        Ok(Self { handle })
    }

    /// Return the raw `cx_bn_t` handle for use with low-level FFI.
    pub fn raw(&self) -> cx_bn_t {
        self.handle
    }

    /// Return the raw `cx_bn_t` handle mutably for use with low-level FFI.
    pub fn raw_mut(&mut self) -> &mut cx_bn_t {
        &mut self.handle
    }

    // ----- init / copy / size -----------------------------------------------

    /// (Re-)initialise this BN from a big-endian byte slice.
    /// Wraps `cx_bn_init`.
    pub fn init(&self, value: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_init(self.handle, value.as_ptr(), value.len()));
        Ok(())
    }

    /// Fill this BN with random data.
    /// Wraps `cx_bn_rand`.
    pub fn rand(&self) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_rand(self.handle));
        Ok(())
    }

    /// Copy the value of `other` into `self`.
    /// Wraps `cx_bn_copy`.
    pub fn copy_from(&self, other: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_copy(self.handle, other.handle));
        Ok(())
    }

    /// Return the size in bytes of this BN.
    /// Wraps `cx_bn_nbytes`.
    pub fn nbytes(&self) -> Result<usize, CxError> {
        let mut n = 0usize;
        check_cx_ok!(cx_bn_nbytes(self.handle, &mut n));
        Ok(n)
    }

    // ----- u32 get/set & export ---------------------------------------------

    /// Set this BN to the given `u32` value.
    /// Wraps `cx_bn_set_u32`.
    pub fn set_u32(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_set_u32(self.handle, n));
        Ok(())
    }

    /// Return the value of this BN as a `u32`.
    /// The BN must fit in 32 bits.
    /// Wraps `cx_bn_get_u32`.
    pub fn get_u32(&self) -> Result<u32, CxError> {
        let mut n = 0u32;
        check_cx_ok!(cx_bn_get_u32(self.handle, &mut n));
        Ok(n)
    }

    /// Export this BN into a big-endian byte buffer.
    /// Wraps `cx_bn_export`.
    pub fn export(&self, bytes: &mut [u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_export(self.handle, bytes.as_mut_ptr(), bytes.len()));
        Ok(())
    }

    // ----- comparison -------------------------------------------------------

    /// Compare two BNs.
    /// Returns `Ordering::Less`, `Equal`, or `Greater`.
    /// Wraps `cx_bn_cmp`.
    pub fn cmp_bn(&self, other: &Bn) -> Result<Ordering, CxError> {
        let mut diff: c_int = 0;
        check_cx_ok!(cx_bn_cmp(self.handle, other.handle, &mut diff));
        Ok(int_to_ordering(diff))
    }

    /// Compare this BN with a `u32`.
    /// Returns `Ordering::Less`, `Equal`, or `Greater`.
    /// Wraps `cx_bn_cmp_u32`.
    pub fn cmp_u32(&self, other: u32) -> Result<Ordering, CxError> {
        let mut diff: c_int = 0;
        check_cx_ok!(cx_bn_cmp_u32(self.handle, other, &mut diff));
        Ok(int_to_ordering(diff))
    }

    /// Returns `true` if this BN is odd.
    /// Wraps `cx_bn_is_odd`.
    pub fn is_odd(&self) -> Result<bool, CxError> {
        let mut odd = false;
        check_cx_ok!(cx_bn_is_odd(self.handle, &mut odd));
        Ok(odd)
    }

    // ----- bitwise logic ----------------------------------------------------

    /// `self = a XOR b`.
    /// Wraps `cx_bn_xor`.
    pub fn xor(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_xor(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a OR b`.
    /// Wraps `cx_bn_or`.
    pub fn or(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_or(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a AND b`.
    /// Wraps `cx_bn_and`.
    pub fn and(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_and(self.handle, a.handle, b.handle));
        Ok(())
    }

    // ----- bit manipulation -------------------------------------------------

    /// Test whether bit at position `pos` is set.
    /// Wraps `cx_bn_tst_bit`.
    pub fn tst_bit(&self, pos: u32) -> Result<bool, CxError> {
        let mut set = false;
        check_cx_ok!(cx_bn_tst_bit(self.handle, pos, &mut set));
        Ok(set)
    }

    /// Set bit at position `pos`.
    /// Wraps `cx_bn_set_bit`.
    pub fn set_bit(&self, pos: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_set_bit(self.handle, pos));
        Ok(())
    }

    /// Clear bit at position `pos`.
    /// Wraps `cx_bn_clr_bit`.
    pub fn clr_bit(&self, pos: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_clr_bit(self.handle, pos));
        Ok(())
    }

    /// Count the number of significant bits.
    /// Wraps `cx_bn_cnt_bits`.
    pub fn cnt_bits(&self) -> Result<u32, CxError> {
        let mut nbits = 0u32;
        check_cx_ok!(cx_bn_cnt_bits(self.handle, &mut nbits));
        Ok(nbits)
    }

    // ----- shifts -----------------------------------------------------------

    /// Shift right by `n` bits.
    /// Wraps `cx_bn_shr`.
    pub fn shr(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_shr(self.handle, n));
        Ok(())
    }

    /// Shift left by `n` bits.
    /// Wraps `cx_bn_shl`.
    pub fn shl(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_shl(self.handle, n));
        Ok(())
    }

    // ----- basic arithmetic -------------------------------------------------

    /// `self = a + b`.
    /// Wraps `cx_bn_add`.
    pub fn add(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_add(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a - b`.
    /// Wraps `cx_bn_sub`.
    pub fn sub(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_sub(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a * b`.
    /// The result BN must have been allocated with at least twice the byte
    /// capacity of the operands.
    /// Wraps `cx_bn_mul`.
    pub fn mul(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mul(self.handle, a.handle, b.handle));
        Ok(())
    }

    // ----- modular arithmetic -----------------------------------------------

    /// `self = (a + b) mod n`.
    /// Wraps `cx_bn_mod_add`.
    pub fn mod_add(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_add(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = (a - b) mod n`.
    /// Wraps `cx_bn_mod_sub`.
    pub fn mod_sub(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_sub(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = (a * b) mod n`.
    /// Wraps `cx_bn_mod_mul`.
    pub fn mod_mul(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_mul(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = d mod n`.
    /// Wraps `cx_bn_reduce`.
    pub fn reduce(&self, d: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_reduce(self.handle, d.handle, n.handle));
        Ok(())
    }

    /// `self = sqrt(a) mod n`, selecting the root with the given sign (0 or 1).
    /// Wraps `cx_bn_mod_sqrt`.
    pub fn mod_sqrt(&self, a: &Bn, n: &Bn, sign: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_sqrt(self.handle, a.handle, n.handle, sign));
        Ok(())
    }

    // ----- modular exponentiation -------------------------------------------

    /// `self = a^e mod n` where `e` is a BN.
    /// Wraps `cx_bn_mod_pow_bn`.
    pub fn mod_pow_bn(&self, a: &Bn, e: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_pow_bn(self.handle, a.handle, e.handle, n.handle));
        Ok(())
    }

    /// `self = a^e mod n` where `e` is a big-endian byte slice.
    /// Wraps `cx_bn_mod_pow`.
    pub fn mod_pow(&self, a: &Bn, e: &[u8], n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_pow(
            self.handle,
            a.handle,
            e.as_ptr(),
            e.len() as u32,
            n.handle,
        ));
        Ok(())
    }

    /// `self = a^e mod n` (alternate implementation).
    /// `e` is a big-endian byte slice.
    /// Wraps `cx_bn_mod_pow2`.
    pub fn mod_pow2(&self, a: &Bn, e: &[u8], n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_pow2(
            self.handle,
            a.handle,
            e.as_ptr(),
            e.len() as u32,
            n.handle,
        ));
        Ok(())
    }

    // ----- modular inversion ------------------------------------------------

    /// `self = a^(-1) mod n` where `n` is **not** prime.
    /// Wraps `cx_bn_mod_invert_nprime`.
    pub fn mod_invert_nprime(&self, a: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_invert_nprime(self.handle, a.handle, n.handle));
        Ok(())
    }

    /// `self = a^(-1) mod n` where `a` is a `u32`.
    /// Wraps `cx_bn_mod_u32_invert`.
    pub fn mod_u32_invert(&self, a: u32, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_u32_invert(self.handle, a, n.handle));
        Ok(())
    }

    // ----- primality --------------------------------------------------------

    /// Returns `true` if this BN is (probably) prime.
    /// Wraps `cx_bn_is_prime`.
    pub fn is_prime(&self) -> Result<bool, CxError> {
        let mut prime = false;
        check_cx_ok!(cx_bn_is_prime(self.handle, &mut prime));
        Ok(prime)
    }

    /// Replace this BN with the next prime ≥ its current value.
    /// Wraps `cx_bn_next_prime`.
    pub fn next_prime(&self) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_next_prime(self.handle));
        Ok(())
    }

    // ----- random -----------------------------------------------------------

    /// Set this BN to a random value in `[0, n)`.
    /// Wraps `cx_bn_rng`.
    pub fn rng(&self, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_rng(self.handle, n.handle));
        Ok(())
    }

    // ----- GF(2^n) ----------------------------------------------------------

    /// `self = a * b` in `GF(2^n)` defined by the irreducible polynomial
    /// `n` with pre-computed helper `h`.
    /// Wraps `cx_bn_gf2_n_mul`.
    pub fn gf2_n_mul(&self, a: &Bn, b: &Bn, n: &Bn, h: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_gf2_n_mul(
            self.handle,
            a.handle,
            b.handle,
            n.handle,
            h.handle,
        ));
        Ok(())
    }
}

impl Drop for Bn {
    fn drop(&mut self) {
        // Destroy is best-effort; the BN may already be invalid if the
        // BN context was unlocked prematurely.
        unsafe {
            cx_bn_destroy(&mut self.handle);
        }
    }
}

// =========================================================================
// MontCtx – RAII wrapper around cx_bn_mont_ctx_t
// =========================================================================

/// Safe RAII wrapper for a Montgomery multiplication context.
///
/// Allocated inside the locked BN context and destroyed on drop.
/// A [`BnLock`] **must** be held for the entire lifetime of a `MontCtx`.
///
/// # Example
///
/// ```ignore
/// let _lock = BnLock::acquire(32)?;
/// let n = Bn::alloc_init(&modulus_bytes)?;
/// let mut ctx = MontCtx::alloc(32)?;
/// ctx.init(&n)?;
/// ```
pub struct MontCtx {
    inner: cx_bn_mont_ctx_t,
}

impl MontCtx {
    /// Allocate a Montgomery context for BNs of `length` bytes.
    /// Wraps `cx_mont_alloc`.
    pub fn alloc(length: usize) -> Result<Self, CxError> {
        let mut inner = cx_bn_mont_ctx_t::default();
        check_cx_ok!(cx_mont_alloc(&mut inner, length));
        Ok(Self { inner })
    }

    /// Initialise the context from a modulus `n`.
    /// The Montgomery constant `h` is computed automatically.
    /// Wraps `cx_mont_init`.
    pub fn init(&mut self, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_init(&mut self.inner, n.handle));
        Ok(())
    }

    /// Initialise the context from a modulus `n` and a pre-computed
    /// Montgomery constant `h`.
    /// Wraps `cx_mont_init2`.
    pub fn init2(&mut self, n: &Bn, h: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_init2(&mut self.inner, n.handle, h.handle));
        Ok(())
    }

    /// Convert `z` into Montgomery representation and store in `x`.
    /// `x = z * R mod n`.
    /// Wraps `cx_mont_to_montgomery`.
    pub fn to_montgomery(&self, x: &Bn, z: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_to_montgomery(x.handle, z.handle, &self.inner));
        Ok(())
    }

    /// Convert `x` from Montgomery representation and store in `z`.
    /// `z = x * R^(-1) mod n`.
    /// Wraps `cx_mont_from_montgomery`.
    pub fn from_montgomery(&self, z: &Bn, x: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_from_montgomery(z.handle, x.handle, &self.inner));
        Ok(())
    }

    /// Montgomery multiplication: `r = a * b * R^(-1) mod n`.
    /// Wraps `cx_mont_mul`.
    pub fn mul(&self, r: &Bn, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_mul(r.handle, a.handle, b.handle, &self.inner));
        Ok(())
    }

    /// Montgomery exponentiation: `r = a^e mod n` (byte-slice exponent).
    /// Wraps `cx_mont_pow`.
    pub fn pow(&self, r: &Bn, a: &Bn, e: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_pow(
            r.handle,
            a.handle,
            e.as_ptr(),
            e.len() as u32,
            &self.inner,
        ));
        Ok(())
    }

    /// Montgomery exponentiation: `r = a^e mod n` (BN exponent).
    /// Wraps `cx_mont_pow_bn`.
    pub fn pow_bn(&self, r: &Bn, a: &Bn, e: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_pow_bn(r.handle, a.handle, e.handle, &self.inner));
        Ok(())
    }

    /// Montgomery modular inversion: `r = a^(-1) mod n` where `n` is not
    /// prime.
    /// Wraps `cx_mont_invert_nprime`.
    pub fn invert_nprime(&self, r: &Bn, a: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_invert_nprime(r.handle, a.handle, &self.inner));
        Ok(())
    }

    /// Access the raw `cx_bn_mont_ctx_t` for low-level FFI.
    pub fn as_raw(&self) -> &cx_bn_mont_ctx_t {
        &self.inner
    }

    /// Access the raw `cx_bn_mont_ctx_t` mutably.
    pub fn as_raw_mut(&mut self) -> &mut cx_bn_mont_ctx_t {
        &mut self.inner
    }
}

// =========================================================================
// Helpers
// =========================================================================

/// Round `n` up to the next multiple of the BN word alignment.
fn align_bn_size(n: usize) -> usize {
    let align = CX_BN_WORD_ALIGNEMENT as usize;
    (n + align - 1) & !(align - 1)
}

/// Map a C `int` comparison result to [`Ordering`].
fn int_to_ordering(diff: c_int) -> Ordering {
    match diff {
        d if d < 0 => Ordering::Less,
        0 => Ordering::Equal,
        _ => Ordering::Greater,
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    fn err_to_unit(e: CxError) {
        let ec = crate::testing::to_hex(e.into());
        crate::log::info!(
            "BN error: \x1b[1;33m{}\x1b[0m",
            core::str::from_utf8(&ec).unwrap()
        );
    }

    #[test]
    fn bn_alloc_set_get_u32() {
        let _lock = BnLock::acquire(32).map_err(err_to_unit)?;
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(12345).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 12345u32);
    }

    #[test]
    fn bn_add_sub() {
        let _lock = BnLock::acquire(32).map_err(err_to_unit)?;
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(100).map_err(err_to_unit)?;
        let b = Bn::alloc(32).map_err(err_to_unit)?;
        b.set_u32(42).map_err(err_to_unit)?;
        let r = Bn::alloc(32).map_err(err_to_unit)?;
        r.add(&a, &b).map_err(err_to_unit)?;
        assert_eq!(r.get_u32().map_err(err_to_unit)?, 142u32);
        r.sub(&a, &b).map_err(err_to_unit)?;
        assert_eq!(r.get_u32().map_err(err_to_unit)?, 58u32);
    }

    #[test]
    fn bn_cmp() {
        let _lock = BnLock::acquire(32).map_err(err_to_unit)?;
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(10).map_err(err_to_unit)?;
        let b = Bn::alloc(32).map_err(err_to_unit)?;
        b.set_u32(20).map_err(err_to_unit)?;
        assert_eq!(a.cmp_bn(&b).map_err(err_to_unit)?, Ordering::Less);
        assert_eq!(b.cmp_bn(&a).map_err(err_to_unit)?, Ordering::Greater);
        assert_eq!(a.cmp_u32(10).map_err(err_to_unit)?, Ordering::Equal);
    }

    #[test]
    fn bn_shift_bits() {
        let _lock = BnLock::acquire(32).map_err(err_to_unit)?;
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(0b1010).map_err(err_to_unit)?;
        a.shl(1).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 0b10100u32);
        a.shr(2).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 0b101u32);
    }
}
