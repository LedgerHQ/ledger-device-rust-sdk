use crate::ecc::{
    ChainCode, CurvesId, CxError, ECPrivateKey, ECPublicKey, HDKeyDeriveMode, Pallas, Secp256k1,
    Secp256r1, Secret, SeedDerive, Stark256, bip32_derive,
};
use ledger_secure_sdk_sys::*;

/// Weierstrass Curves-specific implementation
impl<const N: usize> ECPrivateKey<N, 'W'> {
    /// Sign the incoming message/hash using ECDSA in the given `mode` and with the given hash identifier.
    /// This is a helper function. The two main interfaces are
    /// - [`deterministic_sign`]
    /// - [`sign`]
    fn ecdsa_sign(
        &self,
        hash: &[u8],
        hash_id: u8,
        mode: u32,
    ) -> Result<([u8; Self::S], u32, u32), CxError> {
        let mut sig = [0u8; Self::S];
        let mut sig_len = Self::S;
        let mut info = 0;
        let len = unsafe {
            cx_ecdsa_sign_no_throw(
                // Safety: cast awfully dodgy but that's how it's done in the C SDK
                self as *const ECPrivateKey<N, 'W'> as *const cx_ecfp_256_private_key_s,
                mode,
                hash_id,
                hash.as_ptr(),
                hash.len(),
                sig.as_mut_ptr(),
                &mut sig_len,
                &mut info,
            )
        };
        if len != CX_OK {
            Err(len.into())
        } else {
            Ok((sig, sig_len as u32, info & CX_ECCINFO_PARITY_ODD))
        }
    }

    /// Sign a message/hash using ECDSA with RFC6979, which provides a deterministic nonce rather than
    /// a random one. This nonce is computed using a hash function, hence this function uses an
    /// additional parameter `hash_id` that specifies which one it should use.
    pub fn deterministic_sign(&self, hash: &[u8]) -> Result<([u8; Self::S], u32, u32), CxError> {
        let hash_id = match self.keylength {
            x if x <= 32 => CX_SHA256,
            x if x <= 48 => CX_SHA384,
            x if x <= 64 => CX_SHA512,
            _ => CX_BLAKE2B,
        };
        self.ecdsa_sign(hash, hash_id, CX_RND_RFC6979 | CX_LAST)
    }

    /// Sign a message/hash using ECDSA in its original form
    pub fn sign(&self, hash: &[u8]) -> Result<([u8; Self::S], u32, u32), CxError> {
        self.ecdsa_sign(hash, 0, CX_RND_TRNG | CX_LAST)
    }

    /// Perform a Diffie-Hellman key exchange using the given uncompressed point `p`.
    /// Return the generated shared secret.
    /// We suppose the group size `N` is the same as the shared secret size.
    pub fn ecdh(&self, p: &[u8]) -> Result<[u8; N], CxError> {
        let mut secret = [0u8; N];
        let len = unsafe {
            cx_ecdh_no_throw(
                self as *const ECPrivateKey<N, 'W'> as *const cx_ecfp_256_private_key_s,
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

/// Specific signature verification for Weierstrass curves, which all use ECDSA.
impl<const P: usize> ECPublicKey<P, 'W'> {
    pub fn verify(&self, signature: (&[u8], u32), hash: &[u8]) -> bool {
        unsafe {
            cx_ecdsa_verify_no_throw(
                self as *const ECPublicKey<P, 'W'> as *const cx_ecfp_256_public_key_s,
                hash.as_ptr(),
                hash.len(),
                signature.0.as_ptr(),
                signature.1 as usize,
            )
        }
    }
}

impl SeedDerive for Secp256k1 {
    type Target = ECPrivateKey<32, 'W'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut tmp = Secret::<64>::new();
        let mut cc: ChainCode = Default::default();
        // Ignoring 'Result' here because known to be valid
        let _ = bip32_derive(
            CurvesId::Secp256k1,
            path,
            tmp.as_mut(),
            Some(cc.value.as_mut()),
        );
        let mut sk = Self::Target::new(CurvesId::Secp256k1);
        let keylen = sk.key.len();
        sk.key.copy_from_slice(&tmp.0[..keylen]);
        (sk, Some(cc))
    }
}

impl SeedDerive for Secp256r1 {
    type Target = ECPrivateKey<32, 'W'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut tmp = Secret::<64>::new();
        let mut cc: ChainCode = Default::default();
        // Ignoring 'Result' here because known to be valid
        let _ = bip32_derive(
            CurvesId::Secp256r1,
            path,
            tmp.as_mut(),
            Some(cc.value.as_mut()),
        );
        let mut sk = Self::Target::new(CurvesId::Secp256r1);
        let keylen = sk.key.len();
        sk.key.copy_from_slice(&tmp.0[..keylen]);
        (sk, Some(cc))
    }
}

impl SeedDerive for Stark256 {
    type Target = ECPrivateKey<32, 'W'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut sk = Self::Target::new(CurvesId::Stark256);
        super::stark::eip2645_derive(path, &mut sk.key);
        (sk, None)
    }
}

impl Pallas {
    /// Support ZIP32 Orchard derivation for Pallas
    /// Returns the derived secret key, and optionally fills the provided chain code if not None
    /// # Parameters
    /// - `path`: the derivation path as a slice of u32 integers
    /// - `cc`: an optional mutable reference to a ChainCode structure to be filled with the derived chain code
    /// - `seed`: an optional byte slice representing the seed for derivation, if the derivation mode requires it
    /// # Returns
    /// The derived secret key as a Secret<32> structure
    pub fn zip32_orchard_derive(
        path: &[u32],
        cc: Option<&mut ChainCode>,
        seed: Option<&[u8]>,
    ) -> Secret<32> {
        let mut tmp = Secret::<32>::new();
        unsafe {
            let err = sys_hdkey_derive(
                HDKeyDeriveMode::Zip32Orchard as u8,
                ledger_secure_sdk_sys::CX_CURVE_NONE,
                path.as_ptr(),
                path.len(),
                tmp.as_mut().as_mut_ptr(),
                32,
                match cc {
                    Some(ref cc) => cc.value.as_ptr() as *mut u8,
                    None => core::ptr::null_mut(),
                },
                match cc {
                    Some(_) => 32 as usize,
                    None => 0 as usize,
                },
                match seed {
                    Some(s) => s.as_ptr() as *mut u8,
                    None => core::ptr::null_mut(),
                },
                match seed {
                    Some(s) => s.len() as usize,
                    None => 0,
                },
            );
            if err != 0 {
                panic!("sys_hdkey_derive failed with error code {}", err);
            }
        }
        let mut sk = Secret::<32>::new();
        let keylen = sk.0.len();
        sk.0.copy_from_slice(&tmp.0[..keylen]);
        sk
    }
}

impl SeedDerive for Pallas {
    type Target = Secret<32>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut cc: ChainCode = Default::default();
        let sk = Self::zip32_orchard_derive(path, Some(&mut cc), None);
        (sk, Some(cc))
    }
}
