//! Safe Rust wrappers for the Ledger C SDK big-number (BN) and Montgomery
//! arithmetic functions declared in `ox_bn.h`.
//!
//! The underlying BN engine requires a global lock to be held while any
//! [`Bn`], [`EcPoint`](crate::ecc::math::EcPoint) or [`MontCtx`] is alive.
//! This lock is managed **automatically** via internal reference counting:
//! the first allocation acquires the lock and the last drop releases it.
//! Callers do **not** need to interact with [`BnLock`] at all.
//!
//! # Example
//!
//! ```ignore
//! use ledger_device_sdk::bn::Bn;
//!
//! let a = Bn::alloc(32)?;
//! a.set_u32(42)?;
//! let b = Bn::alloc_init(&[0, 0, 0, 7])?;
//! let r = Bn::alloc(32)?;
//! r.add(&a, &b)?;
//! // lock released automatically when a, b, r are dropped
//! ```

use crate::check_cx_ok;
use crate::ecc::CxError;
use core::cell::Cell;
use core::cmp::Ordering;
use core::ffi::c_int;
use ledger_secure_sdk_sys::*;

// =========================================================================
// Internal reference-counted BN lock management
// =========================================================================

/// Default word-size (in bytes) passed to `cx_bn_lock`.  32 bytes = 256 bits
/// covers all standard elliptic-curve sizes.  BN values larger than this
/// (e.g. 64 bytes for intermediate products) can still be allocated.
pub(crate) const BN_DEFAULT_WORD_NBYTES: usize = 32;

struct BnRefCount {
    count: Cell<u32>,
}

// Safety: the Ledger device is single-threaded.
unsafe impl Sync for BnRefCount {}

static BN_RC: BnRefCount = BnRefCount {
    count: Cell::new(0),
};

/// Increment the BN reference count, acquiring the lock on the first call.
/// `word_nbytes` is only used when the lock is actually acquired (count was 0).
pub(crate) fn bn_retain(word_nbytes: usize) -> Result<(), CxError> {
    let c = BN_RC.count.get();
    if c == 0 {
        check_cx_ok!(cx_bn_lock(word_nbytes, 0));
    }
    BN_RC.count.set(c + 1);
    Ok(())
}

/// Decrement the BN reference count, releasing the lock when it reaches zero.
pub(crate) fn bn_release() {
    let c = BN_RC.count.get();
    debug_assert!(c > 0, "bn_release called with zero ref count");
    let new = c - 1;
    BN_RC.count.set(new);
    if new == 0 {
        unsafe {
            cx_bn_unlock();
        }
    }
}

// =========================================================================
// BnLock – RAII guard for the big-number context
// =========================================================================

/// RAII guard for the BN lock context.
///
/// With the automatic reference-counted locking introduced in [`Bn`],
/// [`EcPoint`](crate::ecc::math::EcPoint) and [`MontCtx`], most callers
/// no longer need to use `BnLock` directly.  It is still available for
/// backward compatibility and for advanced use-cases that need explicit
/// control over lock lifetime.
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
    /// `word_nbytes` is the word alignment byte-size for the BN engine
    /// (e.g. 32 for 256-bit operations).  If the lock is already held
    /// (by a previous `acquire` or by an existing [`Bn`]/[`EcPoint`]
    /// allocation), this simply increments the internal reference count.
    /// # Arguments
    /// * `word_nbytes` - The word alignment byte-size for the BN engine
    /// # Returns
    /// Returns a `BnLock` guard on success, or a `CxError` if the lock could not be acquired.
    pub fn acquire(word_nbytes: usize) -> Result<Self, CxError> {
        bn_retain(word_nbytes)?;
        Ok(BnLock)
    }

    /// Explicitly release the lock (equivalent to dropping the guard).
    pub fn release(self) {
        drop(self);
    }

    /// Returns `true` if the BN context is currently locked.
    pub fn is_locked() -> bool {
        unsafe { cx_bn_locked() == CX_OK }
    }

    /// Returns `true` if the BN context is currently locked (alternate API).
    pub fn is_locked_bool() -> bool {
        unsafe { cx_bn_is_locked() }
    }
}

impl Drop for BnLock {
    fn drop(&mut self) {
        bn_release();
    }
}

// =========================================================================
// Bn – RAII wrapper around cx_bn_t
// =========================================================================

