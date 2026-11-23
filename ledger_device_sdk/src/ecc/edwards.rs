use crate::bn::{Bn, bn_is_held};
use crate::check_cx_ok;
use crate::ecc::math::{CurveDomainParam, EcPoint};
use crate::ecc::{
    ChainCode, CurvesId, CxError, ECPrivateKey, ECPublicKey, HDKeyDeriveMode, Secret, SeedDerive,
    SignatureSize, bip32_derive,
};
use crate::hash::{
    HashInit,
    sha2::{Sha2_256, Sha2_512},
};
use crate::impl_curve;
use ledger_secure_sdk_sys::*;
use zeroize::Zeroize;

impl_curve!(Ed25519, 32, 'E');
impl_curve!(JubJub, 32, 'E');

/// Where an [`Ed25519Stream`] is in its two-pass signing flow.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum SignPhase {
    #[default]
    Uninitialized,
    /// First message pass, the nonce is derived from.
    NoncePass,
    /// Second message pass, the challenge is derived from.
    ChallengePass,
    Complete,
}

/// Streaming PureEdDSA (RFC 8032) Ed25519 signer, for messages too long to be
/// buffered.
///
/// After [`init`](Self::init), the message is streamed twice through
/// [`sign_update`](Self::sign_update), each pass being ended by
/// [`sign_finalize`](Self::sign_finalize): the first pass derives the nonce,
/// the second one the challenge. Once both are done, the signature is available
/// from [`signature`](Self::signature).
///
/// Both passes must carry exactly the same message, and both
/// `sign_finalize` calls must be given the key passed to `init`:
/// `sign_finalize` returns an error otherwise.
pub struct Ed25519Stream {
    hash: Sha2_512,
    /// Fingerprint of the key given to `init`.
    key_id: [u8; 32],
    /// Hash of the message alone, to check that both passes match.
    msg_hash: Sha2_512,
    /// Digest of the first pass message.
    msg_digest: [u8; 64],
    phase: SignPhase,
    big_r: [u8; 32],
    /// Holds the nonce `r` between both passes, then the signature.
    signature: [u8; 64],
}

impl Default for Ed25519Stream {
    fn default() -> Self {
        Ed25519Stream {
            hash: Sha2_512::default(),
            key_id: [0u8; 32],
            msg_hash: Sha2_512::default(),
            msg_digest: [0u8; 64],
            phase: SignPhase::Uninitialized,
            big_r: [0u8; 32],
            signature: [0u8; 64],
        }
    }
}

impl Drop for Ed25519Stream {
    fn drop(&mut self) {
        self.wipe();
    }
}

/// Constant-time equality, so that the comparison leaks nothing about the digests.
fn ct_eq<const N: usize>(a: &[u8; N], b: &[u8; N]) -> bool {
    let diff = a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    core::hint::black_box(diff) == 0
}

/// One-way fingerprint of `key`, identifying it without holding it.
fn key_id(key: &ECPrivateKey<32, 'E'>) -> Result<[u8; 32], CxError> {
    let mut id = [0u8; 32];
    let mut hash = Sha2_256::new();
    let res = hash
        .update(b"Ed25519Stream key id")
        .and_then(|_| hash.update(&key.key[..]))
        .and_then(|_| hash.finalize(&mut id))
        .map_err(|_| CxError::GenericError);
    // Clear the key bytes left in the context.
    hash.reset();
    res.map(|_| id)
}

/// Size in bytes of an Ed25519 scalar (the curve's domain length).
const ED25519_SCALAR_LEN: usize = 32;

