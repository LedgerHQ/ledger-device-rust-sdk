use super::HashInit;
use ledger_secure_sdk_sys::{
    cx_blake2b_init_no_throw, cx_blake2b_init2_no_throw, cx_blake2b_t, cx_hash_t,
};

use super::impl_hash;
impl_hash!(Blake2b_256, cx_blake2b_t, cx_blake2b_init_no_throw, 256);
impl_hash!(Blake2b_384, cx_blake2b_t, cx_blake2b_init_no_throw, 384);
impl_hash!(Blake2b_512, cx_blake2b_t, cx_blake2b_init_no_throw, 512);

pub trait Blake2bWithPerso: HashInit {
    fn new_with_salt_and_perso(salt: Option<&mut [u8]>, perso: Option<&mut [u8]>) -> Self;
}

macro_rules! impl_blake2b_with_perso {
    ($type:ident, $bits:expr) => {
        impl Blake2bWithPerso for $type {
            fn new_with_salt_and_perso(salt: Option<&mut [u8]>, perso: Option<&mut [u8]>) -> Self {
                let (salt_ptr, salt_len) = match salt {
                    Some(s) => (s.as_mut_ptr(), s.len()),
                    None => (::core::ptr::null_mut(), 0),
                };
                let (perso_ptr, perso_len) = match perso {
                    Some(p) => (p.as_mut_ptr(), p.len()),
                    None => (::core::ptr::null_mut(), 0),
                };
                let mut ctx: $type = Default::default();
                let _err = unsafe {
                    cx_blake2b_init2_no_throw(
                        &mut ctx.ctx,
                        $bits,
                        salt_ptr,
                        salt_len,
                        perso_ptr,
                        perso_len,
                    )
                };
                ctx
            }
        }
    };
}

