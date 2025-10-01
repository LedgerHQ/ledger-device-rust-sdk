use crate::ecc::CurvesId;
use crate::io::Reply;
use ledger_secure_sdk_sys::{
    cx_ecfp_384_public_key_t, os_pki_get_info, os_pki_verify, CERTIFICATE_TRUSTED_NAME_MAXLEN,
};

pub enum PkiVerifyError {
    Success = 0,
    MissingCertificate = 1,
    WrongCertificateUsage = 2,
    WrongCertificateCurve = 3,
    WrongSignature = 4,
}

impl From<PkiVerifyError> for Reply {
    fn from(exc: PkiVerifyError) -> Reply {
        Reply(0x6900 + exc as u16)
    }
}

pub fn pki_verify_data(
    data: &mut [u8],
    expected_key_usage: u8,
    expected_curve: CurvesId,
    signature: &mut [u8],
) -> Result<(), PkiVerifyError> {
    let certificate_name = [0u8; CERTIFICATE_TRUSTED_NAME_MAXLEN as usize];
    let mut certficate_name_len: usize = 0;
    let mut key_usage: u8 = 0;
    let mut pub_key = cx_ecfp_384_public_key_t::default();

    let err = unsafe {
        os_pki_get_info(
            &mut key_usage as *mut u8,
            certificate_name.as_ptr() as *mut u8,
            &mut certficate_name_len as *mut usize,
            &mut pub_key as *mut cx_ecfp_384_public_key_t,
        )
    };
    if err != 0 {
        return Err(PkiVerifyError::MissingCertificate);
    }
    if key_usage != expected_key_usage {
        return Err(PkiVerifyError::WrongCertificateUsage);
    }
    if pub_key.curve != expected_curve as u8 {
        return Err(PkiVerifyError::WrongCertificateCurve);
    }

    let err = unsafe {
        os_pki_verify(
            data.as_mut_ptr() as *mut u8,
            data.len(),
            signature.as_mut_ptr() as *mut u8,
            signature.len(),
        )
    };
    if err == true {
        Ok(())
    } else {
        Err(PkiVerifyError::WrongSignature)
    }
}
