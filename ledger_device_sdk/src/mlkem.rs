//! ML-KEM (Module-Lattice Key Encapsulation Mechanism) support (FIPS 203).
//!
//! Provides safe Rust wrappers around the ML-KEM C implementation from the
//! Ledger C SDK (`lib_cxng`). Supports ML-KEM-512, ML-KEM-768, and ML-KEM-1024
//! parameter sets.
//!
//! # Memory considerations
//!
//! The underlying C routines use large stack-allocated workspaces. On the
//! memory-constrained Nano X (28 KB of SRAM), the default 8 KB heap leaves too
//! little stack: a key-generation, encapsulation, or decapsulation call can
//! overflow the stack into the heap and corrupt it. Apps enabling `mlkem` on
//! Nano X must therefore budget a smaller heap, for example by setting
//! `HEAP_SIZE="nanox: 2048"` (the per-target syntax only affects Nano X and
//! leaves the default heap on the other devices).
//!
//! This module is only available when the `mlkem` Cargo feature is enabled.

use ledger_secure_sdk_sys::*;

/// ML-KEM parameter set selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MlKemParam {
    /// ML-KEM-512 (NIST security level 1).
    MlKem512,
    /// ML-KEM-768 (NIST security level 3).
    MlKem768,
    /// ML-KEM-1024 (NIST security level 5).
    MlKem1024,
}

impl MlKemParam {
    const fn as_c(self) -> MLKEM_param_t {
        match self {
            MlKemParam::MlKem512 => MLKEM_512,
            MlKemParam::MlKem768 => MLKEM_768,
            MlKemParam::MlKem1024 => MLKEM_1024,
        }
    }
}

/// ML-KEM error type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MlKemError {
    InvalidParameter,
    InvalidParameterValue,
    InternalError,
}

impl From<u32> for MlKemError {
    fn from(code: u32) -> Self {
        match code {
            CX_INVALID_PARAMETER => MlKemError::InvalidParameter,
            CX_INVALID_PARAMETER_VALUE => MlKemError::InvalidParameterValue,
            _ => MlKemError::InternalError,
        }
    }
}

/// Size of the shared secret for all ML-KEM parameter sets (32 bytes).
pub const SHARED_SECRET_LEN: usize = MLKEM_SSBYTES as usize;

/// ML-KEM-512 public key size in bytes.
pub const MLKEM512_PK_LEN: usize = MLKEM512_PUBLICKEYBYTES as usize;
/// ML-KEM-512 secret key size in bytes.
pub const MLKEM512_SK_LEN: usize = MLKEM512_SECRETKEYBYTES as usize;
/// ML-KEM-512 ciphertext size in bytes.
pub const MLKEM512_CT_LEN: usize = MLKEM512_CIPHERTEXTBYTES as usize;

/// ML-KEM-768 public key size in bytes.
pub const MLKEM768_PK_LEN: usize = MLKEM768_PUBLICKEYBYTES as usize;
/// ML-KEM-768 secret key size in bytes.
pub const MLKEM768_SK_LEN: usize = MLKEM768_SECRETKEYBYTES as usize;
/// ML-KEM-768 ciphertext size in bytes.
pub const MLKEM768_CT_LEN: usize = MLKEM768_CIPHERTEXTBYTES as usize;

/// ML-KEM-1024 public key size in bytes.
pub const MLKEM1024_PK_LEN: usize = MLKEM1024_PUBLICKEYBYTES as usize;
/// ML-KEM-1024 secret key size in bytes.
pub const MLKEM1024_SK_LEN: usize = MLKEM1024_SECRETKEYBYTES as usize;
/// ML-KEM-1024 ciphertext size in bytes.
pub const MLKEM1024_CT_LEN: usize = MLKEM1024_CIPHERTEXTBYTES as usize;

impl MlKemParam {
    /// Returns the public key size in bytes for this parameter set.
    pub const fn pk_len(self) -> usize {
        match self {
            MlKemParam::MlKem512 => MLKEM512_PK_LEN,
            MlKemParam::MlKem768 => MLKEM768_PK_LEN,
            MlKemParam::MlKem1024 => MLKEM1024_PK_LEN,
        }
    }

    /// Returns the secret key size in bytes for this parameter set.
    pub const fn sk_len(self) -> usize {
        match self {
            MlKemParam::MlKem512 => MLKEM512_SK_LEN,
            MlKemParam::MlKem768 => MLKEM768_SK_LEN,
            MlKemParam::MlKem1024 => MLKEM1024_SK_LEN,
        }
    }

    /// Returns the ciphertext size in bytes for this parameter set.
    pub const fn ct_len(self) -> usize {
        match self {
            MlKemParam::MlKem512 => MLKEM512_CT_LEN,
            MlKemParam::MlKem768 => MLKEM768_CT_LEN,
            MlKemParam::MlKem1024 => MLKEM1024_CT_LEN,
        }
    }
}

