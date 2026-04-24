use crate::bn::*;
use crate::check_cx_ok;
use crate::ecc::{CurvesId, CxError, ECPrivateKey};
use crate::impl_curve;
use ledger_secure_sdk_sys::*;

// Montgomery curves (Curve25519, Curve448)

impl_curve!(Curve25519, 32, 'M');
impl_curve!(Curve448, 56, 'M');

impl Curve25519 {
    /// Perform scalar multiplication on Curve25519: `self = k · self`.
    /// # Arguments
    /// * `u` - The BN handle representing the point to multiply (input and output)
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails (e.g. invalid scalar length).
    /// To be used when several multiplications are needed, to avoid the overhead of locking and unlocking the BN handle at each multiplication.
    pub fn sys_scalar_mul(u: &mut Bn, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_x25519(u.raw(), k.as_ptr(), k.len()));
        Ok(())
    }

    /// Perform scalar multiplication on Curve25519: `self = k · self`.
    /// # Arguments
    /// * `u` - The byte array representing the point to multiply (input and output)
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails (e.g. invalid scalar length).
    pub fn scalar_mul(u: &mut [u8], k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_x25519(u.as_mut_ptr(), k.as_ptr(), k.len()));
        Ok(())
    }
}

impl Curve448 {
    /// Perform scalar multiplication on Curve448: `self = k · self`.
    /// # Arguments
    /// * `u` - The BN handle representing the point to multiply (input and output)
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails (e.g. invalid scalar length).
    /// To be used when several multiplications are needed, to avoid the overhead of locking and unlocking the BN handle at each multiplication.
    pub fn sys_scalar_mul(u: &mut Bn, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_x448(u.raw(), k.as_ptr(), k.len()));
        Ok(())
    }

    /// Perform scalar multiplication on Curve448: `self = k · self`.
    /// # Arguments
    /// * `u` - The byte array representing the point to multiply (input and output)
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails (e.g. invalid scalar length).
    pub fn scalar_mul(u: &mut [u8], k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_x448(u.as_mut_ptr(), k.as_ptr(), k.len()));
        Ok(())
    }
}

impl<const N: usize> ECPrivateKey<N, 'M'> {
    /// Perform a Diffie-Hellman key exchange using the given uncompressed point `p`.
    /// Return the generated shared secret.
    /// We suppose the group size `N` is the same as the shared secret size.
    pub fn ecdh(&self, p: &[u8]) -> Result<[u8; N], CxError> {
        let mut secret = [0u8; N];
        let len = unsafe {
            cx_ecdh_no_throw(
                self as *const ECPrivateKey<N, 'M'> as *const cx_ecfp_256_private_key_s,
                CX_ECDH_X,
                p.as_ptr(),
                p.len(),
                secret.as_mut_ptr(),
                N,
            )
        };
        if len != CX_OK {
            Err(len.into())
        } else {
            Ok(secret)
        }
    }
}
