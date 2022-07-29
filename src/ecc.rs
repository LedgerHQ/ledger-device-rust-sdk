use crate::bindings::*;

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
            CX_LOCKED => CxError::Carry,
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

extern "C" {
    pub fn cx_ecfp_generate_pair_no_throw(
        curve: cx_curve_t,
        pubkey: *mut ECCKeyRaw,
        privkey: *const ECCKeyRaw, // This binding is forced to 'const' assuming `keepprivate` will always be set to `true`
        keepprivate: bool,
    ) -> cx_err_t;
    pub fn cx_ecdsa_sign_no_throw(
        pvkey: *const ECCKeyRaw,
        mode: u32,
        hashID: cx_md_t,
        hash: *const u8,
        hash_len: size_t,
        sig: *mut u8,
        sig_len: *mut size_t,
        info: *mut u32,
    ) -> cx_err_t;
    pub fn cx_ecdsa_verify_no_throw(
        pukey: *const ECCKeyRaw,
        hash: *const u8,
        hash_len: size_t,
        sig: *const u8,
        sig_len: size_t,
    ) -> bool;
    pub fn cx_eddsa_sign_no_throw(
        pvkey: *const ECCKeyRaw,
        hashID: cx_md_t,
        hash: *const u8,
        hash_len: size_t,
        sig: *mut u8,
        sig_len: size_t,
    ) -> cx_err_t;
    pub fn cx_eddsa_verify_no_throw(
        pukey: *const ECCKeyRaw,
        hashID: cx_md_t,
        hash: *const u8,
        hash_len: size_t,
        sig: *const u8,
        sig_len: size_t,
    ) -> bool;
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
pub struct ECPublicKey<const S: usize, const TY: char> {
    curve: CurvesId,
    keylength: usize,
    pubkey: [u8; S],
}

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
        self.key.fill_with(|| 0);
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

    /// Fill the key buffer `ECPrivateKey<_,_>.key` with bytes
    /// derived from the seed through BIP32 using the curve secp256k1
    pub fn bip32_fill(mut self, path: &[u32]) -> ECPrivateKey<N, TY> {
        bip32_derive(CurvesId::Secp256k1, path, &mut self.key);
        self
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
                &mut pubkey as *mut ECPublicKey<{ Self::P }, TY> as *mut ECCKeyRaw,
                self as *const ECPrivateKey<N, TY> as *const ECCKeyRaw,
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
    ) -> Result<([u8; Self::S], u32), CxError> {
        let mut sig = [0u8; Self::S];
        let mut sig_len = Self::S as u32;
        let mut info = 0;
        let len = unsafe {
            cx_ecdsa_sign_no_throw(
                self as *const ECPrivateKey<N, 'W'> as *const ECCKeyRaw,
                mode,
                hash_id,
                hash.as_ptr(),
                hash.len() as u32,
                sig.as_mut_ptr(),
                &mut sig_len,
                &mut info,
            )
        };
        if len != CX_OK {
            Err(len.into())
        } else {
            Ok((sig, sig_len))
        }
    }

    /// Sign a message/hash using ECDSA with RFC6979, which provides a deterministic nonce rather than
    /// a random one. This nonce is computed using a hash function, hence this function uses an
    /// additional parameter `hash_id` that specifies which one it should use.
    pub fn deterministic_sign(&self, hash: &[u8]) -> Result<([u8; Self::S], u32), CxError> {
        let hash_id = match self.keylength {
            x if x <= 32 => CX_SHA256,
            x if x <= 48 => CX_SHA384,
            x if x <= 64 => CX_SHA512,
            _ => CX_BLAKE2B,
        };
        self.ecdsa_sign(hash, hash_id, (CX_RND_RFC6979 | CX_LAST) as u32)
    }

    /// Sign a message/hash using ECDSA in its original form
    pub fn sign(&self, hash: &[u8]) -> Result<([u8; Self::S], u32), CxError> {
        self.ecdsa_sign(hash, 0, (CX_RND_TRNG | CX_LAST) as u32)
    }
}

/// Edwards Curves-specific implementation
impl<const N: usize> ECPrivateKey<N, 'E'> {
    /// Size of an Edwards curve public key relative to the private key size
    pub const EP: usize = 2 * N;

