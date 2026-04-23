use crate::ecc::{
    ChainCode, CurvesId, CxError, ECPrivateKey, ECPublicKey, HDKeyDeriveMode, Secret, SeedDerive,
    bip32_derive,
};
use crate::impl_curve;
use ledger_secure_sdk_sys::*;

pub mod stark;
pub use stark::*;

impl_curve!(Secp256k1, 32, 'W');
impl_curve!(Secp256r1, 32, 'W');
impl_curve!(Secp384r1, 48, 'W');
impl_curve!(BrainpoolP256R1, 32, 'W');
impl_curve!(BrainpoolP256T1, 32, 'W');
impl_curve!(BrainpoolP320R1, 40, 'W');
impl_curve!(BrainpoolP320T1, 40, 'W');
impl_curve!(BrainpoolP384R1, 48, 'W');
impl_curve!(BrainpoolP384T1, 48, 'W');
impl_curve!(BrainpoolP512R1, 64, 'W');
impl_curve!(BrainpoolP512T1, 64, 'W');
impl_curve!(Pallas, 32, 'W');

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

impl Pallas {
    /// Support ZIP32 Orchard derivation for Pallas
    /// Returns the derived secret key, and optionally fills the provided chain code if not None
    /// # Parameters
    /// - `path`: the derivation path as a slice of u32 integers
    /// - `cc`: an optional mutable reference to a ChainCode structure to be filled with the derived chain code
    /// - `seed`: an optional byte slice representing the seed for derivation, if the derivation mode requires it
    /// # Returns
    /// The derived secret key as a `Secret<32>`, or a `CxError` if the syscall fails
    pub fn zip32_orchard_derive(
        path: &[u32],
        cc: Option<&mut ChainCode>,
        seed: Option<&[u8]>,
    ) -> Result<Secret<32>, CxError> {
        let mut tmp = Secret::<32>::new();
        let (cc_ptr, cc_len) = match cc {
            Some(cc) => (cc.value.as_mut_ptr(), 32usize),
            None => (core::ptr::null_mut(), 0usize),
        };
        let (seed_ptr, seed_len) = match seed {
            Some(s) => (s.as_ptr() as *mut u8, s.len()),
            None => (core::ptr::null_mut(), 0usize),
        };
        let err = unsafe {
            sys_hdkey_derive(
                HDKeyDeriveMode::Zip32Orchard as u8,
                ledger_secure_sdk_sys::CX_CURVE_NONE,
                path.as_ptr(),
                path.len(),
                tmp.as_mut().as_mut_ptr(),
                32,
                cc_ptr,
                cc_len,
                seed_ptr,
                seed_len,
            )
        };
        if err != CX_OK {
            return Err(err.into());
        }
        let mut sk = Secret::<32>::new();
        let keylen = sk.0.len();
        sk.0.copy_from_slice(&tmp.0[..keylen]);
        Ok(sk)
    }
}

impl SeedDerive for Pallas {
    type Target = Secret<32>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut cc: ChainCode = Default::default();
        let sk = Self::zip32_orchard_derive(path, Some(&mut cc), None)
            .expect("zip32_orchard_derive failed");
        (sk, Some(cc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::ecc::make_bip32_path;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    trait ConstantFill {
        fn set_constant_key(&mut self);
    }

    impl<const N: usize, const TY: char> ConstantFill for ECPrivateKey<N, TY> {
        fn set_constant_key(&mut self) {
            let length = self.key.len();
            self.key[..length - 1].fill_with(|| 0xab);
        }
    }

    const PATH0: [u32; 5] = make_bip32_path(b"m/44'/133'/0'/0/0");
    const PATH1: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/1");

    fn display_error_code(e: CxError) {
        let ec = crate::testing::to_hex(e.into());
        crate::log::info!(
            "Error code: \x1b[1;33m{}\x1b[0m",
            core::str::from_utf8(&ec).unwrap()
        );
    }

    const TEST_HASH: &[u8; 13] = b"test_message1";

    #[test]
    fn zip32_orchard_pallas() {
        let sk1 = Pallas::zip32_orchard_derive(&PATH0, None, None).unwrap();
        let (sk2, cc) = Pallas::derive_from(&PATH0);
        assert_eq!(sk1, sk2);
        assert_eq!(cc.is_some(), true);
    }

    #[test]
    fn pubkey_secp256k1() {
        let sk_bytes = [0x77u8; 32];
        let pk = Secp256k1::from(&sk_bytes)
            .public_key()
            .map_err(display_error_code)?;
        let expected = [
            0x04, 0x79, 0x62, 0xd4, 0x5b, 0x38, 0xe8, 0xbc, 0xf8, 0x2f, 0xa8, 0xef, 0xa8, 0x43,
            0x2a, 0x1, 0xf2, 0xc, 0x9a, 0x53, 0xe2, 0x4c, 0x7d, 0x3f, 0x11, 0xdf, 0x19, 0x7c, 0xb8,
            0xe7, 0x9, 0x26, 0xda, 0x7a, 0x3e, 0xf3, 0xeb, 0xaf, 0xc7, 0x56, 0xdc, 0x3b, 0x24,
            0xb7, 0x52, 0x92, 0xd4, 0xcc, 0x5d, 0x71, 0xb1, 0x70, 0xe9, 0x70, 0x44, 0xa9, 0x85,
            0x83, 0x53, 0x44, 0x3a, 0x96, 0xba, 0xed, 0x23,
        ];
        assert_eq!(pk.as_ref(), &expected);
    }

    #[test]
    fn ecdsa_secp256k1() {
        let sk = Secp256k1::derive_from_path(&PATH0);
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_secp256r1() {
        let sk = Secp256r1::derive_from_path(&PATH0);
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_secp384r1() {
        let mut sk = Secp384r1::new();
        sk.set_constant_key();
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_brainpool256r1() {
        let mut sk = BrainpoolP256R1::new();
        sk.set_constant_key();
        let pk = sk.public_key().map_err(display_error_code)?;
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_brainpool320r1() {
        let mut sk = BrainpoolP320R1::new();
        sk.set_constant_key();
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_brainpool384r1() {
        let mut sk = BrainpoolP384R1::new();
        sk.set_constant_key();
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn ecdsa_brainpool512r1() {
        let mut sk = BrainpoolP512R1::new();
        sk.set_constant_key();
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn test_ecdh() {
        let sk0 = Secp256k1::derive_from_path(&PATH0);
        let pk0 = sk0.public_key().map_err(display_error_code)?;

        let sk1 = Secp256k1::derive_from_path(&PATH1);
        let pk1 = sk1.public_key().map_err(display_error_code)?;

        let shared_secret0 = sk1.ecdh(&pk0.pubkey).map_err(display_error_code)?;
        let shared_secret1 = sk0.ecdh(&pk1.pubkey).map_err(display_error_code)?;

        assert_eq!(shared_secret0, shared_secret1);
    }
}
