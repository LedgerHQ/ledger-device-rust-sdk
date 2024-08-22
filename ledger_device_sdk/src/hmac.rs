//! Hash-based message authentication code (HMAC) related functions
use ledger_secure_sdk_sys::{
    cx_hmac_final, cx_hmac_no_throw, cx_hmac_t, cx_hmac_update, CX_INVALID_PARAMETER, CX_LAST,
    CX_OK,
};

pub mod ripemd;
pub mod sha2;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum HMACError {
    InvalidParameter,
    InvalidOutputLength,
    InternalError,
}

impl From<u32> for HMACError {
    fn from(x: u32) -> HMACError {
        match x {
            CX_INVALID_PARAMETER => HMACError::InvalidParameter,
            _ => HMACError::InternalError,
        }
    }
}

impl From<HMACError> for u32 {
    fn from(e: HMACError) -> u32 {
        e as u32
    }
}

/// Defines the behavior of a rust HMAC object.
/// The implementation for a given algorithm is done using a rust macro
/// to avoid code duplication since only the C structures and functions
/// imported from the C SDK change.
pub trait HMACInit: Sized {
    /// Recovers a mutable version of the HMAC context that can be used
    /// to call HMAC related method in the C SDK.
    fn as_ctx_mut(&mut self) -> &mut cx_hmac_t;
    /// Recovers a constant version of the HMAC context that can be used
    /// to call HMAC related method in the C SDK.
    fn as_ctx(&self) -> &cx_hmac_t;
    /// Creates the HMAC object by initializing the associated context using
    /// the related C structure.
    fn new(key: &[u8]) -> Self;

    /// Computes a HMAC in one line by providing the complete input as well as the
    /// output buffer.
    /// An error can be returned if one of the parameter is invalid
    /// or if the output buffer size is not enough.
    fn hmac(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), HMACError> {
        let err = unsafe {
            cx_hmac_no_throw(
                self.as_ctx_mut(),
                CX_LAST,
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        if err != CX_OK {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Updates the current HMAC object state with the given input data.
    /// This method may be called as many times needed (useful for large bufferized
    /// inputs). This method should not be called after `finalize`.
    /// An error can be returned if the input is invalid or the context in a wrong state.
    fn update(&mut self, input: &[u8]) -> Result<(), HMACError> {
        let err = unsafe { cx_hmac_update(self.as_ctx_mut(), input.as_ptr(), input.len()) };
        if err != CX_OK {
            Err(err.into())
        } else {
            Ok(())
        }
    }

    /// Finalizes the computation of the MAC and stores the result in the output buffer
    /// as well as returning the MAC length.
    /// This method should be called after one or many calls to `update`.
    /// An error can be returned if one of the parameter is invalid
    /// or if the output buffer size is not enough.
    fn finalize(&mut self, output: &mut [u8]) -> Result<usize, HMACError> {
        let mut out_len = output.len();
        let err = unsafe { cx_hmac_final(self.as_ctx_mut(), output.as_mut_ptr(), &mut out_len) };
        if err != CX_OK {
            Err(err.into())
        } else {
            Ok(out_len)
        }
    }
}

/// This macro can be used to implement the HMACInit trait for a given hash
/// algorithm by providing the structure name, the C context name, and the C
/// context initialization function.
macro_rules! impl_hmac {
    ($typename:ident, $ctxname:ident, $initfname:ident) => {
        #[derive(Default)]
        #[allow(non_camel_case_types)]
        pub struct $typename {
            ctx: $ctxname,
        }
        impl HMACInit for $typename {
            fn as_ctx_mut(&mut self) -> &mut cx_hmac_t {
                unsafe { mem::transmute::<&mut $ctxname, &mut cx_hmac_t>(&mut self.ctx) }
            }

            fn as_ctx(&self) -> &cx_hmac_t {
                unsafe { mem::transmute::<&$ctxname, &cx_hmac_t>(&self.ctx) }
            }

            fn new(key: &[u8]) -> Self {
                let mut ctx: $typename = Default::default();
                let _err = unsafe {
                    $initfname(&mut ctx.ctx, key.as_ptr(), key.len().try_into().unwrap())
                };
                ctx
            }
        }
    };
}
pub(crate) use impl_hmac;

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hmac::ripemd::Ripemd160;
    use crate::hmac::HMACInit;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_MSG: &[u8; 29] = b"Not your keys, not your coins";
    const TEST_KEY: &[u8; 16] = b"hmac test key!!!";

    #[test]
    fn test_hmac_oneline() {
        let mut mac = Ripemd160::new(TEST_KEY);

        let mut output: [u8; 20] = [0u8; 20];

        let _ = mac.hmac(TEST_MSG, &mut output);

        let expected = [
            0xfa, 0xde, 0x57, 0x70, 0xf8, 0xa5, 0x04, 0x1a, 0xac, 0xdb, 0xe1, 0xc5, 0x64, 0x21,
            0x0d, 0xa6, 0x89, 0x9b, 0x2e, 0x6f,
        ];
        assert_eq!(&output, &expected);
    }

    #[test]
    fn test_hmac_update() {
        let mut mac = Ripemd160::new(TEST_KEY);

        let mut output: [u8; 20] = [0u8; 20];

        let _ = mac.update(TEST_MSG);

        let res = mac.finalize(&mut output);

        let expected = [
            0xfa, 0xde, 0x57, 0x70, 0xf8, 0xa5, 0x04, 0x1a, 0xac, 0xdb, 0xe1, 0xc5, 0x64, 0x21,
            0x0d, 0xa6, 0x89, 0x9b, 0x2e, 0x6f,
        ];
        assert_eq!(&output, &expected);
        assert_eq!(res.unwrap(), expected.len());
    }
}