/// Generates an ML-KEM key pair using internal randomness.
///
/// # Arguments
/// * `pk` - Output buffer for the public key (size must match `param.pk_len()`).
/// * `sk` - Output buffer for the secret key (size must match `param.sk_len()`).
/// * `param` - The ML-KEM parameter set to use.
pub fn keypair(pk: &mut [u8], sk: &mut [u8], param: MlKemParam) -> Result<(), MlKemError> {
    let err = unsafe {
        MLKEM_crypto_kem_keypair(
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

/// Performs ML-KEM encapsulation using internal randomness.
///
/// Produces a ciphertext and a shared secret from a public key.
///
/// # Arguments
/// * `ct` - Output buffer for the ciphertext (size must match `param.ct_len()`).
/// * `ss` - Output buffer for the shared secret ([`SHARED_SECRET_LEN`] bytes).
/// * `pk` - The recipient's public key.
/// * `param` - The ML-KEM parameter set to use.
pub fn encapsulate(
    ct: &mut [u8],
    ss: &mut [u8; SHARED_SECRET_LEN],
    pk: &[u8],
    param: MlKemParam,
) -> Result<(), MlKemError> {
    let err = unsafe {
        MLKEM_crypto_kem_enc(
            ct.as_mut_ptr(),
            ct.len(),
            ss.as_mut_ptr(),
            ss.len(),
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

/// Performs ML-KEM decapsulation.
///
/// Recovers the shared secret from a ciphertext and a secret key.
///
/// # Arguments
/// * `ss` - Output buffer for the shared secret ([`SHARED_SECRET_LEN`] bytes).
/// * `ct` - The ciphertext to decapsulate.
/// * `sk` - The recipient's secret key.
/// * `param` - The ML-KEM parameter set to use.
pub fn decapsulate(
    ss: &mut [u8; SHARED_SECRET_LEN],
    ct: &[u8],
    sk: &[u8],
    param: MlKemParam,
) -> Result<(), MlKemError> {
    let err = unsafe {
        MLKEM_crypto_kem_dec(
            ss.as_mut_ptr(),
            ss.len(),
            ct.as_ptr(),
            ct.len(),
            sk.as_ptr(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn test_mlkem512_keygen_encaps_decaps() {
        let mut pk = [0u8; MLKEM512_PK_LEN];
        let mut sk = [0u8; MLKEM512_SK_LEN];
        keypair(&mut pk, &mut sk, MlKemParam::MlKem512).unwrap();

        let mut ct = [0u8; MLKEM512_CT_LEN];
        let mut ss_enc = [0u8; SHARED_SECRET_LEN];
        encapsulate(&mut ct, &mut ss_enc, &pk, MlKemParam::MlKem512).unwrap();

        let mut ss_dec = [0u8; SHARED_SECRET_LEN];
        decapsulate(&mut ss_dec, &ct, &sk, MlKemParam::MlKem512).unwrap();

        assert_eq!(&ss_enc, &ss_dec);
    }

    #[test]
    fn test_mlkem768_keygen_encaps_decaps() {
        let mut pk = [0u8; MLKEM768_PK_LEN];
        let mut sk = [0u8; MLKEM768_SK_LEN];
        keypair(&mut pk, &mut sk, MlKemParam::MlKem768).unwrap();

        let mut ct = [0u8; MLKEM768_CT_LEN];
        let mut ss_enc = [0u8; SHARED_SECRET_LEN];
        encapsulate(&mut ct, &mut ss_enc, &pk, MlKemParam::MlKem768).unwrap();

        let mut ss_dec = [0u8; SHARED_SECRET_LEN];
        decapsulate(&mut ss_dec, &ct, &sk, MlKemParam::MlKem768).unwrap();

        assert_eq!(&ss_enc, &ss_dec);
    }

    #[test]
    fn test_mlkem1024_keygen_encaps_decaps() {
        let mut pk = [0u8; MLKEM1024_PK_LEN];
        let mut sk = [0u8; MLKEM1024_SK_LEN];
        keypair(&mut pk, &mut sk, MlKemParam::MlKem1024).unwrap();

        let mut ct = [0u8; MLKEM1024_CT_LEN];
        let mut ss_enc = [0u8; SHARED_SECRET_LEN];
        encapsulate(&mut ct, &mut ss_enc, &pk, MlKemParam::MlKem1024).unwrap();

        let mut ss_dec = [0u8; SHARED_SECRET_LEN];
        decapsulate(&mut ss_dec, &ct, &sk, MlKemParam::MlKem1024).unwrap();

        assert_eq!(&ss_enc, &ss_dec);
    }
}