/// The nonce scalar `r = digest mod L` (RFC 8032 section 5.1.6, step 2), in a
/// BN of the curve's size. `digest` is the SHA-512 digest as a big-endian
/// integer, i.e. byte-reversed.
///
/// On a device, `cx_ecpoint_rnd_scalarmul_bn` rejects a scalar BN larger than
/// the curve with `CX_INVALID_PARAMETER_SIZE`: a 64-byte BN fails even when
/// it holds a value below the order. The digest is therefore reduced into a
/// 32-byte BN first, as the C SDK's `cx_eddsa_sign_hash` does. Speculos
/// accepts the larger BN, and its `cx_bn_nbytes` reports the value's length
/// rounded up to the `cx_bn_lock` word size (32 bytes), not the allocated
/// size, so off-device a test can only check that the value is reduced.
fn nonce_scalar(digest: &[u8; 64]) -> Result<Bn, CxError> {
    let h = Bn::alloc_init_size(64, digest)?;
    let mut order = Bn::alloc(ED25519_SCALAR_LEN)?;
    CurvesId::Ed25519.domain_parameter_bn(CurveDomainParam::Order, &mut order)?;
    let r = Bn::alloc(ED25519_SCALAR_LEN)?;
    r.reduce(&h, &order)?;
    Ok(r)
}

impl Ed25519Stream {
    /// Start a new signature with `key`, discarding any previous state.
    pub fn init(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        self.wipe();
        self.key_id = key_id(key)?;
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
        self.msg_hash.reset();
        self.phase = SignPhase::NoncePass;
        Ok(())
    }

    /// Clear all intermediate state, the nonce in particular, and require a new
    /// [`init`](Self::init).
    fn wipe(&mut self) {
        // `reset` clears the whole context on the C side, out of the
        // optimizer's reach.
        self.hash.reset();
        self.msg_hash.reset();
        self.key_id.zeroize();
        self.signature.zeroize();
        self.big_r.zeroize();
        self.msg_digest.zeroize();
        self.phase = SignPhase::Uninitialized;
    }

    fn compute_r(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        // Compute R (see https://datatracker.ietf.org/doc/html/rfc8032#section-5.1.6, step 3)
        self.hash
            .finalize(&mut self.signature)
            .map_err(|_| CxError::GenericError)?;
        self.signature.reverse();

        // Every BN and point below is destroyed when dropped, on error paths
        // too. The last one dropped releases the BN lock, which erases the BN
        // memory: public_key() and cx_edwards_compress_point_no_throw take the
        // lock themselves.
        let sign = {
            let r = nonce_scalar(&self.signature)?;
            // Get the generator for Ed25519's curve
            let mut ed_p = EcPoint::new(CurvesId::Ed25519)?;
            CurvesId::Ed25519.generator_bn(&mut ed_p)?;
            // Multiply r by generator, store in ed_p. r is secret: use the
            // randomized multiplication, as the C SDK EdDSA signer does.
            ed_p.rnd_scalarmul_bn(r)?;
            // and copy/compress it to big_r
            ed_p.compress(&mut self.big_r)?
        };

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
        // Every BN below is destroyed when dropped, on error paths too, and the
        // last one dropped releases the BN lock, which erases the BN memory.

        // Get the group order
        let mut ed25519_order = Bn::alloc(64)?;
        CurvesId::Ed25519.domain_parameter_bn(CurveDomainParam::Order, &mut ed25519_order)?;

        let h_a = {
            let mut h_scalar = Secret::<64>::new();
            self.hash
                .finalize(h_scalar.as_mut())
                .map_err(|_| CxError::GenericError)?;

            h_scalar.0.reverse();

            // Make k into a BN
            let h_scalar_bn = Bn::alloc_init_size(64, &h_scalar.0)?;

            // Generate the hashed private key
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

            // Load key into bn
            let key_bn = Bn::alloc_init_size(64, key_slice)?;

            // The hash context still holds the key and its digest; temp
            // zeroizes on drop.
            self.hash.reset();

            // multiply h_scalar_bn by key_bn
            let rv = Bn::alloc(64)?;
            rv.mod_mul(&key_bn, &h_scalar_bn, &ed25519_order)?;
            rv
        };

        let r = Bn::alloc_init_size(64, &self.signature)?;

        // finally, compute s:
        let s = Bn::alloc(64)?;
        s.mod_add(&h_a, &r, &ed25519_order)?;

        // Spooky sub 0 to avoid Nano S+ bug
        r.set_u32(0)?;

        s.mod_sub(&s, &r, &ed25519_order)?;
        // and copy s back to normal memory to return.
        s.export(&mut self.signature[..32])?;

        self.signature[..32].reverse();

        // Copy R[32] and S[32] into signature[64]
        self.signature.copy_within(0..32, 32);
        self.signature[0..32].copy_from_slice(&self.big_r);

        Ok(())
    }

