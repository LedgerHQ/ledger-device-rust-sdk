use crate::ecc::{CurvesId, Secret};
use ledger_secure_sdk_sys::*;

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

/// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-2645.md
pub fn eip2645_derive(path: &[u32], key: &mut [u8]) {
    let mut x_key = Secret::<64>::new();
    // Ignoring 'Result' here because known to be valid
    let _ = super::bip32_derive(CurvesId::Secp256k1, path, x_key.as_mut(), None);

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
