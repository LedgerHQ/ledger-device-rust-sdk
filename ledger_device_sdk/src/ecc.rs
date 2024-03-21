use ledger_secure_sdk_sys::*;
use zeroize::Zeroize;

use crate::hash::{sha2::Sha2_512, HashInit};

mod stark;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum CurvesId {
    Secp256k1 = CX_CURVE_SECP256K1,
    Secp256r1 = CX_CURVE_SECP256R1,
    Secp384r1 = CX_CURVE_SECP384R1,
    BrainpoolP256T1 = CX_CURVE_BrainPoolP256T1,
    BrainpoolP256R1 = CX_CURVE_BrainPoolP256R1,
    BrainpoolP320R1 = CX_CURVE_BrainPoolP320R1,
    BrainpoolP320T1 = CX_CURVE_BrainPoolP320T1,
    BrainpoolP384T1 = CX_CURVE_BrainPoolP384T1,
    BrainpoolP384R1 = CX_CURVE_BrainPoolP384R1,
    BrainpoolP512T1 = CX_CURVE_BrainPoolP512T1,
    BrainpoolP512R1 = CX_CURVE_BrainPoolP512R1,
    Bls12381G1 = CX_CURVE_BLS12_381_G1, // unsupported in speculos
    FRP256v1 = CX_CURVE_FRP256V1,       // unsupported in speculos
    Stark256 = CX_CURVE_Stark256,
    Ed25519 = CX_CURVE_Ed25519,
    Ed448 = CX_CURVE_Ed448,           // unsupported in speculos
    Curve25519 = CX_CURVE_Curve25519, // unsupported in speculos
    Curve448 = CX_CURVE_Curve448,     // unsupported in speculos
    Secp521r1 = CX_CURVE_SECP521R1,   // unsupported in speculos
    Invalid,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CxError {
    Carry,
    Locked,
    Unlocked,
    NotLocked,
    NotUnlocked,
    InternalError,
    InvalidParameterSize,
    InvalidParameterValue,
    InvalidParameter,
    NotInvertible,
    Overflow,
    MemoryFull,
    NoResidue,
    PointAtInfinity,
    InvalidPoint,
    InvalidCurve,
    GenericError,
}

impl From<u32> for CxError {
    fn from(x: u32) -> CxError {
        match x {
            CX_CARRY => CxError::Carry,
            CX_LOCKED => CxError::Locked,
            CX_UNLOCKED => CxError::Unlocked,
            CX_NOT_LOCKED => CxError::NotLocked,
            CX_NOT_UNLOCKED => CxError::NotUnlocked,
            CX_INTERNAL_ERROR => CxError::InternalError,
            CX_INVALID_PARAMETER_SIZE => CxError::InvalidParameterSize,
            CX_INVALID_PARAMETER_VALUE => CxError::InvalidParameterValue,
            CX_INVALID_PARAMETER => CxError::InvalidParameter,
            CX_NOT_INVERTIBLE => CxError::NotInvertible,
            CX_OVERFLOW => CxError::Overflow,
            CX_MEMORY_FULL => CxError::MemoryFull,
            CX_NO_RESIDUE => CxError::NoResidue,
            CX_EC_INFINITE_POINT => CxError::PointAtInfinity,
            CX_EC_INVALID_POINT => CxError::InvalidPoint,
            CX_EC_INVALID_CURVE => CxError::InvalidCurve,
            _ => CxError::GenericError,
        }
    }
}

impl From<CxError> for u32 {
    fn from(e: CxError) -> u32 {
        e as u32
    }
}

/// This structure serves the sole purpose of being cast into
/// from `ECPrivateKey` or `ECPublicKey` when calling bindings
/// to elliptic curve cryptographic bindings
/// It is not intended to be constructed directly.
#[repr(C)]
pub struct ECCKeyRaw {
    curve: CurvesId,
    keylength: usize,
    key: [u8; 160],
}

/// This structure matches the lower-level C `cx_ecfp_private_key_t` type
/// so it can be passed to ecc-related syscalls as itself, rather than
/// making an entirely different structure that would need to allocate
/// exactly this type before calling C functions.
/// It has two const parameters `N` and `TY` which represent the length
/// in bytes of the buffer holding the key, and the type of curve among
/// 'W' (Weierstrass), 'M' (Montgomery) and 'E' (Edwards).
/// This latter const parameter allows routing the signing function to
/// the correct call. ECDSA for example cannot be used for Edwards'
/// curves, we have to use EdDSA instead.
#[repr(C)]
pub struct ECPrivateKey<const N: usize, const TY: char> {
    curve: CurvesId,
    keylength: usize,
    key: [u8; N],
}

/// Represents a public key, its layout matching the C SDK's `cx_ecfp_public_key_t` type.
///
/// An ECPublicKey can only be created by calling `ECPrivateKey::public_key()`
/// It is parameterized by the length of the buffer as well and, as this
/// buffer has to be exactly twice the size of the private key + 1 byte,
/// it is constructed from the private key using its const length parameter.
///
/// # Examples
///
/// ```
/// let sk = ECPrivateKey::<32, 'W'>::new();
/// let public_key = sk.public_key();
/// ```
#[repr(C)]
#[derive(Clone)]
pub struct ECPublicKey<const S: usize, const TY: char> {
    pub curve: CurvesId,
    pub keylength: usize,
    pub pubkey: [u8; S],
}

/// Create a default (empty/invalid) private key object
impl<const N: usize, const TY: char> Default for ECPrivateKey<N, TY> {
    fn default() -> ECPrivateKey<N, TY> {
        ECPrivateKey {
            curve: CurvesId::Invalid,
            keylength: N,
            key: [0u8; N],
        }
    }
}

/// Cleanup keys from memory when dropping this structure.
impl<const N: usize, const TY: char> Drop for ECPrivateKey<N, TY> {
    #[inline(never)]
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

/// This is the most generic implementation for ECPrivateKey.
/// It provides a way to create a new private key structure
/// by specifying its length (const parameter `N`), and its
/// type (const parameter `TY`).
/// It defines the needed lengths for holding a complete
/// signature (`const Y`) and a public key (`const S`)
impl<const N: usize, const TY: char> ECPrivateKey<N, TY> {
    /// Size of the public key relative to the private key's size
    pub const P: usize = 2 * N + 1;