    /// End the current message pass.
    ///
    /// `key` must be the key given to [`init`](Self::init). The first call
    /// derives the nonce from the first pass. The second one checks that the
    /// second pass carried the same message, then computes the signature. On
    /// any error, including a different key or a mismatch between both passes,
    /// all state is wiped and no signature is produced.
    ///
    /// Returns [`CxError::Locked`] while the BN lock is held, by a
    /// [`Bn`](crate::bn::Bn), an [`EcPoint`], a
    /// [`MontCtx`](crate::bn::MontCtx) or a [`BnLock`](crate::bn::BnLock):
    /// the signature works in BN memory that is only erased once the lock is
    /// released, so it must not be computed while another object keeps it.
    pub fn sign_finalize(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        if !matches!(self.phase, SignPhase::NoncePass | SignPhase::ChallengePass) {
            return Err(CxError::InvalidParameter);
        }
        if bn_is_held() {
            self.wipe();
            return Err(CxError::Locked);
        }
        let res = self.check_key(key).and_then(|_| match self.phase {
            SignPhase::NoncePass => self.end_nonce_pass(key),
            _ => self.end_challenge_pass(key),
        });
        if res.is_err() {
            self.wipe();
        }
        res
    }

    fn check_key(&self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        if ct_eq(&key_id(key)?, &self.key_id) {
            Ok(())
        } else {
            Err(CxError::InvalidParameterValue)
        }
    }

    fn end_nonce_pass(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        self.msg_hash
            .finalize(&mut self.msg_digest)
            .map_err(|_| CxError::GenericError)?;
        self.msg_hash.reset();
        self.compute_r(key)?;
        self.phase = SignPhase::ChallengePass;
        Ok(())
    }

    fn end_challenge_pass(&mut self, key: &ECPrivateKey<32, 'E'>) -> Result<(), CxError> {
        let mut digest = [0u8; 64];
        self.msg_hash
            .finalize(&mut digest)
            .map_err(|_| CxError::GenericError)?;
        if !ct_eq(&digest, &self.msg_digest) {
            return Err(CxError::InvalidParameterValue);
        }
        self.compute_s(key)?;
        self.msg_digest.zeroize();
        self.phase = SignPhase::Complete;
        Ok(())
    }

    /// Feed the next chunk of the message, in either pass.
    pub fn sign_update(&mut self, msg: &[u8]) -> Result<(), CxError> {
        if !matches!(self.phase, SignPhase::NoncePass | SignPhase::ChallengePass) {
            return Err(CxError::InvalidParameter);
        }
        let res = self
            .hash
            .update(msg)
            .and_then(|_| self.msg_hash.update(msg))
            .map_err(|_| CxError::GenericError);
        if res.is_err() {
            self.wipe();
        }
        res
    }

    /// The signature, once both message passes are complete.
    pub fn signature(&self) -> Result<&[u8; 64], CxError> {
        match self.phase {
            SignPhase::Complete => Ok(&self.signature),
            _ => Err(CxError::InvalidParameter),
        }
    }
}

/// Edwards Curves-specific implementation
impl<const N: usize> ECPrivateKey<N, 'E'> {
    /// Size of an Edwards curve signature relative to the private key size
    pub const EP: usize = 2 * N;

