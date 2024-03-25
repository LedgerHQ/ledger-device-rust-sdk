use ledger_secure_sdk_sys::{
    cx_hash_final, cx_hash_get_size, cx_hash_no_throw, cx_hash_t, cx_hash_update,
    CX_INVALID_PARAMETER, CX_LAST, CX_OK,
};

pub mod blake2;
pub mod ripemd;
pub mod sha2;
pub mod sha3;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum HashError {
    InvalidParameter,
    InvalidOutputLength,
    InternalError,
}

impl From<u32> for HashError {
    fn from(x: u32) -> HashError {
        match x {
            CX_INVALID_PARAMETER => HashError::InvalidParameter,
            _ => HashError::InternalError,
        }
    }
}

impl From<HashError> for u32 {
    fn from(e: HashError) -> u32 {
        e as u32
    }
}

pub trait HashInit: Sized {
    fn as_ctx_mut(&mut self) -> &mut cx_hash_t;
    fn as_ctx(&self) -> &cx_hash_t;
    fn new() -> Self;
    fn reset(&mut self);
    fn get_size(&mut self) -> usize {
        unsafe { cx_hash_get_size(self.as_ctx()) }
    }
    fn hash(&mut self, input: &[u8], output: &mut [u8]) -> Result<(), HashError> {
        let output_size = self.get_size();
        if output_size > output.len() {
            return Err(HashError::InvalidOutputLength);
        }

        let err = unsafe {
            cx_hash_no_throw(
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
    fn update(&mut self, input: &[u8]) -> Result<(), HashError> {
        let err = unsafe { cx_hash_update(self.as_ctx_mut(), input.as_ptr(), input.len()) };
        if err != CX_OK {
            Err(err.into())
        } else {
            Ok(())
        }
    }
    fn finalize(&mut self, output: &mut [u8]) -> Result<(), HashError> {
        let output_size = self.get_size();
        if output_size > output.len() {
            return Err(HashError::InvalidOutputLength);
        }

        let err = unsafe { cx_hash_final(self.as_ctx_mut(), output.as_mut_ptr()) };
        if err != CX_OK {
            Err(err.into())
        } else {
            Ok(())
        }
    }
}

macro_rules! impl_hash {
    ($typename:ident, $ctxname:ident, $initfname:ident, $size:expr) => {
        #[derive(Default)]
        #[allow(non_camel_case_types)]
        pub struct $typename {
            ctx: $ctxname,
        }
        impl HashInit for $typename {
            fn as_ctx_mut(&mut self) -> &mut cx_hash_t {
                &mut self.ctx.header
            }

            fn as_ctx(&self) -> &cx_hash_t {
                &self.ctx.header
            }

            fn new() -> Self {
                let mut ctx: $typename = Default::default();
                let _err = unsafe { $initfname(&mut ctx.ctx, $size) };
                ctx
            }

            fn reset(&mut self) {
                let _err = unsafe { $initfname(&mut self.ctx, $size) };
            }
        }
    };

    ($typename:ident, $ctxname:ident, $initfname:ident) => {
        #[derive(Default)]
        #[allow(non_camel_case_types)]
        pub struct $typename {
            ctx: $ctxname,
        }
        impl HashInit for $typename {
            fn as_ctx_mut(&mut self) -> &mut cx_hash_t {
                &mut self.ctx.header
            }

            fn as_ctx(&self) -> &cx_hash_t {
                &self.ctx.header
            }

            fn new() -> Self {
                let mut ctx: $typename = Default::default();
                let _err = unsafe { $initfname(&mut ctx.ctx) };
                ctx
            }

            fn reset(&mut self) {
                let _err = unsafe { $initfname(&mut self.ctx) };
            }
        }
    };
}
pub(crate) use impl_hash;

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::hash::sha2::Sha2_256;
    use crate::hash::sha3::*;
    use crate::hash::HashInit;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    const TEST_HASH: &[u8; 29] = b"Not your keys, not your coins";

    #[test]
    fn test_hash() {
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
    fn test_sha2_update() {
        let mut hasher = Sha2_256::new();

        let mut output: [u8; 32] = [0u8; 32];

        let ouput_size = hasher.get_size();
        assert_eq!(ouput_size, 32);

        let _ = hasher.update(TEST_HASH);

        let _ = hasher.finalize(&mut output);

        let expected = [
            0x52, 0x49, 0x2e, 0x81, 0x92, 0x16, 0xf3, 0x6b, 0x74, 0x7d, 0xd5, 0xda, 0x70, 0x3a,
            0x26, 0x60, 0x14, 0x34, 0x60, 0x42, 0x42, 0xfa, 0xb2, 0x7e, 0x85, 0x51, 0xe7, 0x82,
            0xa5, 0x11, 0x13, 0x40,
        ];
        assert_eq!(&output, &expected);
    }
}