    /// Size of the encoded signature relative to the private key's size
    pub const S: usize = 6 + 2 * (N + 1);

    /// Create a new private key from a curve identifier and with a given
    /// length and type. The preferred way to create a key is by using
    /// a curve type directly like `Secp256k1::new()`
    pub fn new(curve: CurvesId) -> ECPrivateKey<N, TY> {
        ECPrivateKey {
            curve,
            keylength: N,
            key: [0u8; N],
        }
    }

    /// Retrieve the public key corresponding to the private key (`self`)
    /// The size of the structure holding fits exactly that public key.
    ///
    /// # Const annotation
    ///
    /// The result of this function is an `ECPublicKey<{Self::P}, TY>`
    /// where `Self::P` is a constant computed at compile time which is
    /// (for Weierstrass curves at least) 2*N+1. The `TY` parameter
    /// is the curve type, which is the same as the private key.
    ///
    /// The `where [(); Self::P]:` clause can be surprising: it is some
    /// sort of an 'existential'  clause that is here to make sure that
    /// `Self::P` can actually be computed. An explanation can be found
    /// [here](https://blog.rust-lang.org/inside-rust/2021/09/06/Splitting-const-generics.html#featuregeneric_const_exprs)
    pub fn public_key(&self) -> Result<ECPublicKey<{ Self::P }, TY>, CxError>
    where
        [(); Self::P]:,
    {
        let mut pubkey = ECPublicKey::<{ Self::P }, TY>::new(self.curve);
        let err = unsafe {
            cx_ecfp_generate_pair_no_throw(
                self.curve as u8,
                // Safety: cast awfully dodgy but that's how it's done in the C SDK
                (&mut pubkey as *mut ECPublicKey<{ Self::P }, TY>).cast(), // as *mut cx_ecfp_256_public_key_s,
                self as *const ECPrivateKey<N, TY> as *mut cx_ecfp_256_private_key_s,
                true,
            )
        };
        if err != 0 {
            Err(err.into())
        } else {
            Ok(pubkey)
        }
    }
}

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

macro_rules! check_cx_ok {
    ($fn_call:expr) => {{
        let err = unsafe { $fn_call };
        if err != CX_OK {
            return Err(err.into());
        }
    }};
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

/// General implementation for a public key.
impl<const P: usize, const TY: char> ECPublicKey<P, TY> {
    /// Size of a signature relative to the public key's size
    pub const S: usize = 7 + P; // 6 + 2*(N+1)  and S = 2*N + 1

    /// Creates a new ECPublicKey structure from a curve identifier
    pub fn new(curve_id: CurvesId) -> ECPublicKey<P, TY> {
        ECPublicKey::<P, TY> {
            curve: curve_id,
            keylength: P,
            pubkey: [0u8; P],
        }
    }
}

/// Access public key value by reference
impl<const P: usize, const TY: char> AsRef<[u8]> for ECPublicKey<P, TY> {
    fn as_ref(&self) -> &[u8] {
        &self.pubkey
    }
}

/// Access public key value as array
impl<const P: usize, const TY: char> From<ECPublicKey<P, TY>> for [u8; P] {
    fn from(p: ECPublicKey<P, TY>) -> Self {
        p.pubkey
    }
}

/// Specific signature verification for Weierstrass curves, which
/// all use ECDSA.
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

/// Wrapper for 'os_perso_derive_node_bip32'
///
/// Checks consistency of curve choice and key length
/// in order to prevent the underlying syscall from throwing
pub fn bip32_derive(
    curve: CurvesId,
    path: &[u32],
    key: &mut [u8],
    cc: Option<&mut [u8]>,
) -> Result<(), CxError> {
    match curve {
        CurvesId::Secp256k1 | CurvesId::Secp256r1 | CurvesId::Ed25519 => {
            if key.len() < 64 {
                return Err(CxError::InvalidParameter);
            }
        }
        _ => return Err(CxError::InvalidParameter),
    }
    unsafe {
        match cc {
            Some(buf) => os_perso_derive_node_bip32(
                curve as u8,
                path.as_ptr(),
                path.len() as u32,
                key.as_mut_ptr(),
                buf.as_mut_ptr(),
            ),
            None => os_perso_derive_node_bip32(
                curve as u8,
                path.as_ptr(),
                path.len() as u32,
                key.as_mut_ptr(),
                core::ptr::null_mut(),
            ),
        }
    };
    Ok(())
}

/// Helper buffer that stores secrets that need to be cleared after use
pub struct Secret<const N: usize>([u8; N]);

impl<const N: usize> Default for Secret<N> {
    fn default() -> Secret<N> {
        Secret([0u8; N])
    }
}
impl<const N: usize> Secret<N> {
    pub fn new() -> Secret<N> {
        Secret::default()
    }
}

impl<const N: usize> AsRef<[u8]> for Secret<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for Secret<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> Drop for Secret<N> {
    #[inline(never)]
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[repr(C)]
#[derive(Default)]
pub struct ChainCode {
    pub value: [u8; 32],
}

/// Fill the key buffer `ECPrivateKey<_,_>.key` with bytes
/// derived from the seed through BIP32 or other standard
/// derivation scheme.
/// Since the underlying OS function only supports Secp256k1,
/// Secp256r1 and Ed25519, it is only implemented for these
/// curves.
pub trait SeedDerive {
    type Target;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>);
    fn derive_from_path(path: &[u32]) -> Self::Target {
        Self::derive_from(path).0
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

impl SeedDerive for Stark256 {
    type Target = ECPrivateKey<32, 'W'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut sk = Self::Target::new(CurvesId::Stark256);
        stark::eip2645_derive(path, &mut sk.key);
        (sk, None)
    }
}

/// This macro is used to easily generate zero-sized structures named after a Curve.
/// Each curve has a method `new()` that takes no arguments and returns the correctly
/// const-typed `ECPrivateKey`.
macro_rules! impl_curve {
    ($typename:ident, $size:expr, $curvetype:expr) => {
        pub struct $typename {}
        impl $typename {
            #[allow(clippy::new_ret_no_self)]
            pub fn new() -> ECPrivateKey<$size, $curvetype> {
                ECPrivateKey::<$size, $curvetype>::new(CurvesId::$typename)
            }

            pub fn from(keybytes: &[u8]) -> ECPrivateKey<$size, $curvetype> {
                let mut sk = $typename::new();
                sk.key.copy_from_slice(keybytes);
                sk
            }
        }
    };
}

impl_curve!(Secp256k1, 32, 'W');
impl_curve!(Secp256r1, 32, 'W');
impl_curve!(Secp384r1, 48, 'W');
// impl_curve!( Secp521r1, 66, 'W' );
impl_curve!(BrainpoolP256R1, 32, 'W');
impl_curve!(BrainpoolP256T1, 32, 'W');
impl_curve!(BrainpoolP320R1, 40, 'W');
impl_curve!(BrainpoolP320T1, 40, 'W');
impl_curve!(BrainpoolP384R1, 48, 'W');
impl_curve!(BrainpoolP384T1, 48, 'W');
impl_curve!(BrainpoolP512R1, 64, 'W');
impl_curve!(BrainpoolP512T1, 64, 'W');
impl_curve!(Stark256, 32, 'W');
impl_curve!(Ed25519, 32, 'E');
// impl_curve!( FRP256v1, 32, 'W' );
// impl_curve!( Ed448, 57, 'E' );

/// Creates at compile time an array from the ASCII values of a correctly
/// formatted derivation path.
///
/// Format expected: `b"m/44'/coin_type'/account'/change/address"`.
///
/// Warning: when calling this method, be sure the result is stored in a static
/// or const variable, to be sure evaluation is performed during compilation.
///
/// # Examples
///
/// ```
/// const path: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/0");
/// ```
///
/// # Panics
///
/// Panics if the parameter does not follow the correct format.
pub const fn make_bip32_path<const N: usize>(bytes: &[u8]) -> [u32; N] {
    // Describes current parser state
    #[derive(Copy, Clone)]
    enum Bip32ParserState {
        FirstDigit,
        Digit,
        Hardened,
    }

    let mut path = [0u32; N];

    // Verify path starts with "m/"
    if (bytes[0] != b'm') || (bytes[1] != b'/') {
        panic!("path must start with \"m/\"")
    }

    // Iterate over all characters (skipping m/ header)
    let mut i = 2; // parsed character index
    let mut j = 0; // constructed path number index
    let mut acc = 0; // constructed path number
    let mut state = Bip32ParserState::FirstDigit;

    while i < bytes.len() {
        let c = bytes[i];
        match state {
            // We are expecting a digit, after a /
            // This prevent having empty numbers, like //
            Bip32ParserState::FirstDigit => match c {
                b'0'..=b'9' => {
                    acc = (c - b'0') as u32;
                    path[j] = acc;
                    state = Bip32ParserState::Digit
                }
                _ => panic!("expected digit after '/'"),
            },
            // We are parsing digits for the current path token. We may also
            // find ' for hardening, or /.
            Bip32ParserState::Digit => {
                match c {
                    b'0'..=b'9' => {
                        acc = acc * 10 + (c - b'0') as u32;
                        path[j] = acc;
                    }
                    // Hardening
                    b'\'' => {
                        path[j] = acc + 0x80000000;
                        state = Bip32ParserState::Hardened
                    }
                    // Separator for next number
                    b'/' => {
                        path[j] = acc;
                        j += 1;
                        state = Bip32ParserState::FirstDigit
                    }
                    _ => panic!("unexpected character in path"),
                }
            }
            // Previous number has hardening. Next character must be a /
            // separator.
            Bip32ParserState::Hardened => match c {
                b'/' => {
                    j += 1;
                    state = Bip32ParserState::FirstDigit
                }
                _ => panic!("expected '/' character after hardening"),
            },
        }
        i += 1;
    }

    // Prevent last character from being /
    if let Bip32ParserState::FirstDigit = state {
        panic!("missing number in path")
    }

    // Assert we parsed the exact expected number of tokens in the path
    if j != N - 1 {
        panic!("wrong path length");
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
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

    const PATH0: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/0");
    const PATH1: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/1");

    fn display_error_code(e: CxError) {
        let ec = crate::testing::to_hex(e.into());
        crate::testing::debug_print("\tError code: \x1b[1;33m");
        crate::testing::debug_print(core::str::from_utf8(&ec).unwrap());
        crate::testing::debug_print("\x1b[0m\n");
    }

    const TEST_HASH: &[u8; 13] = b"test_message1";

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
    fn ecdsa_stark256() {
        let sk = Stark256::derive_from_path(&PATH0);
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    #[test]
    fn eddsa_ed25519() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
    }

    #[test]
    fn eddsa_ed25519_slip10() {
        let path: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0'/1'");
        let sk = Ed25519::derive_from_path_slip10(&path);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
    }

    #[test]
    fn eddsa_ed25519_stream_sign() {
        let sk = Ed25519::derive_from_path(&PATH0);
        let pk = sk.public_key().map_err(display_error_code)?;
        const MSG1: &[u8] = b"test_message1";
        const MSG2: &[u8] = b"test_message2";
        const MSG3: &[u8] = b"test_message3";

        let mut streamer = Ed25519Stream::default();
        streamer.init(&sk).map_err(display_error_code)?;

        streamer.sign_update(MSG1).unwrap();
        streamer.sign_update(MSG2).unwrap();
        streamer.sign_update(MSG3).unwrap();
        streamer.sign_finalize(&sk).unwrap();

        streamer.sign_update(MSG1).unwrap();
        streamer.sign_update(MSG2).unwrap();
        streamer.sign_update(MSG3).unwrap();
        streamer.sign_finalize(&sk).unwrap();

        let mut concatenated: [u8; 39] = [0; 39];
        // Copy the contents of each array into the concatenated array
        concatenated[0..13].copy_from_slice(MSG1);
        concatenated[13..26].copy_from_slice(MSG2);
        concatenated[26..39].copy_from_slice(MSG3);
        assert_eq!(
            pk.verify(
                (&streamer.signature, streamer.signature.len() as u32),
                &concatenated,
                CX_SHA512
            ),
            true
        );
    }

    #[test]
    fn test_make_bip32_path() {
        {
            const P: [u32; 1] = make_bip32_path(b"m/1234");
            assert_eq!(P, [1234u32]);
        }
        {
            const P: [u32; 2] = make_bip32_path(b"m/1234/5678");
            assert_eq!(P, [1234u32, 5678u32]);
        }
        {
            const P: [u32; 3] = make_bip32_path(b"m/1234/5678/91011");
            assert_eq!(P, [1234u32, 5678u32, 91011u32]);
        }
        {
            const P: [u32; 4] = make_bip32_path(b"m/1234/5678'/91011/0");
            assert_eq!(P, [1234u32, 5678u32 + 0x80000000u32, 91011u32, 0u32]);
        }
        {
            const P: [u32; 2] = make_bip32_path(b"m/1234/5678'");
            assert_eq!(P, [1234u32, 5678u32 + 0x80000000u32]);
        }
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