impl_blake2b_with_perso!(Blake2b_256, 256);
impl_blake2b_with_perso!(Blake2b_384, 384);
impl_blake2b_with_perso!(Blake2b_512, 512);

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hash::blake2::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_HASH: &[u8; 29] = b"Not your keys, not your coins";

    #[test]
    fn test_hash_blake2b256() {
        let mut blake2 = Blake2b_256::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size = blake2.get_size();
        assert_eq!(ouput_size, 32);

        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0xcd, 0xa6, 0x49, 0x8e, 0x2f, 0x89, 0x71, 0xe8, 0x4e, 0xd5, 0x68, 0x2e, 0x3d, 0x47,
            0x9c, 0xcc, 0x2c, 0xce, 0x7f, 0x37, 0xac, 0x92, 0x9c, 0xa0, 0xb0, 0x41, 0xb2, 0xdd,
            0x06, 0xa9, 0xf3, 0xcb,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_hash_blake2b384() {
        let mut blake2 = Blake2b_384::new();

        let mut output: [u8; 48] = [0u8; 48];

        let ouput_size = blake2.get_size();
        assert_eq!(ouput_size, 48);

        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0x5f, 0x03, 0x04, 0x77, 0x92, 0x5e, 0x91, 0x29, 0xf9, 0xb8, 0xef, 0xf9, 0x88, 0x29,
            0x04, 0xf4, 0x4f, 0x65, 0x3b, 0xef, 0xf8, 0x21, 0xca, 0x48, 0x68, 0xa7, 0xbe, 0x46,
            0x1c, 0x45, 0x82, 0xb3, 0x3d, 0xd7, 0x7b, 0x9e, 0x91, 0x9a, 0xfe, 0x1c, 0x3b, 0xed,
            0x4b, 0x8f, 0x3c, 0x5d, 0xde, 0x53,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_hash_blake2b512() {
        let mut blake2 = Blake2b_512::new();

        let mut output: [u8; 64] = [0u8; 64];

        let ouput_size = blake2.get_size();
        assert_eq!(ouput_size, 64);

        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0xc2, 0xe0, 0xfe, 0x8c, 0xb7, 0x83, 0x43, 0x7c, 0x8f, 0x36, 0x89, 0x48, 0xc4, 0x7a,
            0x9c, 0x7c, 0x27, 0xa3, 0xb5, 0x98, 0x7a, 0x2d, 0x1b, 0x3b, 0xab, 0x48, 0x3d, 0xd6,
            0xf6, 0x4c, 0xd1, 0x20, 0x7d, 0x72, 0x62, 0xb5, 0x35, 0xfe, 0x3f, 0x86, 0xad, 0x0c,
            0x5f, 0x33, 0x4e, 0x55, 0x07, 0x64, 0x49, 0x7c, 0x11, 0xd5, 0xbd, 0x6a, 0x44, 0x2a,
            0x9c, 0x2e, 0x6a, 0xab, 0xf9, 0x31, 0xc0, 0xab,
        ];
        assert_eq!(&output, &expected);
    }

    // ------------------------------------------------------------------
    // Blake2bWithPerso: new_with_salt_and_perso
    // ------------------------------------------------------------------

    #[test]
    fn test_blake2b256_with_no_salt_no_perso() {
        // None/None must produce the same digest as new()
        let mut blake2 = Blake2b_256::new_with_salt_and_perso(None, None);
        let mut output = [0u8; 32];
        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0xcd, 0xa6, 0x49, 0x8e, 0x2f, 0x89, 0x71, 0xe8, 0x4e, 0xd5, 0x68, 0x2e, 0x3d, 0x47,
            0x9c, 0xcc, 0x2c, 0xce, 0x7f, 0x37, 0xac, 0x92, 0x9c, 0xa0, 0xb0, 0x41, 0xb2, 0xdd,
            0x06, 0xa9, 0xf3, 0xcb,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_blake2b384_with_no_salt_no_perso() {
        let mut blake2 = Blake2b_384::new_with_salt_and_perso(None, None);
        let mut output = [0u8; 48];
        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0x5f, 0x03, 0x04, 0x77, 0x92, 0x5e, 0x91, 0x29, 0xf9, 0xb8, 0xef, 0xf9, 0x88, 0x29,
            0x04, 0xf4, 0x4f, 0x65, 0x3b, 0xef, 0xf8, 0x21, 0xca, 0x48, 0x68, 0xa7, 0xbe, 0x46,
            0x1c, 0x45, 0x82, 0xb3, 0x3d, 0xd7, 0x7b, 0x9e, 0x91, 0x9a, 0xfe, 0x1c, 0x3b, 0xed,
            0x4b, 0x8f, 0x3c, 0x5d, 0xde, 0x53,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_blake2b512_with_no_salt_no_perso() {
        let mut blake2 = Blake2b_512::new_with_salt_and_perso(None, None);
        let mut output = [0u8; 64];
        let _ = blake2.hash(TEST_HASH, &mut output);

        let expected = [
            0xc2, 0xe0, 0xfe, 0x8c, 0xb7, 0x83, 0x43, 0x7c, 0x8f, 0x36, 0x89, 0x48, 0xc4, 0x7a,
            0x9c, 0x7c, 0x27, 0xa3, 0xb5, 0x98, 0x7a, 0x2d, 0x1b, 0x3b, 0xab, 0x48, 0x3d, 0xd6,
            0xf6, 0x4c, 0xd1, 0x20, 0x7d, 0x72, 0x62, 0xb5, 0x35, 0xfe, 0x3f, 0x86, 0xad, 0x0c,
            0x5f, 0x33, 0x4e, 0x55, 0x07, 0x64, 0x49, 0x7c, 0x11, 0xd5, 0xbd, 0x6a, 0x44, 0x2a,
            0x9c, 0x2e, 0x6a, 0xab, 0xf9, 0x31, 0xc0, 0xab,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_blake2b256_with_salt_and_perso() {
        // BLAKE2b salt and personalization are each 16 bytes
        let mut salt = [0x01u8; 16];
        let mut perso = [0x02u8; 16];

        let mut blake2 = Blake2b_256::new_with_salt_and_perso(Some(&mut salt), Some(&mut perso));
        let mut output = [0u8; 32];
        let _ = blake2.hash(TEST_HASH, &mut output);

        // With salt/perso the digest must differ from the plain one
        let plain_expected = [
            0xcd, 0xa6, 0x49, 0x8e, 0x2f, 0x89, 0x71, 0xe8, 0x4e, 0xd5, 0x68, 0x2e, 0x3d, 0x47,
            0x9c, 0xcc, 0x2c, 0xce, 0x7f, 0x37, 0xac, 0x92, 0x9c, 0xa0, 0xb0, 0x41, 0xb2, 0xdd,
            0x06, 0xa9, 0xf3, 0xcb,
        ];
        assert_eq!(output != plain_expected, true);
    }

    #[test]
    fn test_blake2b256_with_salt_and_perso_deterministic() {
        let mut salt = [0x01u8; 16];
        let mut perso = [0x02u8; 16];
        let mut blake2 = Blake2b_256::new_with_salt_and_perso(Some(&mut salt), Some(&mut perso));
        let mut output1 = [0u8; 32];
        let _ = blake2.hash(TEST_HASH, &mut output1);

        // Second call with the same parameters must produce the same digest
        let mut salt2 = [0x01u8; 16];
        let mut perso2 = [0x02u8; 16];
        let mut blake2_2 =
            Blake2b_256::new_with_salt_and_perso(Some(&mut salt2), Some(&mut perso2));
        let mut output2 = [0u8; 32];
        let _ = blake2_2.hash(TEST_HASH, &mut output2);

        assert_eq!(&output1, &output2);
    }
}