    pub fn sign(&self, hash: &[u8]) -> Result<(<Self as SignatureSize>::Signature, u32), CxError>
    where
        Self: SignatureSize,
    {
        self.check_layout()?;
        let mut sig = Self::empty_signature();
        let buf = sig.as_mut();
        let sig_len = buf.len();
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
                buf.as_mut_ptr(),
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
    /// Verifies the first `signature.1` bytes of `signature.0` against `hash`.
    ///
    /// Returns `false` if `signature.1` exceeds the length of `signature.0`,
    /// or if the key's curve, buffer size and `keylength` do not match.
    pub fn verify(&self, signature: (&[u8], u32), hash: &[u8], hash_id: u8) -> bool {
        if signature.1 as usize > signature.0.len() || !self.fits_layout() {
            return false;
        }
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
    /// A tuple of (ask, nsk, ovk, dk) as `Secret<32>` values, or a `CxError` if the syscall fails
    pub fn zip32_sapling_derive(
        path: &[u32],
        cc: Option<&mut ChainCode>,
        seed: Option<&[u8]>,
    ) -> Result<(Secret<32>, Secret<32>, Secret<32>, Secret<32>), CxError> {
        let mut tmp = Secret::<128>::new();
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
                HDKeyDeriveMode::Zip32Sapling as u8,
                CurvesId::JubJub as cx_curve_t,
                path.as_ptr(),
                path.len(),
                tmp.as_mut().as_mut_ptr(),
                128,
                cc_ptr,
                cc_len,
                seed_ptr,
                seed_len,
            )
        };
        if err != CX_OK {
            return Err(err.into());
        }
        let mut ask = Secret::<32>::new();
        let mut nsk = Secret::<32>::new();
        let mut ovk = Secret::<32>::new();
        let mut dk = Secret::<32>::new();
        ask.0.copy_from_slice(&tmp.0[..32]);
        nsk.0.copy_from_slice(&tmp.0[32..64]);
        ovk.0.copy_from_slice(&tmp.0[64..96]);
        dk.0.copy_from_slice(&tmp.0[96..128]);
        Ok((ask, nsk, ovk, dk))
    }
}

impl SeedDerive for JubJub {
    type Target = (Secret<32>, Secret<32>, Secret<32>, Secret<32>);
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut cc: ChainCode = Default::default();
        let keys = Self::zip32_sapling_derive(path, Some(&mut cc), None)
            .expect("zip32_sapling_derive failed");
        (keys, Some(cc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::ecc::make_bip32_path;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const PATH0: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/0");

    fn display_error_code(e: CxError) {
        let ec = crate::testing::to_hex(e.into());
        crate::log::info!(
            "Error code: \x1b[1;33m{}\x1b[0m",
            core::str::from_utf8(&ec).unwrap()
        );
    }

    const TEST_HASH: &[u8; 13] = b"test_message1";

    #[test]
    fn eddsa_ed25519() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
    }

    #[test]
    fn eddsa_verify_rejects_length_beyond_signature() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
        assert_eq!(pk.verify((&s.0, s.1 + 1), TEST_HASH, CX_SHA512), false);
        assert_eq!(pk.verify((&s.0, u32::MAX), TEST_HASH, CX_SHA512), false);
    }

    #[test]
    fn eddsa_ed25519_slip10() {
        let path: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0'/1'");
        let sk = Ed25519::derive_from_path_slip10(&path);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
    }

    const MSG1: &[u8] = b"test_message1";
    const MSG2: &[u8] = b"test_message2";
    const MSG3: &[u8] = b"test_message3";

    fn stream_pass(
        streamer: &mut Ed25519Stream,
        sk: &ECPrivateKey<32, 'E'>,
        chunks: &[&[u8]],
    ) -> Result<(), CxError> {
        for chunk in chunks {
            streamer.sign_update(chunk)?;
        }
        streamer.sign_finalize(sk)
    }

    #[test]
    fn eddsa_ed25519_stream_sign() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let pk = sk.public_key().map_err(display_error_code)?;

