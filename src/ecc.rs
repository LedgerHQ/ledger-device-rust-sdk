use crate::bindings::*;

#[repr(u8)]
pub enum CurvesId {
    Secp256k1 = CX_CURVE_SECP256K1,
}

/// Wrapper for 'os_perso_derive_node_bip32'
pub fn bip32_derive(curve: CurvesId, path: &[u32], key: &mut [u8]) {
    unsafe { os_perso_derive_node_bip32( curve as u8,
                                     path.as_ptr(), 
                                     path.len() as u32,
                                     key.as_mut_ptr(),
                                     core::ptr::null_mut() ) };
}

/// Wrapper for 'cx_ecfp_init_private_key'
pub fn ec_init_key(curve: CurvesId, raw_key: &[u8]) -> cx_ecfp_private_key_t {
    let mut ec_k = cx_ecfp_private_key_t::default();
    unsafe { cx_ecfp_init_private_key(curve as u8, 
        raw_key.as_ptr(), 
        raw_key.len() as u32, 
        &mut ec_k as *mut cx_ecfp_private_key_t) };
    ec_k
}

/// Wrapper for 'cx_ecfp_generate_pair'
pub fn ec_get_pubkey(curve: CurvesId, privkey: &mut cx_ecfp_private_key_t) -> cx_ecfp_public_key_t {
    let mut ec_pubkey = cx_ecfp_public_key_t::default();
    unsafe { 
        cx_ecfp_generate_pair(
            curve as u8, 
            &mut ec_pubkey as *mut cx_ecfp_public_key_t,
            privkey as *mut cx_ecfp_private_key_t, 
            1);
    }
    ec_pubkey
}

pub type DEREncodedECDSASignature = [u8; 73];
/// Wrapper for 'cx_ecdsa_sign'
pub fn ecdsa_sign(pvkey: &cx_ecfp_private_key_t, mode: i32, hash_id: u8, hash: &[u8]) -> Result<(DEREncodedECDSASignature,i32), ()> {
    let mut sig = [0u8; 73];
    let mut info = 0;
    let len = unsafe {
        cx_ecdsa_sign(  pvkey, 
                        mode,
                        hash_id,
                        hash.as_ptr(),
                        hash.len() as u32,
                        sig.as_mut_ptr(), 
                        sig.len() as u32, 
                        &mut info)
    };
    if len == 0 {
        Err(())
    } else {
        Ok((sig,len))
    } 
}

/// Wrapper for 'cx_ecdsa_verify'
pub fn ecdsa_verify(pubkey: &cx_ecfp_public_key_t, sig: &[u8], mode: i32,
hash_id: u8, hash: &[u8]) -> bool {
    let status = unsafe {
        cx_ecdsa_verify(
           pubkey as *const cx_ecfp_public_key_t,
           mode,
           hash_id,
           hash.as_ptr(),
           hash.len() as u32,
           sig.as_ptr(),
           sig.len() as u32)
    };
    status == 1
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
    enum BIP32ParserState {
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
    let mut state = BIP32ParserState::FirstDigit;

    while i < bytes.len() {
        let c = bytes[i];
        match state {
            // We are expecting a digit, after a /
            // This prevent having empty numbers, like //
            BIP32ParserState::FirstDigit => {
                match c {
                    b'0'..=b'9' => {
                        acc = (c - b'0') as u32;
                        path[j] = acc;
                        state = BIP32ParserState::Digit
                    },
                    _ => panic!("expected digit after '/'")
                }
            },
            // We are parsing digits for the current path token. We may also
            // find ' for hardening, or /.
            BIP32ParserState::Digit => {
                match c {
                    b'0'..=b'9' => {
                        acc = acc * 10 + (c - b'0') as u32;
                        path[j] = acc;
                    },
                    // Hardening
                    b'\'' => {
                        path[j] = acc + 0x800000;
                        j += 1;
                        state = BIP32ParserState::Hardened
                    },
                    // Separator for next number
                    b'/' => {
                        path[j] = acc;
                        j += 1;
                        state = BIP32ParserState::FirstDigit
                    },
                    _ => panic!("unexpected character in path")
                }
            },
            // Previous number has hardening. Next character must be a /
            // separator.
            BIP32ParserState::Hardened => {
                match c {
                    b'/' => state = BIP32ParserState::FirstDigit,
                    _ => panic!("expected '/' character after hardening")
                }
            },
        }
        i += 1;
    }

    // Prevent last character from being /
    match state {
        BIP32ParserState::FirstDigit => panic!("missing number in path"),
        _ => {}
    }

    // Assert we parsed the exact expected number of tokens in the path
    if j != N-1 {
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

    #[test]
    fn ecdsa() {
        // Test signature bindings with an ECDSA + verification
        let mut raw_key = [0u8; 32];
        let message = b"test_message1";
        let rnd_mode = (CX_RND_RFC6979 | CX_LAST) as i32;
        let hash = CX_SHA256;

        bip32_derive(CurvesId::Secp256k1, &PATH, &mut raw_key);

        let mut k = ec_init_key(CurvesId::Secp256k1, &raw_key);
        let (sig, sig_len) = ecdsa_sign(&k, rnd_mode, hash, message)?;

        let pubkey = ec_get_pubkey(CurvesId::Secp256k1, &mut k);

        let verif = ecdsa_verify(&pubkey, &sig[..sig_len as usize], rnd_mode, hash, message);

        assert_eq!(verif , true);
    }

    #[test]
    fn deterministic_ecdsa() {
        // Test signature bindings with a deterministic ECDSA + verification

        let mut raw_key = [0u8; 32];
        let message = b"test_message";
        let rnd_mode = (CX_RND_RFC6979 | CX_LAST) as i32;
        let hash = CX_SHA256;

        bip32_derive(CurvesId::Secp256k1, &PATH, &mut raw_key);

        let mut k = ec_init_key(CurvesId::Secp256k1, &raw_key);
        let (sig, sig_len) = ecdsa_sign(&k, rnd_mode, hash, message)?;

        let pubkey = ec_get_pubkey(CurvesId::Secp256k1, &mut k);
        let verif = ecdsa_verify(&pubkey, &sig[..sig_len as usize], rnd_mode, hash, message);

        assert_eq!(verif , true);
    }

}