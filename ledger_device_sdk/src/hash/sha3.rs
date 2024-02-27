use super::HashInit;
use ledger_secure_sdk_sys::{
    cx_hash_t, cx_keccak_init_no_throw, cx_sha3_init_no_throw, cx_sha3_t,
    cx_shake128_init_no_throw, cx_shake256_init_no_throw,
};

use super::impl_hash;
impl_hash!(Keccak256, cx_sha3_t, cx_keccak_init_no_throw, 256);
impl_hash!(Sha3_224, cx_sha3_t, cx_sha3_init_no_throw, 224);
impl_hash!(Sha3_256, cx_sha3_t, cx_sha3_init_no_throw, 256);
impl_hash!(Sha3_384, cx_sha3_t, cx_sha3_init_no_throw, 384);
impl_hash!(Sha3_512, cx_sha3_t, cx_sha3_init_no_throw, 512);
impl_hash!(Shake128, cx_sha3_t, cx_shake128_init_no_throw, 128);
impl_hash!(Shake256, cx_sha3_t, cx_shake256_init_no_throw, 256);

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hash::sha3::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_HASH: &[u8; 29] = b"Not your keys, not your coins";

    #[test]
    fn test_hash_keccak() {
        let mut keccak = Keccak256::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size = keccak.get_size();
        assert_eq!(ouput_size, 32);

        let _ = keccak.hash(TEST_HASH, &mut output);

        let expected = [
            0x1f, 0x20, 0x7c, 0xd9, 0xfd, 0x9f, 0x0b, 0x09, 0xb0, 0x04, 0x93, 0x6c, 0xa5, 0xe0,
            0xd3, 0x1b, 0xa1, 0x6c, 0xd6, 0x14, 0x53, 0xaa, 0x28, 0x7e, 0x65, 0xaa, 0x88, 0x25,
            0x3c, 0xdc, 0x1c, 0x94,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_hash_sha3224() {
        let mut sha3224 = Sha3_224::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size: usize = sha3224.get_size();
        assert_eq!(ouput_size, 28);

        let _ = sha3224.hash(TEST_HASH, &mut output);

        let expected = [
            0x92, 0xd0, 0x94, 0x85, 0xb4, 0x74, 0xdc, 0x58, 0xcc, 0xbb, 0x03, 0xb5, 0x0e, 0x1c,
            0x1c, 0xe2, 0xab, 0x33, 0xd5, 0xf2, 0xf9, 0xbd, 0xfd, 0xda, 0xcd, 0x88, 0xc6, 0xfc,
        ];
        assert_eq!(&output[..28], &expected);
    }

    #[test]
    fn test_hash_sha3256() {
        let mut sha3256 = Sha3_256::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size: usize = sha3256.get_size();
        assert_eq!(ouput_size, 32);

        let _ = sha3256.hash(TEST_HASH, &mut output);

        let expected = [
            0x80, 0x8b, 0x0a, 0xd9, 0xdd, 0x0f, 0xe7, 0x6f, 0x8d, 0xb4, 0xbb, 0x99, 0xe6, 0x3e,
            0x9d, 0x24, 0xce, 0xa6, 0x4c, 0xfc, 0xdf, 0x93, 0x7a, 0xdb, 0xca, 0x9a, 0xe1, 0x1f,
            0x27, 0x3a, 0x00, 0xb7,
        ];
        assert_eq!(&output[..32], &expected);
    }

    #[test]
    fn test_hash_sha3384() {
        let mut sha3384 = Sha3_384::new();

        let mut output: [u8; 48] = [0u8; 48];

        let ouput_size: usize = sha3384.get_size();
        assert_eq!(ouput_size, 48);

        let _ = sha3384.hash(TEST_HASH, &mut output);

        let expected = [
            0x16, 0xdd, 0xbe, 0xd5, 0xf8, 0x95, 0xf9, 0x04, 0xe9, 0xb8, 0xc2, 0x71, 0xe9, 0xa0,
            0x66, 0x7d, 0xa4, 0x35, 0x7e, 0xd9, 0x87, 0x4b, 0x27, 0x23, 0x00, 0xf9, 0xd5, 0x10,
            0x55, 0xd5, 0x6c, 0x65, 0xe3, 0x3d, 0x83, 0xd7, 0xff, 0x8e, 0xab, 0x98, 0x80, 0x60,
            0xe5, 0x9c, 0x1a, 0x34, 0xe7, 0xdc,
        ];
        assert_eq!(&output[..48], &expected);
    }

    #[test]
    fn test_hash_sha3512() {
        let mut sha3512 = Sha3_512::new();

        let mut output: [u8; 64] = [0u8; 64];

        let ouput_size: usize = sha3512.get_size();
        assert_eq!(ouput_size, 64);

        let _ = sha3512.hash(TEST_HASH, &mut output);

        let expected = [
            0x4e, 0x81, 0x24, 0xc2, 0xed, 0x1e, 0x9a, 0x1c, 0x60, 0x0f, 0xc0, 0x6b, 0x49, 0xd3,
            0xa3, 0x54, 0x28, 0x81, 0x86, 0x62, 0xf0, 0xcd, 0x95, 0x1d, 0x67, 0x58, 0x3d, 0x8d,
            0x28, 0xab, 0x97, 0x9a, 0x56, 0xab, 0x57, 0xa3, 0x78, 0x15, 0x01, 0x86, 0x6a, 0x00,
            0xf3, 0x89, 0x11, 0x7d, 0x7e, 0x47, 0x84, 0xcd, 0x0c, 0x00, 0x25, 0xad, 0xac, 0xbe,
            0x00, 0xb2, 0xf5, 0xf2, 0x6e, 0x0d, 0x61, 0x59,
        ];
        assert_eq!(&output[..64], &expected);
    }

    #[test]
    fn test_hash_shake128() {
        let mut shake128 = Shake128::new();

        let mut output: [u8; 16] = [0u8; 16];

        let ouput_size: usize = shake128.get_size();
        assert_eq!(ouput_size, 16);

        let _ = shake128.hash(TEST_HASH, &mut output);

        let expected = [
            0x45, 0xd9, 0xa1, 0x61, 0x7b, 0x0d, 0x7b, 0xb1, 0xf1, 0x09, 0x63, 0xe1, 0xb0, 0xa5,
            0xaa, 0x2c,
        ];
        assert_eq!(&output[..16], &expected);
    }

    #[test]
    fn test_hash_shake256() {
        let mut shake256 = Shake256::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size: usize = shake256.get_size();
        assert_eq!(ouput_size, 32);

        let _ = shake256.hash(TEST_HASH, &mut output);

        let expected = [
            0x3d, 0x51, 0xd1, 0xfc, 0x5e, 0x2a, 0x3e, 0x4b, 0x9c, 0xdf, 0x2b, 0x03, 0x18, 0xf5,
            0xd1, 0x91, 0x87, 0x4d, 0x52, 0xc1, 0x8c, 0x7b, 0x33, 0x36, 0x52, 0x7b, 0x0b, 0x64,
            0x28, 0xfa, 0xad, 0xf1,
        ];
        assert_eq!(&output[..32], &expected);
    }
}