        let mut streamer = Ed25519Stream::default();
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1, MSG2, MSG3]).map_err(display_error_code)?;
        // Chunking may differ between passes, only the bytes matter.
        stream_pass(&mut streamer, &sk, &[MSG1, b"test_message2test_message3"])
            .map_err(display_error_code)?;
        let sig = *streamer.signature().map_err(display_error_code)?;

        let mut concatenated: [u8; 39] = [0; 39];
        // Copy the contents of each array into the concatenated array
        concatenated[0..13].copy_from_slice(MSG1);
        concatenated[13..26].copy_from_slice(MSG2);
        concatenated[26..39].copy_from_slice(MSG3);
        assert_eq!(
            pk.verify((&sig, sig.len() as u32), &concatenated, CX_SHA512),
            true
        );

        // PureEdDSA is deterministic: the stream must match the one-shot signature.
        let (one_shot, _) = sk.sign(&concatenated).map_err(display_error_code)?;
        assert_eq!(sig == one_shot, true);

        // A new signature after `init` on the same stream is the same again.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[&concatenated]).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[&concatenated]).map_err(display_error_code)?;
        assert_eq!(
            *streamer.signature().map_err(display_error_code)? == sig,
            true
        );
    }

    // On a device, cx_ecpoint_rnd_scalarmul_bn rejects a scalar BN larger
    // than the curve; Speculos accepts it, so the signing tests above pass
    // either way. Check the nonce scalar directly: it must be reduced below
    // the order and fit the curve's size. (Speculos reports a BN's value
    // length rounded up to the 32-byte lock word size, so any reduced value
    // reports 32 there, and only a device can also see an oversized
    // allocation.)
    #[test]
    fn eddsa_ed25519_stream_nonce_scalar_has_curve_size() {
        let curve_len = CurvesId::Ed25519.size_bytes();
        let mut order = Bn::alloc(curve_len).map_err(display_error_code)?;
        CurvesId::Ed25519
            .domain_parameter_bn(CurveDomainParam::Order, &mut order)
            .map_err(display_error_code)?;
        let mut order_bytes = [0u8; 32];
        order.export(&mut order_bytes).map_err(display_error_code)?;

        // A digest below the order is its own residue.
        let mut digest = [0u8; 64];
        digest[63] = 7;
        let r = nonce_scalar(&digest).map_err(display_error_code)?;
        assert_eq!(r.nbytes().map_err(display_error_code)?, curve_len);
        assert_eq!(r.get_u32().map_err(display_error_code)?, 7);

        // order + 5 reduces to 5 (the order's last byte is 0xed: no carry).
        let mut digest = [0u8; 64];
        digest[32..].copy_from_slice(&order_bytes);
        digest[63] = order_bytes[31] + 5;
        let r = nonce_scalar(&digest).map_err(display_error_code)?;
        assert_eq!(r.nbytes().map_err(display_error_code)?, curve_len);
        assert_eq!(r.get_u32().map_err(display_error_code)?, 5);

        // The largest digest reduces below the order.
        let r = nonce_scalar(&[0xff; 64]).map_err(display_error_code)?;
        assert_eq!(r.nbytes().map_err(display_error_code)?, curve_len);
        assert_eq!(
            r.cmp_bn(&order).map_err(display_error_code)?,
            core::cmp::Ordering::Less
        );
    }

    #[test]
    fn eddsa_ed25519_stream_rejects_mismatched_passes() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let mut streamer = Ed25519Stream::default();

        // Different second pass.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1]).map_err(display_error_code)?;
        assert_eq!(stream_pass(&mut streamer, &sk, &[MSG2]).is_err(), true);
        assert_eq!(streamer.signature().is_err(), true);

        // Truncated second pass.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1, MSG2]).map_err(display_error_code)?;
        assert_eq!(stream_pass(&mut streamer, &sk, &[MSG1]).is_err(), true);
        assert_eq!(streamer.signature().is_err(), true);

        // The failed stream is wiped: nothing proceeds without a new `init`.
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);
        assert_eq!(streamer.sign_finalize(&sk).is_err(), true);
    }

    #[test]
    fn eddsa_ed25519_stream_rejects_other_key() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let path1: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/1");
        let other = Ed25519::derive_from_path(&path1);
        let mut streamer = Ed25519Stream::default();

        // Other key at the end of the first pass.
        streamer.init(&sk).map_err(display_error_code)?;
        assert_eq!(
            stream_pass(&mut streamer, &other, &[MSG1]),
            Err(CxError::InvalidParameterValue)
        );
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);

        // Other key at the end of the second pass.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1]).map_err(display_error_code)?;
        assert_eq!(
            stream_pass(&mut streamer, &other, &[MSG1]),
            Err(CxError::InvalidParameterValue)
        );
        assert_eq!(streamer.signature().is_err(), true);
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);
    }

    #[test]
    fn eddsa_ed25519_stream_rejects_out_of_order_calls() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let mut streamer = Ed25519Stream::default();

        // Nothing before `init`.
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);
        assert_eq!(streamer.sign_finalize(&sk).is_err(), true);
        assert_eq!(streamer.signature().is_err(), true);

        // No signature after the first pass only.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1]).map_err(display_error_code)?;
        assert_eq!(streamer.signature().is_err(), true);

        // Nothing more once complete, and the signature stays available.
        stream_pass(&mut streamer, &sk, &[MSG1]).map_err(display_error_code)?;
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);
        assert_eq!(streamer.sign_finalize(&sk).is_err(), true);
        assert_eq!(streamer.signature().is_ok(), true);
    }

    fn stream_sign(sk: &ECPrivateKey<32, 'E'>, msg: &[u8]) -> Result<[u8; 64], CxError> {
        let mut streamer = Ed25519Stream::default();
        streamer.init(sk)?;
        stream_pass(&mut streamer, sk, &[msg])?;
        stream_pass(&mut streamer, sk, &[msg])?;
        Ok(*streamer.signature()?)
    }

    #[test]
    fn eddsa_ed25519_stream_rejects_held_bn_lock() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let expected = stream_sign(&sk, MSG1).map_err(display_error_code)?;
        let mut streamer = Ed25519Stream::default();

        // Lock held through the first pass.
        streamer.init(&sk).map_err(display_error_code)?;
        {
            let _outer = Bn::alloc(32).map_err(display_error_code)?;
            assert_eq!(
                stream_pass(&mut streamer, &sk, &[MSG1]),
                Err(CxError::Locked)
            );
        }
        assert_eq!(streamer.sign_update(MSG1).is_err(), true);
        assert_eq!(bn_is_held(), false);

        // Lock taken between both passes.
        streamer.init(&sk).map_err(display_error_code)?;
        stream_pass(&mut streamer, &sk, &[MSG1]).map_err(display_error_code)?;
        {
            let _outer = EcPoint::new(CurvesId::Ed25519).map_err(display_error_code)?;
            assert_eq!(
                stream_pass(&mut streamer, &sk, &[MSG1]),
                Err(CxError::Locked)
            );
        }
        assert_eq!(streamer.signature().is_err(), true);
        assert_eq!(bn_is_held(), false);

        // Signing works again, with the same signature, once the lock is released.
        assert_eq!(
            stream_sign(&sk, MSG1).map_err(display_error_code)? == expected,
            true
        );
    }

    #[test]
    fn eddsa_ed25519_stream_releases_bn_memory() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let expected = stream_sign(&sk, MSG1).map_err(display_error_code)?;
        // Far more signatures than the BN memory could hold if any of them
        // left its handles allocated.
        for _ in 0..32 {
            let sig = stream_sign(&sk, MSG1).map_err(display_error_code)?;
            assert_eq!(sig == expected, true);
            assert_eq!(bn_is_held(), false);
            assert_eq!(crate::bn::BnLock::is_locked_bool(), false);
        }
    }
}
