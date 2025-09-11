use core::ops::{Add, AddAssign, Mul, Rem, RemAssign, Sub, SubAssign};
use core::default::Default;
use ledger_secure_sdk_sys::{
    cx_math_addm_no_throw,
    cx_math_add_no_throw,
    cx_math_cmp_no_throw,
    cx_math_invprimem_no_throw,
    cx_math_invintm_no_throw,
    cx_math_is_prime_no_throw,
    cx_math_modm_no_throw,
    cx_math_multm_no_throw,
    cx_math_mult_no_throw,
    cx_math_next_prime_no_throw,
    cx_math_powm_no_throw,
    cx_math_subm_no_throw,
    cx_math_sub_no_throw,
    CX_OK,
};


#[derive(Debug, Copy, Clone)]
pub struct Number<const N: usize> {
    pub data: [u8; N],
}

impl Number<4> {
    pub fn invintm(&self, modulus: &Self) -> Self {
        let v: u32 = u32::from_be_bytes(self.data);
        let mut res = Number::<4>::default();
        unsafe {
            let err = cx_math_invintm_no_throw(
                res.data.as_mut_ptr(),
                v,
                modulus.data.as_ptr(),
                4,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error computing inverse of Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Number<N> {
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

    pub fn addm(&self, other: &Self, modulus: &Self) -> Self {

        if self >= modulus || other >= modulus {
            panic!("Operands must be less than modulus");
        }

        let mut res = Number::<N>::default();
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
                _ => panic!("Error adding Number with error code: {}", err),
            }
        }
    }

    pub fn mulm(&self, other: &Self, modulus: &Self) -> Self {

        if other >= modulus {
            panic!("Second operand must be less than modulus");
        }

        let mut res = Number::<N>::default();
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
                _ => panic!("Error multiplying Number with error code: {}", err),
            }
        }
    }