/// Safe RAII wrapper around a single big-number handle (`cx_bn_t`).
///
/// The BN is allocated inside the locked BN context and automatically
/// destroyed when dropped.  The BN lock is acquired transparently on
/// the first allocation and released when the last `Bn` (or
/// [`EcPoint`](crate::ecc::math::EcPoint) / [`MontCtx`]) is dropped.
#[derive(Debug)]
pub struct Bn {
    handle: cx_bn_t,
}

impl Bn {
    // ----- allocation / init ------------------------------------------------

    /// Allocate an uninitialised BN with room for `nbytes` bytes.
    /// # Arguments
    /// * `nbytes` - The byte capacity of the BN to allocate
    /// # Returns
    /// Returns a new `Bn` instance on success, or a `CxError` if the allocation fails.
    pub fn alloc(nbytes: usize) -> Result<Self, CxError> {
        bn_retain(BN_DEFAULT_WORD_NBYTES)?;
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        let err = unsafe { cx_bn_alloc(&mut handle, nbytes) };
        if err != CX_OK {
            bn_release();
            return Err(err.into());
        }
        Ok(Self { handle })
    }

    /// Allocate a BN and initialise it from a big-endian byte slice.
    /// The BN will have room for `nbytes` bytes where `nbytes` is at least
    /// `value.len()` rounded up to the BN word alignment.
    /// # Arguments
    /// * `value` - The big-endian byte slice to initialise the BN with
    /// # Returns
    /// Returns a new `Bn` instance on success, or a `CxError` if the allocation or initialisation fails.
    pub fn alloc_init(value: &[u8]) -> Result<Self, CxError> {
        bn_retain(BN_DEFAULT_WORD_NBYTES)?;
        let nbytes = align_bn_size(value.len());
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        let err = unsafe {
            cx_bn_alloc_init(&mut handle, nbytes, value.as_ptr(), value.len())
        };
        if err != CX_OK {
            bn_release();
            return Err(err.into());
        }
        Ok(Self { handle })
    }

    /// Allocate a BN with `nbytes` capacity and initialise from a byte slice.
    /// # Arguments
    /// * `nbytes` - The byte capacity of the BN to allocate (must be at least `value.len()` rounded up to the BN word alignment)
    /// * `value` - The big-endian byte slice to initialise the BN with
    /// # Returns
    /// Returns a new `Bn` instance on success, or a `CxError` if the allocation or initialisation fails.
    pub fn alloc_init_size(nbytes: usize, value: &[u8]) -> Result<Self, CxError> {
        bn_retain(BN_DEFAULT_WORD_NBYTES)?;
        let mut handle: cx_bn_t = CX_BN_FLAG_UNSET;
        let err = unsafe {
            cx_bn_alloc_init(&mut handle, nbytes, value.as_ptr(), value.len())
        };
        if err != CX_OK {
            bn_release();
            return Err(err.into());
        }
        Ok(Self { handle })
    }

    /// Return the raw `cx_bn_t` handle for use with low-level FFI.
    /// # Arguments
    /// * `self` - The `Bn` instance to access
    /// # Returns
    /// Returns the raw `cx_bn_t` handle for use with low-level FFI.
    pub fn raw(&self) -> cx_bn_t {
        self.handle
    }

    /// Return the raw `cx_bn_t` handle mutably for use with low-level FFI.
    /// # Arguments
    /// * `self` - The `Bn` instance to access
    /// # Returns
    /// Returns the raw `cx_bn_t` handle mutably for use with low-level FFI.
    pub fn raw_mut(&mut self) -> &mut cx_bn_t {
        &mut self.handle
    }

    // ----- init / copy / size -----------------------------------------------

