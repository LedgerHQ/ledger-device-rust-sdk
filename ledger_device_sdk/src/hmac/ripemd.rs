use super::HMACInit;
use core::mem;
use ledger_secure_sdk_sys::{cx_hmac_ripemd160_init_no_throw, cx_hmac_ripemd160_t, cx_hmac_t};

use super::impl_hmac;
impl_hmac!(
    Ripemd160,
    cx_hmac_ripemd160_t,
    cx_hmac_ripemd160_init_no_throw
);

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hmac::ripemd::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_MSG: &[u8; 29] = b"Not your keys, not your coins";
    const TEST_KEY: &[u8; 16] = b"hmac test key!!!";

    #[test]
    fn test_hmac_ripemd160() {
        let mut mac = Ripemd160::new(TEST_KEY);

        let mut output: [u8; 20] = [0u8; 20];

        let _ = mac.hmac(TEST_MSG, &mut output);

        let expected = [
            0xfa, 0xde, 0x57, 0x70, 0xf8, 0xa5, 0x04, 0x1a, 0xac, 0xdb, 0xe1, 0xc5, 0x64, 0x21,
            0x0d, 0xa6, 0x89, 0x9b, 0x2e, 0x6f,
        ];
        assert_eq!(&output, &expected);
    }
}
