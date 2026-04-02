use crate::check_cx_ok;
use crate::ecc::{
    ChainCode, CurvesId, CxError, ECPrivateKey, ECPublicKey, Ed25519, HDKeyDeriveMode, JubJub,
    Secret, SeedDerive, bip32_derive,
};
use crate::hash::{HashInit, sha2::Sha2_512};
use ledger_secure_sdk_sys::*;

pub struct Ed25519Stream {
    hash: Sha2_512,
    pub big_r: [u8; 32],
    pub signature: [u8; 64],
}

impl Default for Ed25519Stream {
    fn default() -> Self {
        Ed25519Stream {
            hash: Sha2_512::default(),
            big_r: [0u8; 32],
            signature: [0u8; 64],
        }
    }
}

impl Ed25519Stream {
    pub fn init(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        // Compute prefix (see https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.6, step 1)
        let mut temp = Secret::<64>::new();
        self.hash.reset();
        self.hash
            .hash(&key.key[..], temp.as_mut())
            .map_err(|_| CxError::GenericError)?;
        self.hash.reset();
        self.hash
            .update(&temp.0[32..64])
            .map_err(|_| CxError::GenericError)?;
        Ok(())
    }

    fn compute_r(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        // Compute R (see https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.6, step 3)
        self.hash
            .finalize(&mut self.signature)
            .map_err(|_| CxError::GenericError)?;
        self.signature.reverse();

        check_cx_ok!(cx_bn_lock(32, 0));

        let mut r = CX_BN_FLAG_UNSET;

        check_cx_ok!(cx_bn_alloc_init(
            &mut r as *mut cx_bn_t,
            64,
            self.signature.as_ptr(),
            self.signature.len(),
        ));

        let mut ed_p = cx_ecpoint_t::default();
        // Get the generator for Ed25519's curve
        check_cx_ok!(cx_ecpoint_alloc(
            &mut ed_p as *mut cx_ecpoint_t,
            CX_CURVE_Ed25519
        ));
        check_cx_ok!(cx_ecdomain_generator_bn(CX_CURVE_Ed25519, &mut ed_p));

        // Multiply r by generator, store in ed_p
        check_cx_ok!(cx_ecpoint_scalarmul_bn(&mut ed_p, r));

        // and copy/compress it to ctx.big_r
        let mut sign = 0;

        check_cx_ok!(cx_ecpoint_compress(
            &ed_p,
            self.big_r.as_mut_ptr(),
            self.big_r.len(),
            &mut sign
        ));

        check_cx_ok!(cx_bn_unlock());

        self.big_r.reverse();
        self.big_r[31] |= if sign != 0 { 0x80 } else { 0x00 };

        // Compute S (see https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.6, step 4)
        self.hash.reset();
        self.hash
            .update(&self.big_r)
            .map_err(|_| CxError::GenericError)?;

        let mut pk = key.public_key()?;

        check_cx_ok!(cx_edwards_compress_point_no_throw(
            CX_CURVE_Ed25519,
            pk.pubkey.as_mut_ptr(),
            pk.keylength,
        ));
        // Note: public key has a byte in front of it in W, from how the ledger's system call
        // works; it's not for ed25519.
        self.hash
            .update(&pk.pubkey[1..33])
            .map_err(|_| CxError::GenericError)?;
        Ok(())
    }