    pub fn subm(&self, other: &Self, modulus: &Self) -> Self {

        if self >= modulus || other >= modulus {
            panic!("Operands must be less than modulus");
        }

        let mut res = Number::<N>::default();
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
                _ => panic!("Error subtracting Number with error code: {}", err),
            }
        }
    }

    pub fn powm(&self, exponent: &Self, modulus: &Self) -> Self {

        let mut res = Number::<N>::default();
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
                _ => panic!("Error exponentiating Number with error code: {}", err),
            }
        }
    }

    pub fn invprimem(&self, modulus: &Self) -> Self {

        let mut res = Number::<N>::default();
        unsafe {
            let err = cx_math_invprimem_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error computing inverse of Number with error code: {}", err),
            }
        }
    }

    pub fn is_prime(&self) -> bool {

        let mut is_prime: bool = false;
        unsafe {
            let err = cx_math_is_prime_no_throw(
                self.data.as_ptr(),
                N,
                &mut is_prime as *mut bool,
            );
            match err {
                CX_OK => {
                    return is_prime;
                }
                _ => panic!("Error checking primality of Number with error code: {}", err),
            }
        }
    }

    pub fn next_prime(&self) -> Self {
        let mut res = self.clone();
        unsafe {
            let err = cx_math_next_prime_no_throw(
                res.data.as_mut_ptr(),
                res.len() as u32,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error computing next prime of Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Number<N> 
    where Number<{2 * N}>: Sized
 {
    pub fn to_double(&self) -> Number<{ 2 * N }> {
         let mut res = Number::<{2 * N}>::default();
        res.data[N..2 * N].copy_from_slice(&self.data);
        res
    }
}

impl<const N: usize> Default for Number<N> {
    fn default() -> Self {
        Self { data: [0; N] }
    }
}

impl<const N: usize> Add for Number<N> {
    type Output = Number<N>;
    fn add(self, other: Self) -> Self::Output {
        let mut res = Number::<N>::default();
        unsafe {
            let err = cx_math_add_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error adding Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> AddAssign for Number<N> {
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
                _ => panic!("Error adding Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Sub for Number<N> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        let mut res = Number::<N>::default();
        unsafe {
            let err = cx_math_sub_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error subtracting Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> SubAssign for Number<N> {
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
                _ => panic!("Error subtracting Number with error code: {}", err),
            }
        }
    }
}

// impl<const N: usize, const M: usize> Mul<Number<M>> for Number<N> 
//     where Number<{N + M}>: Sized
impl<const N: usize> Mul for Number<N> 
    where Number<{N + N}>: Sized
{
    type Output = Number<{N + N}>;

    fn mul(self, other: Number<N>) -> Self::Output {
        let mut res = Number::<{N + N}>::default();
        unsafe {
            let err = cx_math_mult_no_throw(
                res.data.as_mut_ptr(),
                self.data.as_ptr(),
                other.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => res,
                _ => panic!("Error multiplying Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> Rem for Number<N> {
    type Output = Self;

    fn rem(self, modulus: Self) -> Self::Output {
        let mut res = self;
        unsafe {
            let err = cx_math_modm_no_throw(
                res.data.as_mut_ptr(),
                N,
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => return res,
                _ => panic!("Error computing modulus of Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> RemAssign for Number<N> {
    fn rem_assign(&mut self, modulus: Self) {
        unsafe {
            let err = cx_math_modm_no_throw(
                self.data.as_mut_ptr(),
                N,
                modulus.data.as_ptr(),
                N,
            );
            match err {
                CX_OK => {}
                _ => panic!("Error computing modulus of Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> PartialEq<Number<N>> for Number<N> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let mut diff: i32 = 0;
            let err = cx_math_cmp_no_throw(self.data.as_ptr(), other.data.as_ptr(), N, &mut diff as *mut i32);
            match err {
                CX_OK => {
                    if diff != 0 {
                        return false;
                    }
                    else {
                        return true;
                    }
                }
                _ => panic!("Error comparing Number with error code: {}", err),
            }
        }
    }
}

impl<const N: usize> PartialOrd<Number<N>> for Number<N> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        unsafe {
            let mut diff: i32 = 0;
            let err = cx_math_cmp_no_throw(self.data.as_ptr(), other.data.as_ptr(), N, &mut diff as *mut i32);
            match err {
                CX_OK => {
                    match diff {
                        0 => Some(core::cmp::Ordering::Equal),
                        1 => Some(core::cmp::Ordering::Greater),
                        -1 => Some(core::cmp::Ordering::Less),
                        _ => None,
                    }
                }
                _ => panic!("Error comparing Number with error code: {}", err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_err as assert_eq;
    use crate::math::*;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn test_number_add() {
        let a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        let c = a + b;
        let expected = Number::<4>::from_slice(&[5, 5, 5, 5]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_add_assign() {
        let mut a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        a += b;
        let expected = Number::<4>::from_slice(&[5, 5, 5, 5]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_number_add_overflow() {
        let a = Number::<4>::from_slice(&[255, 255, 255, 255]).unwrap();
        let b = Number::<4>::from_slice(&[0, 0, 0, 1]).unwrap();
        let c = a + b;
        let expected = Number::<4>::from_slice(&[0, 0, 0, 0]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_sub() {
        let a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        let c = a - b;
        let expected = Number::<4>::from_slice(&[3, 0, 254, 253]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_sub_assign() {
        let mut a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[1, 2, 3, 4]).unwrap();
        a -= b;
        let expected = Number::<4>::from_slice(&[3, 0, 254, 253]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_number_sub_underflow() {
        let mut a = Number::<4>::from_slice(&[0, 0, 0, 1]).unwrap();
        let b = Number::<4>::from_slice(&[0, 0, 0, 2]).unwrap();
        a -= b;
        let expected = Number::<4>::from_slice(&[255, 255, 255, 255]).unwrap();
        assert_eq!(&a, &expected);
    }

    #[test]
    fn test_number_mul() {
        let a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[0, 0, 0, 2]).unwrap();
        let c = a * b;
        let expected = Number::<8>::from_slice(&[0, 0, 0, 0, 8, 6, 4, 2]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_rem() {
        let a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[2, 2, 2, 2]).unwrap();
        let c = a % b;
        let expected = Number::<4>::from_slice(&[2, 0, 255, 255]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_rem_assign() {
        let mut a = Number::<4>::from_slice(&[4, 3, 2, 1]).unwrap();
        let b = Number::<4>::from_slice(&[2, 2, 2, 2]).unwrap();
        a %= b;
        let expected = Number::<4>::from_slice(&[2, 0, 255, 255]).unwrap();
        assert_eq!(&a, &expected);
    }


    #[test]
    fn test_number_addm() {
        let a = Number::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = Number::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.addm(&b, &m);
        let expected = Number::<4>::from_slice(&[0x05, 0x05, 0x05, 0x05]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_subm() {
        let a = Number::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = Number::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.subm(&b, &m);
        let expected = Number::<4>::from_slice(&[0x03, 0x00, 0xFE, 0xFD]).unwrap();
        assert_eq!(&c, &expected);
    }   

    #[test]
    fn test_number_mulm() {
        let a = Number::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = Number::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.mulm(&b, &m);
        let expected = Number::<4>::from_slice(&[0x1E, 0x1C, 0x21, 0x2C]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_powm() {
        let a = Number::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let b = Number::<4>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        let m = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.powm(&b, &m);
        let expected = Number::<4>::from_slice(&[0x5C, 0xAC, 0x83, 0x6E]).unwrap();
        assert_eq!(&c, &expected);
    }   

    #[test]
    fn test_number_invprimem() {
        let a = Number::<4>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap();
        let m = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        let c = a.invprimem(&m);
        let expected = Number::<4>::from_slice(&[0x57, 0xD2, 0xCD, 0x46]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_invintm() {
        let a = Number::<4>::from_slice(&[0x00, 0x00, 0x00, 0x15]).unwrap();
        let m = Number::<4>::from_slice(&[0x00, 0x00, 0x00, 0x16]).unwrap();
        let c = a.invintm(&m);
        let expected = Number::<4>::from_slice(&[0x00, 0x00, 0x00, 0x15]).unwrap();
        assert_eq!(&c, &expected);
    }

    #[test]
    fn test_number_is_prime() {
        let a = Number::<4>::from_slice(&[0x7F, 0xFF, 0xFF, 0xFF]).unwrap();
        assert_eq!(a.is_prime(), true);
        let b = Number::<4>::from_slice(&[0, 0, 0, 8]).unwrap();
        assert_eq!(b.is_prime(), false);
    }

    #[test]
    fn test_number_next_prime() {
        let a = Number::<4>::from_slice(&[0, 0, 0, 7]).unwrap();
        let b = a.next_prime();
        let expected = Number::<4>::from_slice(&[0, 0, 0, 11]).unwrap();
        assert_eq!(&b, &expected);
    } 
}  