    /// (Re-)initialise this BN from a big-endian byte slice.
    /// # Arguments
    /// * `self` - The `Bn` instance to initialise
    /// * `value` - The big-endian byte slice to initialise the BN with
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the initialisation fails.
    pub fn init(&self, value: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_init(self.handle, value.as_ptr(), value.len()));
        Ok(())
    }

    /// Fill this BN with random data.
    /// # Arguments
    /// * `self` - The `Bn` instance to fill with random data
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn rand(&self) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_rand(self.handle));
        Ok(())
    }

    /// Copy the value of `other` into `self`.
    /// # Arguments
    /// * `self` - The `Bn` instance to copy into
    /// * `other` - The `Bn` instance to copy from
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn copy_from(&self, other: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_copy(self.handle, other.handle));
        Ok(())
    }

    /// Return the size in bytes of this BN.
    /// # Arguments
    /// * `self` - The `Bn` instance to query
    /// # Returns
    /// Returns the size in bytes of this BN on success, or a `CxError` if the operation fails.
    pub fn nbytes(&self) -> Result<usize, CxError> {
        let mut n = 0usize;
        check_cx_ok!(cx_bn_nbytes(self.handle, &mut n));
        Ok(n)
    }

    // ----- u32 get/set & export ---------------------------------------------

    /// Set this BN to the given `u32` value.
    /// # Arguments
    /// * `self` - The `Bn` instance to set
    /// * `n` - The `u32` value to set
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn set_u32(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_set_u32(self.handle, n));
        Ok(())
    }

    /// Return the value of this BN as a `u32`.
    /// The BN must fit in 32 bits.
    /// # Arguments
    /// * `self` - The `Bn` instance to query
    /// # Returns
    /// Returns the `u32` value on success, or a `CxError` if the operation fails.
    pub fn get_u32(&self) -> Result<u32, CxError> {
        let mut n = 0u32;
        check_cx_ok!(cx_bn_get_u32(self.handle, &mut n));
        Ok(n)
    }

    /// Export this BN into a big-endian byte buffer.
    /// # Arguments
    /// * `self` - The `Bn` instance to export
    /// * `bytes` - The buffer to export the BN into
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn export(&self, bytes: &mut [u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_export(self.handle, bytes.as_mut_ptr(), bytes.len()));
        Ok(())
    }

    // ----- comparison -------------------------------------------------------

    /// Compare two BNs.
    /// Returns `Ordering::Less`, `Equal`, or `Greater`.
    /// # Arguments
    /// * `self` - The first `Bn` instance to compare
    /// * `other` - The second `Bn` instance to compare
    /// # Returns
    /// Returns `Ordering::Less`, `Equal`, or `Greater` on success, or a `CxError` if the comparison fails.
    pub fn cmp_bn(&self, other: &Bn) -> Result<Ordering, CxError> {
        let mut diff: c_int = 0;
        check_cx_ok!(cx_bn_cmp(self.handle, other.handle, &mut diff));
        Ok(int_to_ordering(diff))
    }

    /// Compare this BN with a `u32`.
    /// Returns `Ordering::Less`, `Equal`, or `Greater`.
    /// # Arguments
    /// * `self` - The `Bn` instance to compare
    /// * `other` - The `u32` value to compare with
    /// # Returns
    /// Returns `Ordering::Less`, `Equal`, or `Greater` on success, or a `CxError` if the comparison fails.
    pub fn cmp_u32(&self, other: u32) -> Result<Ordering, CxError> {
        let mut diff: c_int = 0;
        check_cx_ok!(cx_bn_cmp_u32(self.handle, other, &mut diff));
        Ok(int_to_ordering(diff))
    }

    /// Returns `true` if this BN is odd.
    /// # Arguments
    /// * `self` - The `Bn` instance to query
    /// # Returns
    /// Returns `true` if the BN is odd, or a `CxError` if the operation fails.
    pub fn is_odd(&self) -> Result<bool, CxError> {
        let mut odd = false;
        check_cx_ok!(cx_bn_is_odd(self.handle, &mut odd));
        Ok(odd)
    }

    // ----- bitwise logic ----------------------------------------------------

    /// `self = a XOR b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn xor(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_xor(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a OR b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn or(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_or(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a AND b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn and(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_and(self.handle, a.handle, b.handle));
        Ok(())
    }

    // ----- bit manipulation -------------------------------------------------

    /// Test whether bit at position `pos` is set.
    /// # Arguments
    /// * `self` - The `Bn` instance to query
    /// * `pos` - The bit position to test
    /// # Returns
    /// Returns `true` if the bit is set, or a `CxError` if the operation fails.
    pub fn tst_bit(&self, pos: u32) -> Result<bool, CxError> {
        let mut set = false;
        check_cx_ok!(cx_bn_tst_bit(self.handle, pos, &mut set));
        Ok(set)
    }

    /// Set bit at position `pos`.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// * `pos` - The bit position to set
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn set_bit(&self, pos: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_set_bit(self.handle, pos));
        Ok(())
    }

    /// Clear bit at position `pos`.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// * `pos` - The bit position to clear
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn clr_bit(&self, pos: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_clr_bit(self.handle, pos));
        Ok(())
    }

    /// Count the number of significant bits.
    /// # Arguments
    /// * `self` - The `Bn` instance to query
    /// # Returns
    /// Returns the number of significant bits, or a `CxError` if the operation fails.
    pub fn cnt_bits(&self) -> Result<u32, CxError> {
        let mut nbits = 0u32;
        check_cx_ok!(cx_bn_cnt_bits(self.handle, &mut nbits));
        Ok(nbits)
    }

    // ----- shifts -----------------------------------------------------------

    /// Shift right by `n` bits.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// * `n` - The number of bits to shift
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn shr(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_shr(self.handle, n));
        Ok(())
    }

    /// Shift left by `n` bits.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// * `n` - The number of bits to shift
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn shl(&self, n: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_shl(self.handle, n));
        Ok(())
    }

    // ----- basic arithmetic -------------------------------------------------

    /// `self = a + b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn add(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_add(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a - b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn sub(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_sub(self.handle, a.handle, b.handle));
        Ok(())
    }

    /// `self = a * b`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    /// The result BN must have been allocated with at least twice the byte
    /// capacity of the operands.
    pub fn mul(&self, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mul(self.handle, a.handle, b.handle));
        Ok(())
    }

    // ----- modular arithmetic -----------------------------------------------

    /// `self = (a + b) mod n`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// * `n` - The modulus
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_add(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_add(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = (a - b) mod n`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// * `n` - The modulus
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_sub(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_sub(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = (a * b) mod n`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first `Bn` operand
    /// * `b` - The second `Bn` operand
    /// * `n` - The modulus
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_mul(&self, a: &Bn, b: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_mul(self.handle, a.handle, b.handle, n.handle));
        Ok(())
    }

    /// `self = d mod n`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `d` - The `Bn` instance representing the value to reduce
    /// * `n` - The modulus
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn reduce(&self, d: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_reduce(self.handle, d.handle, n.handle));
        Ok(())
    }

    /// `self = sqrt(a) mod n`, selecting the root with the given sign (0 or 1).
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance representing the value to take the square root of
    /// * `n` - The modulus
    /// * `sign` - The sign of the root (0 or 1)
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_sqrt(&self, a: &Bn, n: &Bn, sign: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_sqrt(self.handle, a.handle, n.handle, sign));
        Ok(())
    }

    // ----- modular exponentiation -------------------------------------------

    /// `self = a^e mod n` where `e` is a BN.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The base `Bn` instance
    /// * `e` - The exponent `Bn` instance
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_pow_bn(&self, a: &Bn, e: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_pow_bn(self.handle, a.handle, e.handle, n.handle));
        Ok(())
    }

    /// `self = a^e mod n` where `e` is a big-endian byte slice.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The base `Bn` instance
    /// * `e` - The exponent as a big-endian byte slice
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
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
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The base `Bn` instance
    /// * `e` - The exponent as a big-endian byte slice
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
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
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance representing the value to invert
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_invert_nprime(&self, a: &Bn, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_invert_nprime(self.handle, a.handle, n.handle));
        Ok(())
    }

    /// `self = a^(-1) mod n` where `a` is a `u32`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The `u32` value to invert
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mod_u32_invert(&self, a: u32, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_mod_u32_invert(self.handle, a, n.handle));
        Ok(())
    }

    // ----- primality --------------------------------------------------------

    /// Returns `true` if this BN is (probably) prime.
    /// # Arguments
    /// * `self` - The `Bn` instance to check for primality
    /// # Returns
    /// Returns `Ok(true)` if the BN is probably prime, `Ok(false)` otherwise,
    /// or a `CxError` if the operation fails.
    pub fn is_prime(&self) -> Result<bool, CxError> {
        let mut prime = false;
        check_cx_ok!(cx_bn_is_prime(self.handle, &mut prime));
        Ok(prime)
    }

    /// Replace this BN with the next prime ≥ its current value.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn next_prime(&self) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_next_prime(self.handle));
        Ok(())
    }

    // ----- random -----------------------------------------------------------

    /// Set this BN to a random value in `[0, n)`.
    /// # Arguments
    /// * `self` - The `Bn` instance to modify
    /// * `n` - The upper bound `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn rng(&self, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_bn_rng(self.handle, n.handle));
        Ok(())
    }

    // ----- GF(2^n) ----------------------------------------------------------

    /// `self = a * b` in `GF(2^n)` defined by the irreducible polynomial
    /// `n` with pre-computed helper `h`.
    /// # Arguments
    /// * `self` - The `Bn` instance to store the result
    /// * `a` - The first operand `Bn` instance
    /// * `b` - The second operand `Bn` instance
    /// * `n` - The irreducible polynomial `Bn` instance
    /// * `h` - The pre-computed helper `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
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
        unsafe {
            cx_bn_destroy(&mut self.handle);
        }
        bn_release();
    }
}