    fn compute_s(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        // Compute S (see https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.6, step 5)
        check_cx_ok!(cx_bn_lock(32, 0));
        let (h_a, ed25519_order) = {
            let mut h_scalar = Secret::<64>::new();
            self.hash
                .finalize(h_scalar.as_mut())
                .map_err(|_| CxError::GenericError)?;

            h_scalar.0.reverse();

            // Make k into a BN
            let mut h_scalar_bn = CX_BN_FLAG_UNSET;
            check_cx_ok!(cx_bn_alloc_init(
                &mut h_scalar_bn as *mut cx_bn_t,
                64,
                h_scalar.0.as_ptr(),
                h_scalar.0.len(),
            ));

            // Get the group order
            let mut ed25519_order = CX_BN_FLAG_UNSET;

            check_cx_ok!(cx_bn_alloc(&mut ed25519_order, 64));

            check_cx_ok!(cx_ecdomain_parameter_bn(
                CX_CURVE_Ed25519,
                CX_CURVE_PARAM_Order,
                ed25519_order,
            ));

            // Generate the hashed private key
            let mut rv = CX_BN_FLAG_UNSET;
            let mut temp = Secret::<64>::new();

            self.hash.reset();
            self.hash
                .hash(&key.key[0..key.keylength], temp.as_mut())
                .map_err(|_| CxError::GenericError)?;

            // Bit twiddling for ed25519
            temp.0[0] &= 248;
            temp.0[31] &= 63;
            temp.0[31] |= 64;

            let key_slice = &mut temp.0[0..32];

            key_slice.reverse();
            let mut key_bn = CX_BN_FLAG_UNSET;

            // Load key into bn
            check_cx_ok!(cx_bn_alloc_init(
                &mut key_bn as *mut cx_bn_t,
                64,
                key_slice.as_ptr(),
                key_slice.len(),
            ));

            check_cx_ok!(cx_bn_alloc(&mut rv, 64));

            // multiply h_scalar_bn by key_bn
            check_cx_ok!(cx_bn_mod_mul(rv, key_bn, h_scalar_bn, ed25519_order));

            // Destroy the private key, so it doesn't leak from with_private_key even in the bn
            // area. temp will zeroize on drop already.
            check_cx_ok!(cx_bn_destroy(&mut key_bn));
            (rv, ed25519_order)
        };

        let mut r = CX_BN_FLAG_UNSET;
        check_cx_ok!(cx_bn_alloc_init(
            &mut r as *mut cx_bn_t,
            64,
            self.signature.as_ptr(),
            self.signature.len(),
        ));

        // finally, compute s:
        let mut s = CX_BN_FLAG_UNSET;
        check_cx_ok!(cx_bn_alloc(&mut s, 64));
        check_cx_ok!(cx_bn_mod_add(s, h_a, r, ed25519_order));

        // Spooky sub 0 to avoid Nano S+ bug
        check_cx_ok!(cx_bn_set_u32(r, 0));

        check_cx_ok!(cx_bn_mod_sub(s, s, r, ed25519_order));
        // and copy s back to normal memory to return.
        check_cx_ok!(cx_bn_export(s, self.signature.as_mut_ptr(), 32));
        check_cx_ok!(cx_bn_unlock());

        self.signature[..32].reverse();

        // Copy R[32] and S[32] into signature[64]
        self.signature.copy_within(0..32, 32);
        self.signature[0..32].copy_from_slice(&self.big_r);

        Ok(())
    }

    pub fn sign_finalize(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        match self.big_r.iter().all(|b| b == &0) {
            true => self.compute_r(key),
            false => self.compute_s(key),
        }
    }

    pub fn sign_update(&mut self, msg: &[u8]) -> Result<(), CxError> {
        self.hash.update(msg).map_err(|_| CxError::GenericError)?;
        Ok(())
    }
}

/// Edwards Curves-specific implementation
impl<const N: usize> ECPrivateKey<N, 'E'> {
    /// Size of an Edwards curve signature relative to the private key size
    pub const EP: usize = 2 * N;

    pub fn sign(&self, hash: &[u8]) -> Result<([u8; Self::EP], u32), CxError> {
        let mut sig = [0u8; Self::EP];
        let sig_len = Self::EP;
        let hash_id = match self.keylength {
            x if x <= 32 => CX_SHA512,
            _ => CX_BLAKE2B,
        };
        let len = unsafe {
            cx_eddsa_sign_no_throw(
                self as *const ECPrivateKey<N, 'E'> as *const cx_ecfp_256_private_key_s,
                hash_id,
                hash.as_ptr(),
                hash.len(),
                sig.as_mut_ptr(),
                sig_len,
            )
        };
        if len != CX_OK {
            Err(len.into())
        } else {
            Ok((sig, sig_len as u32))
        }
    }
}

