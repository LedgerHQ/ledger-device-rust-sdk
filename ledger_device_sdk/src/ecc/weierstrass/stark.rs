use crate::ecc::{ChainCode, CurvesId, ECPrivateKey, Secret, SeedDerive, bip32_derive};
use crate::impl_curve;
use ledger_secure_sdk_sys::*;

impl_curve!(Stark256, 32, 'W');

// C_cx_secp256k1_n - (C_cx_secp256k1_n % C_cx_Stark256_n)
const STARK_DERIVE_BIAS: [u8; 32] = [
    0xf8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x0e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf7,
    0x38, 0xa1, 0x3b, 0x4b, 0x92, 0x0e, 0x94, 0x11, 0xae, 0x6d, 0xa5, 0xf4, 0x0b, 0x03, 0x58, 0xb1,
];

// n: 0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f
const C_CX_STARK256_N: [u8; 32] = [
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xb7, 0x81, 0x12, 0x6d, 0xca, 0xe7, 0xb2, 0x32, 0x1e, 0x66, 0xa2, 0x41, 0xad, 0xc6, 0x4d, 0x2f,
];

impl SeedDerive for Stark256 {
    type Target = ECPrivateKey<32, 'W'>;
    fn derive_from(path: &[u32]) -> (Self::Target, Option<ChainCode>) {
        let mut sk = Self::Target::new(CurvesId::Stark256);
        eip2645_derive(path, &mut sk.key);
        (sk, None)
    }
}

/// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2645.md
fn eip2645_derive(path: &[u32], key: &mut [u8]) {
    let mut x_key = Secret::<64>::new();
    // Ignoring 'Result' here because known to be valid
    let _ = bip32_derive(CurvesId::Secp256k1, path, x_key.as_mut(), None);

    let mut index = 0;
    let mut cmp = 0;

    loop {
        x_key.as_mut()[32] = index;
        unsafe { cx_hash_sha256(x_key.as_ref().as_ptr(), 33, key.as_mut_ptr(), 32) };
        unsafe { cx_math_cmp_no_throw(key.as_ptr(), STARK_DERIVE_BIAS.as_ptr(), 32, &mut cmp) };
        if cmp < 0 {
            unsafe { cx_math_modm_no_throw(key.as_mut_ptr(), 32, C_CX_STARK256_N.as_ptr(), 32) };
            break;
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::ecc::{CxError, SeedDerive, make_bip32_path};
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
}
