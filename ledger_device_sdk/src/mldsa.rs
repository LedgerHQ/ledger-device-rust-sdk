//! ML-DSA (Module-Lattice Digital Signature Algorithm) support (FIPS 204).
//!
//! Provides safe Rust wrappers around the ML-DSA C implementation from the
//! Ledger C SDK (`lib_cxng`). Supports ML-DSA-44 and ML-DSA-65 parameter sets,
//! plus ML-DSA-87 when the `mldsa_87` Cargo feature is enabled.
//!
//! The `mldsa_optimization` feature enables an alternative implementation that
//! trades RAM for speed.
//!
//! # Memory considerations
//!
//! The underlying C routines use large stack-allocated workspaces. On the
//! memory-constrained Nano X (28 KB of SRAM), the default 8 KB heap leaves too
//! little stack: a key-generation, signing, or verification call can overflow
//! the stack into the heap and corrupt it. Apps enabling `mldsa` on Nano X must
//! therefore budget a smaller heap, for example by setting
//! `HEAP_SIZE="nanox: 2048"` (the per-target syntax only affects Nano X and
//! leaves the default heap on the other devices).
//!
//! This module is only available when the `mldsa` Cargo feature is enabled.

use ledger_secure_sdk_sys::*;

/// ML-DSA parameter set selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MlDsaParam {
    /// ML-DSA-44 (NIST security level 2).
    MlDsa44,
    /// ML-DSA-65 (NIST security level 3).
    MlDsa65,
    /// ML-DSA-87 (NIST security level 5). Requires the `mldsa_87` feature.
    #[cfg(feature = "mldsa_87")]
    MlDsa87,
}

impl MlDsaParam {
    const fn as_c(self) -> MLDSA_param_t {
        match self {
            MlDsaParam::MlDsa44 => MLDSA_44,
            MlDsaParam::MlDsa65 => MLDSA_65,
            #[cfg(feature = "mldsa_87")]
            MlDsaParam::MlDsa87 => MLDSA_87,
        }
    }
}

/// Pre-hash algorithm selector for HashML-DSA (FIPS 204, Section 5.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MlDsaPrehash {
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
    Shake128,
    Shake256,
}

impl MlDsaPrehash {
    const fn as_c(self) -> MLDSA_prehash_t {
        match self {
            MlDsaPrehash::Sha256 => MLDSA_PREHASH_SHA256,
            MlDsaPrehash::Sha512 => MLDSA_PREHASH_SHA512,
            MlDsaPrehash::Sha3_256 => MLDSA_PREHASH_SHA3_256,
            MlDsaPrehash::Sha3_512 => MLDSA_PREHASH_SHA3_512,
            MlDsaPrehash::Shake128 => MLDSA_PREHASH_SHAKE128,
            MlDsaPrehash::Shake256 => MLDSA_PREHASH_SHAKE256,
        }
    }
}

/// ML-DSA error type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MlDsaError {
    InvalidParameter,
    InvalidParameterValue,
    InternalError,
}

impl From<u32> for MlDsaError {
    fn from(code: u32) -> Self {
        match code {
            CX_INVALID_PARAMETER => MlDsaError::InvalidParameter,
            CX_INVALID_PARAMETER_VALUE => MlDsaError::InvalidParameterValue,
            _ => MlDsaError::InternalError,
        }
    }
}

/// ML-DSA-44 public key size in bytes.
pub const MLDSA44_PK_LEN: usize = MLDSA44_PUBLICKEYBYTES as usize;
/// ML-DSA-44 secret key size in bytes.
pub const MLDSA44_SK_LEN: usize = MLDSA44_SECRETKEYBYTES as usize;
/// ML-DSA-44 signature size in bytes.
pub const MLDSA44_SIG_LEN: usize = MLDSA44_SIGBYTES as usize;

/// ML-DSA-65 public key size in bytes.
pub const MLDSA65_PK_LEN: usize = MLDSA65_PUBLICKEYBYTES as usize;
/// ML-DSA-65 secret key size in bytes.
pub const MLDSA65_SK_LEN: usize = MLDSA65_SECRETKEYBYTES as usize;
/// ML-DSA-65 signature size in bytes.
pub const MLDSA65_SIG_LEN: usize = MLDSA65_SIGBYTES as usize;

/// ML-DSA-87 public key size in bytes.
#[cfg(feature = "mldsa_87")]
pub const MLDSA87_PK_LEN: usize = MLDSA87_PUBLICKEYBYTES as usize;
/// ML-DSA-87 secret key size in bytes.
#[cfg(feature = "mldsa_87")]
pub const MLDSA87_SK_LEN: usize = MLDSA87_SECRETKEYBYTES as usize;
/// ML-DSA-87 signature size in bytes.
#[cfg(feature = "mldsa_87")]
pub const MLDSA87_SIG_LEN: usize = MLDSA87_SIGBYTES as usize;

