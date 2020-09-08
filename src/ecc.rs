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

