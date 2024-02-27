use super::HashInit;
use ledger_secure_sdk_sys::{cx_hash_t, cx_ripemd160_init_no_throw, cx_ripemd160_t};

use super::impl_hash;
impl_hash!(Ripemd160, cx_ripemd160_t, cx_ripemd160_init_no_throw);

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hash::ripemd::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_HASH: &[u8; 29] = b"Not your keys, not your coins";

    #[test]
    fn test_hash_ripemd160() {
        let mut ripemd = Ripemd160::new();

        let mut output: [u8; 20] = [0u8; 20];

        let ouput_size = ripemd.get_size();
        assert_eq!(ouput_size, 20);

        let _ = ripemd.hash(TEST_HASH, &mut output);

        let expected = [
            0x75, 0x0f, 0x75, 0x73, 0x6a, 0x34, 0xac, 0x02, 0xd0, 0x72, 0xec, 0x2a, 0xf5, 0xf7,
            0x1d, 0x16, 0xc2, 0x6f, 0x63, 0x23,
        ];
        assert_eq!(&output, &expected);
    }
}
