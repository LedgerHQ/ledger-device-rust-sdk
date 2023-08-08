use nanos_sdk::bindings::*;

extern "C" {
    pub fn cx_hkdf_extract(hash_id: cx_md_t, ikm: &[u8], ikm_len: u32, 
        salt: &[u8], salt_len: u32, prk: &[u8]);
}
extern "C" {
    pub fn cx_hkdf_expand(hash_id: cx_md_t, prk: &[u8], prk_len: u32, info: &[u8], 
        info_len: u32, okm: &[u8], okm_len: u32);
}

/// Wrapper for cx_hkdf_extract
/// 
/// Extract entropy 
/// 
/// In: 
/// ikm: input key material (acting as data)
/// salt: salt acting as a key
/// prk: pseudorandom key
pub fn hkdf_extract(ikm: &[u8], salt: &mut [u8], prk: &mut [u8]) {
    unsafe {
        cx_hkdf_extract(
            CX_SHA256,
            ikm.as_ptr(),
            ikm.len() as u32,
            salt.as_mut_ptr(),
            salt.len() as u32,
            prk.as_mut_ptr()
        );
    }
}

/// Wrapper for cx_hkdf_expand
/// 
/// Expand generated output of an already reasonably random input 
/// 
/// In: 
/// prk: pseudorandom key
/// info: 
/// okm: output key material
pub fn hkdf_expand(prk: &[u8], info: &[u8], okm: &mut [u8]) {
    unsafe {
        cx_hkdf_expand(
            CX_SHA256,
            prk.as_ptr(),
            prk.len() as u32,
            info.as_mut_ptr(),
            info.len() as u32,
            okm.as_mut_ptr(),
            okm.len() as u32
        );
    }
}