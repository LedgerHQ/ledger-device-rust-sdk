use core::default::Default;
use core::fmt::Display;
use core::ops::{Add, AddAssign, Mul, Rem, RemAssign, Sub, SubAssign};
use ledger_secure_sdk_sys::{
    CX_OK, cx_math_add_no_throw, cx_math_addm_no_throw, cx_math_cmp_no_throw,
    cx_math_invintm_no_throw, cx_math_invprimem_no_throw, cx_math_is_prime_no_throw,
    cx_math_modm_no_throw, cx_math_mult_no_throw, cx_math_multm_no_throw,
    cx_math_next_prime_no_throw, cx_math_powm_no_throw, cx_math_sub_no_throw,
    cx_math_subm_no_throw,
};

#[derive(Debug, Copy, Clone)]
pub struct BigUint<const N: usize> {
    pub data: [u8; N],
}

impl<const N: usize> Display for BigUint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for byte in &self.data {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl BigUint<4> {
    pub fn invintm(&self, modulus: &Self) -> Self {
        let v: u32 = u32::from_be_bytes(self.data);
        let mut res = BigUint::<4>::default();
        unsafe {
            let err = cx_math_invintm_no_throw(res.data.as_mut_ptr(), v, modulus.data.as_ptr(), 4);
            match err {
                CX_OK => res,
                _ => panic!(
                    "Error computing inverse of BigUint with error code: {}",
                    err
                ),
            }
        }
    }
}

impl<const N: usize> BigUint<N> {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != N {
            return None;
        }
        let mut data = [0u8; N];
        data.copy_from_slice(slice);
        Some(Self { data })
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        N
    }

    pub fn is_empty(&self) -> bool {
        N == 0
    }

    pub fn addm(&self, other: &Self, modulus: &Self) -> Self {
        if self >= modulus || other >= modulus {
            panic!("Operands must be less than modulus");
        }

        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_addm_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error adding BigUint with error code: {}", err),
            }
        }
    }

    pub fn mulm(&self, other: &Self, modulus: &Self) -> Self {
        if other >= modulus {
            panic!("Second operand must be less than modulus");
        }

        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_multm_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error multiplying BigUint with error code: {}", err),
            }
        }
    }

    pub fn subm(&self, other: &Self, modulus: &Self) -> Self {
        if self >= modulus || other >= modulus {
            panic!("Operands must be less than modulus");
        }

        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_subm_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error subtracting BigUint with error code: {}", err),
            }
        }
    }

    pub fn powm(&self, exponent: &Self, modulus: &Self) -> Self {
        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_powm_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                exponent.data.as_ptr(),
                N,
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error exponentiating BigUint with error code: {}", err),
            }
        }
    }

    pub fn invprimem(&self, modulus: &Self) -> Self {
        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_invprimem_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!(
                    "Error computing inverse of BigUint with error code: {}",
                    err
                ),
            }
        }
    }

    pub fn is_prime(&self) -> bool {
        let mut is_prime: bool = false;
        unsafe {
            let err = cx_math_is_prime_no_throw(self.data.as_ptr(), N, &mut is_prime as *mut bool);
            match err {
                CX_OK => is_prime,
                _ => panic!(
                    "Error checking primality of BigUint with error code: {}",
                    err
                ),
            }
        }
    }

    pub fn next_prime(&self) -> Self {
        let mut res = *self;
        unsafe {
            let err = cx_math_next_prime_no_throw(res.data.as_mut_ptr(), res.len() as u32);
            match err {
                CX_OK => res,
                _ => panic!(
                    "Error computing next prime of BigUint with error code: {}",
                    err
                ),
            }
        }
    }
}

/// `a` widened to `M` bytes (`M` is `2 * N` for the callers).
fn widen<const N: usize, const M: usize>(a: &BigUint<N>) -> BigUint<M> {
    let mut res = BigUint::<M>::default();
    res.data[M - N..].copy_from_slice(&a.data);
    res
}

/// `a * b` on `M` bytes (`M` is `2 * N` for the callers).
fn mul_wide<const N: usize, const M: usize>(a: &BigUint<N>, b: &BigUint<N>) -> BigUint<M> {
    let mut res = BigUint::<M>::default();
    unsafe {
        let err = cx_math_mult_no_throw(res.data.as_mut_ptr(), a.data.as_ptr(), b.data.as_ptr(), N);
        match err {
            CX_OK => res,
            _ => panic!("Error multiplying BigUint with error code: {}", err),
        }
    }
}