// =========================================================================
// MontCtx – RAII wrapper around cx_bn_mont_ctx_t
// =========================================================================

/// Safe RAII wrapper for a Montgomery multiplication context.
///
/// Allocated inside the locked BN context and destroyed on drop.
/// The BN lock is managed automatically via reference counting.
///
/// # Example
///
/// ```ignore
/// let n = Bn::alloc_init(&modulus_bytes)?;
/// let mut ctx = MontCtx::alloc(32)?;
/// ctx.init(&n)?;
/// ```
pub struct MontCtx {
    inner: cx_bn_mont_ctx_t,
}

impl MontCtx {
    /// Allocate a Montgomery context for BNs of `length` bytes.
    /// # Arguments
    /// * `length` - The length in bytes for the BNs
    /// # Returns
    /// Returns `Ok(MontCtx)` on success, or a `CxError` if the operation fails.
    pub fn alloc(length: usize) -> Result<Self, CxError> {
        bn_retain(BN_DEFAULT_WORD_NBYTES)?;
        let mut inner = cx_bn_mont_ctx_t::default();
        let err = unsafe { cx_mont_alloc(&mut inner, length) };
        if err != CX_OK {
            bn_release();
            return Err(err.into());
        }
        Ok(Self { inner })
    }

    /// Initialise the context from a modulus `n`.
    /// The Montgomery constant `h` is computed automatically.
    /// # Arguments
    /// * `self` - The `MontCtx` instance to initialise
    /// * `n` - The modulus `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn init(&mut self, n: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_init(&mut self.inner, n.handle));
        Ok(())
    }

    /// Initialise the context from a modulus `n` and a pre-computed
    /// Montgomery constant `h`.
    /// # Arguments
    /// * `self` - The `MontCtx` instance to initialise
    /// * `n` - The modulus `Bn` instance
    /// * `h` - The pre-computed Montgomery constant `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn init2(&mut self, n: &Bn, h: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_init2(&mut self.inner, n.handle, h.handle));
        Ok(())
    }

    /// Convert `z` into Montgomery representation and store in `x`.
    /// `x = z * R mod n`.
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `x` - The `Bn` instance to store the result
    /// * `z` - The `Bn` instance to convert
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn to_montgomery(&self, x: &Bn, z: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_to_montgomery(x.handle, z.handle, &self.inner));
        Ok(())
    }

    /// Convert `x` from Montgomery representation and store in `z`.
    /// `z = x * R^(-1) mod n`.
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `z` - The `Bn` instance to store the result
    /// * `x` - The `Bn` instance to convert
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn from_montgomery(&self, z: &Bn, x: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_from_montgomery(z.handle, x.handle, &self.inner));
        Ok(())
    }

    /// Montgomery multiplication: `r = a * b * R^(-1) mod n`.
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `r` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance
    /// * `b` - The `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn mul(&self, r: &Bn, a: &Bn, b: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_mul(r.handle, a.handle, b.handle, &self.inner));
        Ok(())
    }

    /// Montgomery exponentiation: `r = a^e mod n` (byte-slice exponent).
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `r` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance
    /// * `e` - The byte-slice exponent
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
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
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `r` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance
    /// * `e` - The `Bn` instance exponent
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
    pub fn pow_bn(&self, r: &Bn, a: &Bn, e: &Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_mont_pow_bn(r.handle, a.handle, e.handle, &self.inner));
        Ok(())
    }

    /// Montgomery modular inversion: `r = a^(-1) mod n` where `n` is not
    /// prime.
    /// # Arguments
    /// * `self` - The `MontCtx` instance
    /// * `r` - The `Bn` instance to store the result
    /// * `a` - The `Bn` instance
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the operation fails.
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

impl Drop for MontCtx {
    fn drop(&mut self) {
        bn_release();
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
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(12345).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 12345u32);
    }

    #[test]
    fn bn_add_sub() {
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
        let a = Bn::alloc(32).map_err(err_to_unit)?;
        a.set_u32(0b1010).map_err(err_to_unit)?;
        a.shl(1).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 0b10100u32);
        a.shr(2).map_err(err_to_unit)?;
        assert_eq!(a.get_u32().map_err(err_to_unit)?, 0b101u32);
    }
}