    pub fn sign(&self, hash: &[u8]) -> Result<([u8; Self::EP], u32), CxError> {
        let mut sig = [0u8; Self::EP];
        let sig_len = Self::EP as u32;
        let hash_id = match self.keylength {
            x if x <= 32 => CX_SHA512,
            _ => CX_BLAKE2B,
        };
        let len = unsafe {
            cx_eddsa_sign_no_throw(
                self as *const ECPrivateKey<N, 'E'> as *const ECCKeyRaw,
                hash_id,
                hash.as_ptr(),
                hash.len() as u32,
                sig.as_mut_ptr(),
                sig_len,
            )
        };
        if len != CX_OK {
            Err(len.into())
        } else {
            Ok((sig, sig_len))
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

/// Specific signature verification for Weierstrass curves, which
/// all use ECDSA.
impl<const P: usize> ECPublicKey<P, 'W'> {
    pub fn verify(&self, signature: (&[u8], u32), hash: &[u8]) -> bool {
        unsafe {
            cx_ecdsa_verify_no_throw(
                self as *const ECPublicKey<P, 'W'> as *const ECCKeyRaw,
                hash.as_ptr(),
                hash.len() as u32,
                signature.0.as_ptr(),
                signature.1,
            )
        }
    }
}

/// Specific signature verification for Edwards curves, which all use EdDSA
impl<const P: usize> ECPublicKey<P, 'E'> {
    pub fn verify(&self, signature: (&[u8], u32), hash: &[u8], hash_id: u8) -> bool {
        unsafe {
            cx_eddsa_verify_no_throw(
                self as *const ECPublicKey<P, 'E'> as *const ECCKeyRaw,
                hash_id,
                hash.as_ptr(),
                hash.len() as u32,
                signature.0.as_ptr(),
                signature.1,
            )
        }
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

            pub fn from_bip32(path: &[u32]) -> ECPrivateKey<$size, $curvetype> {
                ECPrivateKey::<$size, $curvetype>::new(CurvesId::$typename).bip32_fill(path)
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

/// Wrapper for 'os_perso_derive_node_bip32'
pub fn bip32_derive(curve: CurvesId, path: &[u32], key: &mut [u8]) {
    unsafe {
        os_perso_derive_node_bip32(
            curve as u8,
            path.as_ptr(),
            path.len() as u32,
            key.as_mut_ptr(),
            core::ptr::null_mut(),
        )
    };
}

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
                        j += 1;
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
                b'/' => state = Bip32ParserState::FirstDigit,
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
        panic!("path is too short");
    }

    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::TestType;
    use testmacro::test_item as test;

    const PATH: [u32; 5] = make_bip32_path(b"m/44'/535348'/0'/0/0");

    fn display_error_code(e: CxError) {
        let ec = crate::to_hex(e.into());
        crate::debug_print("\tError code: \x1b[1;33m");
        crate::debug_print(core::str::from_utf8(&ec).unwrap());
        crate::debug_print("\x1b[0m\n");
    }

    const TEST_HASH: &[u8; 13] = b"test_message1";

    #[test]
    fn ecdsa_secp256k1() {
        let sk = Secp256k1::from_bip32(&PATH);
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
        let sk = Secp256r1::from_bip32(&PATH);
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
        let sk = Secp384r1::from_bip32(&PATH);
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    // #[test]
    // fn ecdsa_secp521r1() {
    //     let sk = Secp521r1::from_bip32(&PATH);
    //     let s = sk.deterministic_sign(TEST_HASH).map_err(display_error_code)?;
    //     let pk = sk.public_key().map_err(display_error_code)?;
    //     assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    // }

    #[test]
    fn ecdsa_brainpool256r1() {
        let sk = BrainpoolP256R1::from_bip32(&PATH);
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
        let sk = BrainpoolP320R1::from_bip32(&PATH);
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
        let sk = BrainpoolP384R1::from_bip32(&PATH);
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
        let sk = BrainpoolP512R1::from_bip32(&PATH);
        let s = sk
            .deterministic_sign(TEST_HASH)
            .map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    }

    // #[test]
    // fn ecdsa_frp256v1() {
    //     let sk = FRP256v1::from_bip32(&PATH);
    //     let s = sk.deterministic_sign(TEST_HASH, CX_SHA256).map_err(display_error_code)?;
    //     let pk = sk.public_key().map_err(display_error_code)?;
    //     assert_eq!(pk.verify((&s.0, s.1), TEST_HASH), true);
    // }

    #[test]
    fn ecdsa_stark256() {
        let sk = Stark256::from_bip32(&PATH);
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
        let sk = Ed25519::from_bip32(&PATH);
        let s = sk.sign(TEST_HASH).map_err(display_error_code)?;
        let pk = sk.public_key().map_err(display_error_code)?;
        assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHA512), true);
    }

    // #[test]
    // fn ecdsa_ed448() {
    //     let sk = Ed448::from_bip32(&PATH);
    //     let s = sk.sign(TEST_HASH, CX_SHAKE256).map_err(display_error_code)?;
    //     let pk = sk.public_key().map_err(display_error_code)?;
    //     assert_eq!(pk.verify((&s.0, s.1), TEST_HASH, CX_SHAKE256), true);
    // }

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
    }
}