/// Maximum context string length as per FIPS 204.
pub const MAX_CTX_LEN: usize = 255;

impl MlDsaParam {
    /// Returns the public key size in bytes for this parameter set.
    pub const fn pk_len(self) -> usize {
        match self {
            MlDsaParam::MlDsa44 => MLDSA44_PK_LEN,
            MlDsaParam::MlDsa65 => MLDSA65_PK_LEN,
            #[cfg(feature = "mldsa_87")]
            MlDsaParam::MlDsa87 => MLDSA87_PK_LEN,
        }
    }

    /// Returns the secret key size in bytes for this parameter set.
    pub const fn sk_len(self) -> usize {
        match self {
            MlDsaParam::MlDsa44 => MLDSA44_SK_LEN,
            MlDsaParam::MlDsa65 => MLDSA65_SK_LEN,
            #[cfg(feature = "mldsa_87")]
            MlDsaParam::MlDsa87 => MLDSA87_SK_LEN,
        }
    }

    /// Returns the signature size in bytes for this parameter set.
    pub const fn sig_len(self) -> usize {
        match self {
            MlDsaParam::MlDsa44 => MLDSA44_SIG_LEN,
            MlDsaParam::MlDsa65 => MLDSA65_SIG_LEN,
            #[cfg(feature = "mldsa_87")]
            MlDsaParam::MlDsa87 => MLDSA87_SIG_LEN,
        }
    }
}

/// Generates an ML-DSA key pair using internal randomness.
///
/// # Arguments
/// * `pk` - Output buffer for the public key (size must match `param.pk_len()`).
/// * `sk` - Output buffer for the secret key (size must match `param.sk_len()`).
/// * `param` - The ML-DSA parameter set to use.
pub fn keygen(pk: &mut [u8], sk: &mut [u8], param: MlDsaParam) -> Result<(), MlDsaError> {
    let err = unsafe {
        MLDSA_keygen(
            pk.as_mut_ptr(),
            pk.len(),
            sk.as_mut_ptr(),
            sk.len(),
            param.as_c(),
        )
    };
    if err != CX_OK {
        Err(err.into())
    } else {
        Ok(())
    }
}

/// Signs a message using ML-DSA.
///
/// # Arguments
/// * `sig` - Output buffer for the signature (size must match `param.sig_len()`).
/// * `msg` - The message to sign.
/// * `ctx` - Optional context string (at most [`MAX_CTX_LEN`] bytes, or empty slice).
/// * `sk` - The signer's secret key.
/// * `param` - The ML-DSA parameter set to use.
///
/// # Returns
/// The actual signature length on success.
pub fn sign(
    sig: &mut [u8],
    msg: &[u8],
    ctx: &[u8],
    sk: &[u8],
    param: MlDsaParam,
) -> Result<usize, MlDsaError> {
    if ctx.len() > MAX_CTX_LEN {
        return Err(MlDsaError::InvalidParameterValue);
    }
    let mut sig_actual_len: usize = 0;
    let ctx_ptr = if ctx.is_empty() {
        core::ptr::null()
    } else {
        ctx.as_ptr()
    };
    let err = unsafe {
        MLDSA_sign(
            sig.as_mut_ptr(),
            sig.len(),
            &mut sig_actual_len,
            msg.as_ptr(),
            msg.len(),
            ctx_ptr,
            ctx.len(),
            sk.as_ptr(),
            sk.len(),
            param.as_c(),
        )
    };
    if err != CX_OK {
        Err(err.into())
    } else {
        Ok(sig_actual_len)
    }
}

/// Verifies an ML-DSA signature.
///
/// # Arguments
/// * `sig` - The signature to verify.
/// * `msg` - The message that was signed.
/// * `ctx` - Optional context string (must match what was used during signing).
/// * `pk` - The signer's public key.
/// * `param` - The ML-DSA parameter set to use.
pub fn verify(
    sig: &[u8],
    msg: &[u8],
    ctx: &[u8],
    pk: &[u8],
    param: MlDsaParam,
) -> Result<(), MlDsaError> {
    if ctx.len() > MAX_CTX_LEN {
        return Err(MlDsaError::InvalidParameterValue);
    }
    let ctx_ptr = if ctx.is_empty() {
        core::ptr::null()
    } else {
        ctx.as_ptr()
    };
    let err = unsafe {
        MLDSA_verify(
            sig.as_ptr(),
            sig.len(),
            msg.as_ptr(),
            msg.len(),
            ctx_ptr,
            ctx.len(),
            pk.as_ptr(),
            pk.len(),
            param.as_c(),
        )
    };
    if err != CX_OK {
        Err(err.into())
    } else {
        Ok(())
    }
}

