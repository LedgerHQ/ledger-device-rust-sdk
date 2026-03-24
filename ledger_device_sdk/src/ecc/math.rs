use super::*;
use crate::bn::*;
use crate::check_cx_ok;

// ---------------------------------------------------------------------------
// Curve domain parameter identifier
// ---------------------------------------------------------------------------

/// Identifies a specific domain parameter of an elliptic curve.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CurveDomainParam {
    None = CX_CURVE_PARAM_NONE,
    A = CX_CURVE_PARAM_A,
    B = CX_CURVE_PARAM_B,
    Field = CX_CURVE_PARAM_Field,
    Gx = CX_CURVE_PARAM_Gx,
    Gy = CX_CURVE_PARAM_Gy,
    Order = CX_CURVE_PARAM_Order,
    Cofactor = CX_CURVE_PARAM_Cofactor,
}

// ---------------------------------------------------------------------------
// EcPoint – safe wrapper around `cx_ecpoint_t`
// ---------------------------------------------------------------------------

/// Safe RAII wrapper around `cx_ecpoint_t`.
///
/// Allocates the point inside the BN context on creation and destroys it
/// on drop.  A [`BnLock`] **must** be held for the entire lifetime of
/// any `EcPoint`.
pub struct EcPoint {
    inner: cx_ecpoint_t,
}

impl EcPoint {
    /// Allocate a new EC point on the given curve.
    /// Wraps `cx_ecpoint_alloc`.
    pub fn new(curve: CurvesId) -> Result<Self, CxError> {
        let mut point = cx_ecpoint_t::default();
        check_cx_ok!(cx_ecpoint_alloc(&mut point, curve as u8));
        Ok(Self { inner: point })
    }

    /// Curve identifier of this point.
    pub fn curve(&self) -> CurvesId {
        self.inner.curve.into()
    }