// `to_double` and `Mul` return a `BigUint` twice as large. Without the
// unstable `generic_const_exprs` feature, that size cannot be computed from
// `N` in a signature, so they are implemented for a list of sizes: every size
// up to 64 bytes (512 bits), then up to 512 bytes (4096 bits).
macro_rules! double_size {
    ($($n:literal)*) => {
        $(
            impl BigUint<$n> {
                pub fn to_double(&self) -> BigUint<{ 2 * $n }> {
                    widen(self)
                }
            }

            impl Mul for BigUint<$n> {
                type Output = BigUint<{ 2 * $n }>;

                fn mul(self, other: BigUint<$n>) -> Self::Output {
                    mul_wide(&self, &other)
                }
            }
        )*
    };
}

double_size!(
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
    33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62
    63 64 96 128 192 256 384 512
);

impl<const N: usize> Default for BigUint<N> {
    fn default() -> Self {
        Self { data: [0; N] }
    }
}

impl<const N: usize> Add for BigUint<N> {
    type Output = BigUint<N>;
    fn add(self, other: Self) -> Self::Output {
        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_add_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error adding BigUint with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> AddAssign for BigUint<N> {
    fn add_assign(&mut self, other: Self) {
        unsafe {
            let err = cx_math_add_no_throw(
                self.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => {}
                _ => panic!("Error adding BigUint with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Sub for BigUint<N> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let mut res = BigUint::<N>::default();
        unsafe {
            let err = cx_math_sub_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error subtracting BigUint with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> SubAssign for BigUint<N> {
    fn sub_assign(&mut self, other: Self) {
        unsafe {
            let err = cx_math_sub_no_throw(
                self.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => {}
                _ => panic!("Error subtracting BigUint with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Rem for BigUint<N> {
    type Output = Self;

    fn rem(self, modulus: Self) -> Self::Output {
        let mut res = self;
        unsafe {
            let err = cx_math_modm_no_throw(res.data.as_mut_ptr(), N, modulus.data.as_ptr(), N);
            match err {
                CX_OK => res,
                _ => panic!(
                    "Error computing modulus of BigUint with error code: {}",
                    err
                ),
            }
        }
    }
}

impl<const N: usize> RemAssign for BigUint<N> {
    fn rem_assign(&mut self, modulus: Self) {
        unsafe {
            let err = cx_math_modm_no_throw(self.data.as_mut_ptr(), N, modulus.data.as_ptr(), N);
            match err {
                CX_OK => {}
                _ => panic!(
                    "Error computing modulus of BigUint with error code: {}",
                    err
                ),
            }
        }
    }
}

impl<const N: usize> PartialEq<BigUint<N>> for BigUint<N> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let mut diff: i32 = 0;
            let err = cx_math_cmp_no_throw(
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
                &mut diff as *mut i32,
            );
            match err {
                CX_OK => diff == 0,
                _ => panic!("Error comparing BigUint with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> PartialOrd<BigUint<N>> for BigUint<N> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        unsafe {
            let mut diff: i32 = 0;
            let err = cx_math_cmp_no_throw(
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
                &mut diff as *mut i32,
            );
            match err {
                CX_OK => match diff {
                    0 => Some(core::cmp::Ordering::Equal),
                    1 => Some(core::cmp::Ordering::Greater),
                    -1 => Some(core::cmp::Ordering::Less),
                    _ => None,
                },
                _ => panic!("Error comparing BigUint with error code: {}", err),
            }
        }
    }
}

// Conversion to and from Rust primitive u32
impl<const N: usize> From<u32> for BigUint<N> {
    fn from(value: u32) -> Self {
        assert!(N >= 4, "BigUint<{N}> is too small to represent a u32");
        let data = value.to_be_bytes();
        let mut r = Self::default();
        r.data[N - 4..N].copy_from_slice(&data);
        r
    }
}

impl<const N: usize> From<BigUint<N>> for u32 {
    fn from(value: BigUint<N>) -> Self {
        assert!(N >= 4, "BigUint<{N}> is too small to represent a u32");
        let slice = &value.data[N - 4..N];
        u32::from_be_bytes(slice.try_into().unwrap())
    }
}

// Conversion to and from Rust primitive u16
impl<const N: usize> From<u16> for BigUint<N> {
    fn from(value: u16) -> Self {
        assert!(N >= 2, "BigUint<{N}> is too small to represent a u16");
        let data = value.to_be_bytes();
        let mut r = Self::default();
        r.data[N - 2..N].copy_from_slice(&data);
        r
    }
}
impl<const N: usize> From<BigUint<N>> for u16 {
    fn from(value: BigUint<N>) -> Self {
        assert!(N >= 2, "BigUint<{N}> is too small to represent a u16");
        let slice = &value.data[N - 2..N];
        u16::from_be_bytes(slice.try_into().unwrap())
    }
}

// Conversion to and from Rust primitive u8
impl<const N: usize> From<u8> for BigUint<N> {
    fn from(value: u8) -> Self {
        assert!(N >= 1, "BigUint<{N}> is too small to represent a u8");
        let data = value.to_be_bytes();
        let mut r = Self::default();
        r.data[N - 1..N].copy_from_slice(&data);
        r
    }
}
impl<const N: usize> From<BigUint<N>> for u8 {
    fn from(value: BigUint<N>) -> Self {
        assert!(N >= 1, "BigUint<{N}> is too small to represent a u8");
        let slice = &value.data[N - 1..N];
        u8::from_be_bytes(slice.try_into().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::math::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn test_BigUint_add() {
        let a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        let c = a + b;
        let expected = BigUint::<4>::from_slice(&[5, 5, 5, 5]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_add_assign() {
        let mut a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        a += b;
        let expected = BigUint::<4>::from_slice(&[5, 5, 5, 5]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_BigUint_add_overflow() {
        let a = BigUint::<4>::from_slice(&[255, 255, 255, 255]).unwrap();
        let b = BigUint::<4>::from_slice(&[0, 0, 0, 1]).unwrap();
        let c = a + b;
        let expected = BigUint::<4>::from_slice(&[0, 0, 0, 0]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_sub() {
        let a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        let c = a - b;
        let expected = BigUint::<4>::from_slice(&[3, 0, 254, 253]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_sub_assign() {
        let mut a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        a -= b;
        let expected = BigUint::<4>::from_slice(&[3, 0, 254, 253]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_BigUint_sub_underflow() {
        let mut a = BigUint::<4>::from_slice(&[0, 0, 0, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[0, 0, 0, 2]).unwrap();
        a -= b;
        let expected = BigUint::<4>::from_slice(&[255, 255, 255, 255]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_BigUint_mul() {
        let a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[0, 0, 0, 2]).unwrap();
        let c = a * b;
        let expected = BigUint::<8>::from_slice(&[0, 0, 0, 0, 8, 6, 4, 2]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_to_double() {
        let a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let expected = BigUint::<8>::from_slice(&[0, 0, 0, 0, 4, 3, 2, 1]).unwrap();
        assert_eq!(&a.to_double(), &expected);
    }

    #[test]
    fn test_BigUint_rem() {
        let a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[2, 2, 2, 2]).unwrap();
        let c = a % b;
        let expected = BigUint::<4>::from_slice(&[2, 0, 255, 255]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_rem_assign() {
        let mut a = BigUint::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = BigUint::<4>::from_slice(&[2, 2, 2, 2]).unwrap();
        a %= b;
        let expected = BigUint::<4>::from_slice(&[2, 0, 255, 255]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_BigUint_addm() {
        let a = BigUint::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = BigUint::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.addm(&b, &m);
        let expected = BigUint::<4>::from_slice(&[0x05, 0x05, 0x05, 0x05]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_subm() {
        let a = BigUint::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = BigUint::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.subm(&b, &m);
        let expected = BigUint::<4>::from_slice(&[0x03, 0x00, 0xFE, 0xFD]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_mulm() {
        let a = BigUint::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = BigUint::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.mulm(&b, &m);
        let expected = BigUint::<4>::from_slice(&[0x1E, 0x1C, 0x21, 0x2C]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_powm() {
        let a = BigUint::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = BigUint::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.powm(&b, &m);
        let expected = BigUint::<4>::from_slice(&[0x5C, 0xAC, 0x83, 0x6E]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_invprimem() {
        let a = BigUint::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.invprimem(&m);
        let expected = BigUint::<4>::from_slice(&[0x57, 0xD2, 0xCD, 0x46]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_invintm() {
        let a = BigUint::<4>::from_slice(&[0x00, 0x00, 0x00, 0x15]).unwrap();
        let m = BigUint::<4>::from_slice(&[0x00, 0x00, 0x00, 0x16]).unwrap();
        let c = a.invintm(&m);
        let expected = BigUint::<4>::from_slice(&[0x00, 0x00, 0x00, 0x15]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_BigUint_is_prime() {
        let a = BigUint::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        assert_eq!(a.is_prime(), true);
        let b = BigUint::<4>::from_slice(&[0, 0, 0, 8]).unwrap();
        assert_eq!(b.is_prime(), false);
    }

    #[test]
    fn test_BigUint_next_prime() {
        let a = BigUint::<4>::from_slice(&[0, 0, 0, 7]).unwrap();
        let b = a.next_prime();
        let expected = BigUint::<4>::from_slice(&[0, 0, 0, 11]).unwrap();
        assert_eq!(&b, &expected);
    }
}
