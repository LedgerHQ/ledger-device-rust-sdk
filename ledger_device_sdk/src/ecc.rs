use ledger_secure_sdk_sys::*;
use zeroize::Zeroize;

pub mod edwards;
pub use edwards::*;
pub mod math;
pub use math::*;
pub mod montgomery;
pub use montgomery::*;
pub mod weierstrass;
pub use weierstrass::*;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum CurvesId {
    Secp256k1 = ledger_secure_sdk_sys::CX_CURVE_SECP256K1,
    Secp256r1 = ledger_secure_sdk_sys::CX_CURVE_SECP256R1,
    Secp384r1 = ledger_secure_sdk_sys::CX_CURVE_SECP384R1,
    BrainpoolP256T1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP256T1,
    BrainpoolP256R1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP256R1,
    BrainpoolP320R1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP320R1,
    BrainpoolP320T1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP320T1,
    BrainpoolP384T1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP384T1,
    BrainpoolP384R1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP384R1,
    BrainpoolP512T1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP512T1,
    BrainpoolP512R1 = ledger_secure_sdk_sys::CX_CURVE_BrainPoolP512R1,
    Bls12381G1 = ledger_secure_sdk_sys::CX_CURVE_BLS12_381_G1,
    FRP256v1 = ledger_secure_sdk_sys::CX_CURVE_FRP256V1, // unsupported in speculos
    Stark256 = ledger_secure_sdk_sys::CX_CURVE_Stark256,
    Bls12377G1 = ledger_secure_sdk_sys::CX_CURVE_BLS12_377_G1,
    Pallas = ledger_secure_sdk_sys::CX_CURVE_PALLAS,
    Vesta = ledger_secure_sdk_sys::CX_CURVE_VESTA,
    Ed25519 = ledger_secure_sdk_sys::CX_CURVE_Ed25519,
    Ed448 = ledger_secure_sdk_sys::CX_CURVE_Ed448,
    EdBLS12 = ledger_secure_sdk_sys::CX_CURVE_EdBLS12,
    JubJub = ledger_secure_sdk_sys::CX_CURVE_JUBJUB,
    Curve25519 = ledger_secure_sdk_sys::CX_CURVE_Curve25519,
    Curve448 = ledger_secure_sdk_sys::CX_CURVE_Curve448,
    Secp521r1 = ledger_secure_sdk_sys::CX_CURVE_SECP521R1,
    Invalid,
}

impl From<u8> for CurvesId {
    fn from(x: u8) -> CurvesId {
        match x {
            ledger_secure_sdk_sys::CX_CURVE_SECP256K1 => CurvesId::Secp256k1,
            ledger_secure_sdk_sys::CX_CURVE_SECP256R1 => CurvesId::Secp256r1,
            ledger_secure_sdk_sys::CX_CURVE_SECP384R1 => CurvesId::Secp384r1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP256T1 => CurvesId::BrainpoolP256T1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP256R1 => CurvesId::BrainpoolP256R1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP320R1 => CurvesId::BrainpoolP320R1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP320T1 => CurvesId::BrainpoolP320T1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP384T1 => CurvesId::BrainpoolP384T1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP384R1 => CurvesId::BrainpoolP384R1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP512T1 => CurvesId::BrainpoolP512T1,
            ledger_secure_sdk_sys::CX_CURVE_BrainPoolP512R1 => CurvesId::BrainpoolP512R1,
            ledger_secure_sdk_sys::CX_CURVE_BLS12_381_G1 => CurvesId::Bls12381G1,
            ledger_secure_sdk_sys::CX_CURVE_FRP256V1 => CurvesId::FRP256v1,
            ledger_secure_sdk_sys::CX_CURVE_Stark256 => CurvesId::Stark256,
            ledger_secure_sdk_sys::CX_CURVE_BLS12_377_G1 => CurvesId::Bls12377G1,
            ledger_secure_sdk_sys::CX_CURVE_PALLAS => CurvesId::Pallas,
            ledger_secure_sdk_sys::CX_CURVE_VESTA => CurvesId::Vesta,
            ledger_secure_sdk_sys::CX_CURVE_Ed25519 => CurvesId::Ed25519,
            ledger_secure_sdk_sys::CX_CURVE_Ed448 => CurvesId::Ed448,
            ledger_secure_sdk_sys::CX_CURVE_EdBLS12 => CurvesId::EdBLS12,
            ledger_secure_sdk_sys::CX_CURVE_JUBJUB => CurvesId::JubJub,
            ledger_secure_sdk_sys::CX_CURVE_Curve25519 => CurvesId::Curve25519,
            ledger_secure_sdk_sys::CX_CURVE_Curve448 => CurvesId::Curve448,
            ledger_secure_sdk_sys::CX_CURVE_SECP521R1 => CurvesId::Secp521r1,
            _ => CurvesId::Invalid,
        }
    }
}

impl From<CurvesId> for u8 {
    fn from(curve: CurvesId) -> u8 {
        curve as u8
    }
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
        [(); Self::P]: Sized,
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

#[macro_export]
macro_rules! check_cx_ok {
    ($fn_call:expr) => {{
        let err = unsafe { $fn_call };
        if err != CX_OK {
            return Err(err.into());
        }
    }};
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
#[derive(PartialEq)]
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

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum HDKeyDeriveMode {
    Bip32 = HDKEY_DERIVE_MODE_NORMAL,
    Slip10Ed25519 = HDKEY_DERIVE_MODE_ED25519_SLIP10,
    Slip21 = HDKEY_DERIVE_MODE_SLIP21,
    Zip32Sapling = HDKEY_DERIVE_MODE_ZIP32_SAPLING,
    Zip32Orchard = HDKEY_DERIVE_MODE_ZIP32_ORCHARD,
    Zip32Registered = HDKEY_DERIVE_MODE_ZIP32_REGISTERED,
}

/// This macro is used to easily generate zero-sized structures named after a Curve.
/// Each curve has a method `new()` that takes no arguments and returns the correctly
/// const-typed `ECPrivateKey`.
#[macro_export]
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

// impl_curve!(Secp256k1, 32, 'W');
// impl_curve!(Secp256r1, 32, 'W');
// impl_curve!(Secp384r1, 48, 'W');
// // impl_curve!( Secp521r1, 66, 'W' );
// impl_curve!(BrainpoolP256R1, 32, 'W');
// impl_curve!(BrainpoolP256T1, 32, 'W');
// impl_curve!(BrainpoolP320R1, 40, 'W');
// impl_curve!(BrainpoolP320T1, 40, 'W');
// impl_curve!(BrainpoolP384R1, 48, 'W');
// impl_curve!(BrainpoolP384T1, 48, 'W');
// impl_curve!(BrainpoolP512R1, 64, 'W');
// impl_curve!(BrainpoolP512T1, 64, 'W');
// impl_curve!(Stark256, 32, 'W');
// impl_curve!(Ed25519, 32, 'E');
// impl_curve!(JubJub, 32, 'E');
// impl_curve!(Pallas, 32, 'W');
// impl_curve!(Curve25519, 32, 'M');
// impl_curve!(Curve448, 56, 'M');

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
}