    /// Initialize the point from raw byte coordinates.
    /// Wraps `cx_ecpoint_init`.
    pub fn init(&mut self, x: &[u8], y: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_init(
            &mut self.inner,
            x.as_ptr(),
            x.len(),
            y.as_ptr(),
            y.len(),
        ));
        Ok(())
    }

    /// Initialize the point from BN handles.
    /// Wraps `cx_ecpoint_init_bn`.
    pub fn init_bn(&mut self, x: Bn, y: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_init_bn(&mut self.inner, x.raw(), y.raw()));
        Ok(())
    }

    /// Export the point coordinates to raw byte buffers.
    /// Wraps `cx_ecpoint_export`.
    pub fn export(&self, x: &mut [u8], y: &mut [u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_export(
            &self.inner,
            x.as_mut_ptr(),
            x.len(),
            y.as_mut_ptr(),
            y.len(),
        ));
        Ok(())
    }

    /// Export the point coordinates to BN handles.
    /// Wraps `cx_ecpoint_export_bn`.
    pub fn export_bn(&self, x: &mut Bn, y: &mut Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_export_bn(&self.inner, x.raw_mut(), y.raw_mut()));
        Ok(())
    }

    /// Compress the point into a byte buffer.
    /// Returns the sign of the y-coordinate.
    /// Wraps `cx_ecpoint_compress`.
    pub fn compress(&self, xy_compressed: &mut [u8]) -> Result<u32, CxError> {
        let mut sign = 0u32;
        check_cx_ok!(cx_ecpoint_compress(
            &self.inner,
            xy_compressed.as_mut_ptr(),
            xy_compressed.len(),
            &mut sign,
        ));
        Ok(sign)
    }

    /// Decompress a point from a compressed representation.
    /// Wraps `cx_ecpoint_decompress`.
    pub fn decompress(&mut self, xy_compressed: &[u8], sign: u32) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_decompress(
            &mut self.inner,
            xy_compressed.as_ptr(),
            xy_compressed.len(),
            sign,
        ));
        Ok(())
    }

    /// Point addition: `self = P + Q`.
    /// Wraps `cx_ecpoint_add`.
    pub fn add(&mut self, p: &EcPoint, q: &EcPoint) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_add(&mut self.inner, &p.inner, &q.inner));
        Ok(())
    }

    /// Negate this point in place.
    /// Wraps `cx_ecpoint_neg`.
    pub fn neg(&mut self) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_neg(&mut self.inner));
        Ok(())
    }

    /// Scalar multiplication: `self = k · self` (scalar as raw bytes).
    /// Wraps `cx_ecpoint_scalarmul`.
    pub fn scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_scalarmul(&mut self.inner, k.as_ptr(), k.len()));
        Ok(())
    }

    /// Scalar multiplication: `self = k · self` (scalar as BN handle).
    /// Wraps `cx_ecpoint_scalarmul_bn`.
    pub fn scalarmul_bn(&mut self, bn_k: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_scalarmul_bn(&mut self.inner, bn_k.raw()));
        Ok(())
    }

    /// Randomised scalar multiplication (side-channel resistant).
    /// Wraps `cx_ecpoint_rnd_scalarmul`.
    pub fn rnd_scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_scalarmul(
            &mut self.inner,
            k.as_ptr(),
            k.len()
        ));
        Ok(())
    }

    /// Randomised scalar multiplication with BN scalar.
    /// Wraps `cx_ecpoint_rnd_scalarmul_bn`.
    pub fn rnd_scalarmul_bn(&mut self, bn_k: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_scalarmul_bn(&mut self.inner, bn_k.raw()));
        Ok(())
    }

    /// Randomised fixed-point scalar multiplication (side-channel resistant).
    /// Wraps `cx_ecpoint_rnd_fixed_scalarmul`.
    pub fn rnd_fixed_scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_fixed_scalarmul(
            &mut self.inner,
            k.as_ptr(),
            k.len()
        ));
        Ok(())
    }

    /// Double scalar multiplication: `self = k·P + r·Q` (raw-byte scalars).
    /// Wraps `cx_ecpoint_double_scalarmul`.
    pub fn double_scalarmul(
        &mut self,
        p: &mut EcPoint,
        q: &mut EcPoint,
        k: &[u8],
        r: &[u8],
    ) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_double_scalarmul(
            &mut self.inner,
            &mut p.inner,
            &mut q.inner,
            k.as_ptr(),
            k.len(),
            r.as_ptr(),
            r.len(),
        ));
        Ok(())
    }

    /// Double scalar multiplication: `self = k·P + r·Q` (BN scalars).
    /// Wraps `cx_ecpoint_double_scalarmul_bn`.
    pub fn double_scalarmul_bn(
        &mut self,
        p: &mut EcPoint,
        q: &mut EcPoint,
        bn_k: Bn,
        bn_r: Bn,
    ) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_double_scalarmul_bn(
            &mut self.inner,
            &mut p.inner,
            &mut q.inner,
            bn_k.raw(),
            bn_r.raw(),
        ));
        Ok(())
    }

    /// Compare two EC points for equality.
    /// Wraps `cx_ecpoint_cmp`.
    pub fn cmp(&self, other: &EcPoint) -> Result<bool, CxError> {
        let mut is_equal = false;
        check_cx_ok!(cx_ecpoint_cmp(&self.inner, &other.inner, &mut is_equal));
        Ok(is_equal)
    }

    /// Check whether this point lies on its curve.
    /// Wraps `cx_ecpoint_is_on_curve`.
    pub fn is_on_curve(&self) -> Result<bool, CxError> {
        let mut on_curve = false;
        check_cx_ok!(cx_ecpoint_is_on_curve(&self.inner, &mut on_curve));
        Ok(on_curve)
    }

    /// Check whether this point is the point at infinity.
    /// Wraps `cx_ecpoint_is_at_infinity`.
    pub fn is_at_infinity(&self) -> Result<bool, CxError> {
        let mut at_infinity = false;
        check_cx_ok!(cx_ecpoint_is_at_infinity(&self.inner, &mut at_infinity));
        Ok(at_infinity)
    }

    /// Access the inner `cx_ecpoint_t`.
    pub fn as_raw(&self) -> &cx_ecpoint_t {
        &self.inner
    }

    /// Access the inner `cx_ecpoint_t` mutably.
    pub fn as_raw_mut(&mut self) -> &mut cx_ecpoint_t {
        &mut self.inner
    }
}