/// Signs a pre-hashed message using HashML-DSA (FIPS 204, Algorithm 4).
///
/// The caller must hash the message with the chosen algorithm before calling
/// this function.
///
/// # Arguments
/// * `sig` - Output buffer for the signature (size must match `param.sig_len()`).
/// * `ph` - The pre-hashed message digest.
/// * `ctx` - Optional context string (at most [`MAX_CTX_LEN`] bytes, or empty slice).
/// * `sk` - The signer's secret key.
/// * `prehash_alg` - The hash algorithm used to produce `ph`.
/// * `param` - The ML-DSA parameter set to use.
///
/// # Returns
/// The actual signature length on success.
pub fn sign_prehash(
    sig: &mut [u8],
    ph: &[u8],
    ctx: &[u8],
    sk: &[u8],
    prehash_alg: MlDsaPrehash,
    param: MlDsaParam,
) -> Result<usize, MlDsaError> {
    if ctx.len() > MAX_CTX_LEN {
        return Err(MlDsaError::InvalidParameterValue);
    }
    let mut sig_actual_len: usize = 0;
    let ctx_ptr = if ctx.is_empty() {
        core::ptr::null()
    } else {
        ctx.as_ptr()
    };
    let err = unsafe {
        MLDSA_sign_prehash(
            sig.as_mut_ptr(),
            sig.len(),
            &mut sig_actual_len,
            ph.as_ptr(),
            ph.len(),
            ctx_ptr,
            ctx.len(),
            sk.as_ptr(),
            sk.len(),
            prehash_alg.as_c(),
            param.as_c(),
        )
    };
    if err != CX_OK {
        Err(err.into())
    } else {
        Ok(sig_actual_len)
    }
}

/// Verifies a HashML-DSA pre-hash signature (FIPS 204, Algorithm 5).
///
/// The caller must hash the message with the chosen algorithm before calling
/// this function.
///
/// # Arguments
/// * `sig` - The signature to verify.
/// * `ph` - The pre-hashed message digest.
/// * `ctx` - Optional context string (must match what was used during signing).
/// * `pk` - The signer's public key.
/// * `prehash_alg` - The hash algorithm used to produce `ph`.
/// * `param` - The ML-DSA parameter set to use.
pub fn verify_prehash(
    sig: &[u8],
    ph: &[u8],
    ctx: &[u8],
    pk: &[u8],
    prehash_alg: MlDsaPrehash,
    param: MlDsaParam,
) -> Result<(), MlDsaError> {
    if ctx.len() > MAX_CTX_LEN {
        return Err(MlDsaError::InvalidParameterValue);
    }
    let ctx_ptr = if ctx.is_empty() {
        core::ptr::null()
    } else {
        ctx.as_ptr()
    };
    let err = unsafe {
        MLDSA_verify_prehash(
            sig.as_ptr(),
            sig.len(),
            ph.as_ptr(),
            ph.len(),
            ctx_ptr,
            ctx.len(),
            pk.as_ptr(),
            pk.len(),
            prehash_alg.as_c(),
            param.as_c(),
        )
    };
    if err != CX_OK {
        Err(err.into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_MSG: &[u8] = b"Test message";

    #[test]
    fn test_mldsa44_sign_verify() {
        let mut pk = [0u8; MLDSA44_PK_LEN];
        let mut sk = [0u8; MLDSA44_SK_LEN];
        keygen(&mut pk, &mut sk, MlDsaParam::MlDsa44).unwrap();

        let mut sig = [0u8; MLDSA44_SIG_LEN];
        let sig_len = sign(&mut sig, TEST_MSG, &[], &sk, MlDsaParam::MlDsa44).unwrap();
        assert_eq!(sig_len, MLDSA44_SIG_LEN);

        verify(&sig[..sig_len], TEST_MSG, &[], &pk, MlDsaParam::MlDsa44).unwrap();
    }

    #[test]
    fn test_mldsa65_sign_verify() {
        let mut pk = [0u8; MLDSA65_PK_LEN];
        let mut sk = [0u8; MLDSA65_SK_LEN];
        keygen(&mut pk, &mut sk, MlDsaParam::MlDsa65).unwrap();

        let mut sig = [0u8; MLDSA65_SIG_LEN];
        let sig_len = sign(&mut sig, TEST_MSG, &[], &sk, MlDsaParam::MlDsa65).unwrap();
        assert_eq!(sig_len, MLDSA65_SIG_LEN);

        verify(&sig[..sig_len], TEST_MSG, &[], &pk, MlDsaParam::MlDsa65).unwrap();
    }

    #[test]
    fn test_mldsa44_sign_verify_with_context() {
        let mut pk = [0u8; MLDSA44_PK_LEN];
        let mut sk = [0u8; MLDSA44_SK_LEN];
        keygen(&mut pk, &mut sk, MlDsaParam::MlDsa44).unwrap();

        let ctx = b"test context";
        let mut sig = [0u8; MLDSA44_SIG_LEN];
        let sig_len = sign(&mut sig, TEST_MSG, ctx, &sk, MlDsaParam::MlDsa44).unwrap();

        verify(&sig[..sig_len], TEST_MSG, ctx, &pk, MlDsaParam::MlDsa44).unwrap();
    }
}