/// Specific signature verification for Edwards curves, which all use EdDSA
impl<const P: usize> ECPublicKey<P, 'E'> {
    pub fn verify(&self, signature: (&[u8], u32), hash: &[u8], hash_id: u8) -> bool {
        unsafe {
            cx_eddsa_verify_no_throw(
                self as *const ECPublicKey<P, 'E'> as *const cx_ecfp_256_public_key_s,
                hash_id,
                hash.as_ptr(),
                hash.len(),
                signature.0.as_ptr(),
                signature.1 as usize,
            )
        }
    }
}

impl SeedDerive for Ed25519 {
    type Target = ECPrivateKey<32, 'E'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut tmp = Secret::<64>::new();
        let mut cc: ChainCode = Default::default();
        // Ignoring 'Result' here because known to be valid
        let _ = bip32_derive(
            CurvesId::Ed25519,
            path,
            tmp.as_mut(),
            Some(cc.value.as_mut()),
        );
        let mut sk = Self::Target::new(CurvesId::Ed25519);
        let keylen = sk.key.len();
        sk.key.copy_from_slice(&tmp.0[..keylen]);
        (sk, Some(cc))
    }
}

/// Support SLIP10 derivation for Ed25519
impl Ed25519 {
    pub fn derive_from_path_slip10(path: &[u32]) -> ECPrivateKey<32, 'E'> {
        let mut tmp = Secret::<64>::new();
        unsafe {
            os_perso_derive_node_with_seed_key(
                HDW_ED25519_SLIP10,
                CurvesId::Ed25519 as u8,
                path.as_ptr(),
                path.len() as u32,
                tmp.as_mut().as_mut_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
            );
        }
        let mut sk = ECPrivateKey::new(CurvesId::Ed25519);
        let keylen = sk.key.len();
        sk.key.copy_from_slice(&tmp.0[..keylen]);
        sk
    }
}

impl JubJub {
    /// Support ZIP32 Sapling derivation for JubJub
    /// Returns the derived ask, nsk, ovk and dk keys, and optionally fills the provided chain code if not None
    /// # Parameters
    /// - `path`: the derivation path as a slice of u32 integers
    /// - `cc`: an optional mutable reference to a ChainCode structure to be filled with the derived chain code
    /// - `seed`: an optional byte slice representing the seed for derivation, if the derivation mode requires it
    /// # Returns
    /// A tuple containing the derived ask, nsk, ovk and dk keys as Secret<32> structures
    pub fn zip32_sapling_derive(
        path: &[u32],
        cc: Option<&mut ChainCode>,
        seed: Option<&[u8]>,
    ) -> (Secret<32>, Secret<32>, Secret<32>, Secret<32>) {
        let mut tmp = Secret::<128>::new();
        unsafe {
            let err = sys_hdkey_derive(
                HDKeyDeriveMode::Zip32Sapling as u8,
                CurvesId::JubJub as cx_curve_t,
                path.as_ptr(),
                path.len(),
                tmp.as_mut().as_mut_ptr(),
                128,
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
        let mut ask = Secret::<32>::new();
        let mut nsk = Secret::<32>::new();
        let mut ovk = Secret::<32>::new();
        let mut dk = Secret::<32>::new();
        ask.0.copy_from_slice(&tmp.0[..32]);
        nsk.0.copy_from_slice(&tmp.0[32..64]);
        ovk.0.copy_from_slice(&tmp.0[64..96]);
        dk.0.copy_from_slice(&tmp.0[96..128]);
        (ask, nsk, ovk, dk)
    }
}

impl SeedDerive for JubJub {
    type Target = (Secret<32>, Secret<32>, Secret<32>, Secret<32>);
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut cc: ChainCode = Default::default();
        let keys = Self::zip32_sapling_derive(path, Some(&mut cc), None);
        (keys, Some(cc))
    }
}