impl Drop for EcPoint {
    fn drop(&mut self) {
        unsafe {
            cx_ecpoint_destroy(&mut self.inner);
        }
    }
}

// ---------------------------------------------------------------------------
// Curve-type macro – creates zero-sized types with domain query helpers
// ---------------------------------------------------------------------------

macro_rules! impl_math_curve {
    ($typename:ident) => {
        pub struct $typename {}
        impl $typename {
            pub fn id() -> CurvesId {
                CurvesId::$typename
            }

            /// Curve size in **bits**.
            /// Wraps `cx_ecdomain_size`.
            pub fn size_bits() -> usize {
                let mut length = 0usize;
                unsafe {
                    cx_ecdomain_size(Self::id() as u8, &mut length);
                }
                length
            }

            /// Curve field-element size in **bytes**.
            /// Wraps `cx_ecdomain_parameters_length`.
            pub fn size_bytes() -> usize {
                let mut length = 0usize;
                unsafe {
                    cx_ecdomain_parameters_length(Self::id() as u8, &mut length);
                }
                length
            }

            /// Retrieve a specific domain parameter as raw bytes.
            /// Wraps `cx_ecdomain_parameter`.
            pub fn domain_parameter(id: CurveDomainParam, buf: &mut [u8]) -> Result<(), CxError> {
                check_cx_ok!(cx_ecdomain_parameter(
                    Self::id() as u8,
                    id as u8,
                    buf.as_mut_ptr(),
                    buf.len() as u32,
                ));
                Ok(())
            }

            /// Retrieve a specific domain parameter as a BN handle.
            /// Requires the BN context to be locked.
            /// Wraps `cx_ecdomain_parameter_bn`.
            pub fn domain_parameter_bn(id: CurveDomainParam, bn: cx_bn_t) -> Result<(), CxError> {
                check_cx_ok!(cx_ecdomain_parameter_bn(Self::id() as u8, id as u8, bn));
                Ok(())
            }

            /// Retrieve the generator point as raw bytes (`Gx`, `Gy`).
            /// Both buffers must have the same length (the field-element size).
            /// Wraps `cx_ecdomain_generator`.
            pub fn generator(gx: &mut [u8], gy: &mut [u8]) -> Result<(), CxError> {
                check_cx_ok!(cx_ecdomain_generator(
                    Self::id() as u8,
                    gx.as_mut_ptr(),
                    gy.as_mut_ptr(),
                    gx.len(),
                ));
                Ok(())
            }

            /// Retrieve the generator point as an `EcPoint` (BN-based).
            /// Requires the BN context to be locked.
            /// Wraps `cx_ecdomain_generator_bn`.
            pub fn generator_bn(p: &mut EcPoint) -> Result<(), CxError> {
                check_cx_ok!(cx_ecdomain_generator_bn(Self::id() as u8, p.as_raw_mut(),));
                Ok(())
            }
        }
    };
}

// Instantiate all supported curves
impl_math_curve!(Secp256k1);
impl_math_curve!(Secp256r1);
impl_math_curve!(Secp384r1);
impl_math_curve!(BrainpoolP256T1);
impl_math_curve!(BrainpoolP256R1);
impl_math_curve!(BrainpoolP320R1);
impl_math_curve!(BrainpoolP320T1);
impl_math_curve!(BrainpoolP384T1);
impl_math_curve!(BrainpoolP384R1);
impl_math_curve!(BrainpoolP512T1);
impl_math_curve!(BrainpoolP512R1);
impl_math_curve!(Bls12381G1);
impl_math_curve!(FRP256v1);
impl_math_curve!(Stark256);
impl_math_curve!(Bls12377G1);
impl_math_curve!(Pallas);
impl_math_curve!(Vesta);
impl_math_curve!(Ed25519);
impl_math_curve!(Ed448);
impl_math_curve!(EdBLS12);
impl_math_curve!(JubJub);
impl_math_curve!(Curve25519);
impl_math_curve!(Curve448);
impl_math_curve!(Secp521r1);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn secp256k1_math() {
        assert_eq!(Secp256k1::id() as u8, CurvesId::Secp256k1 as u8);
        assert_eq!(Secp256k1::size_bits(), 256);
        assert_eq!(Secp256k1::size_bytes(), 32);
    }
}
