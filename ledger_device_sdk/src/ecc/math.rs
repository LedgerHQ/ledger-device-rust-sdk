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

    fn err(e: CxError) {
        let ec = crate::testing::to_hex(e.into());
        crate::log::info!(
            "EC math error: \x1b[1;33m{}\x1b[0m",
            core::str::from_utf8(&ec).unwrap()
        );
    }

    // ------------------------------------------------------------------
    // Curve domain queries
    // ------------------------------------------------------------------

    #[test]
    fn secp256k1_math() {
        assert_eq!(Secp256k1::id() as u8, CurvesId::Secp256k1 as u8);
        assert_eq!(Secp256k1::size_bits(), 256);
        assert_eq!(Secp256k1::size_bytes(), 32);
    }

    #[test]
    fn secp256r1_size() {
        assert_eq!(Secp256r1::id() as u8, CurvesId::Secp256r1 as u8);
        assert_eq!(Secp256r1::size_bits(), 256);
        assert_eq!(Secp256r1::size_bytes(), 32);
    }

    #[test]
    fn secp384r1_size() {
        assert_eq!(Secp384r1::size_bits(), 384);
        assert_eq!(Secp384r1::size_bytes(), 48);
    }

    #[test]
    fn ed25519_size() {
        assert_eq!(Ed25519::size_bits(), 256);
        assert_eq!(Ed25519::size_bytes(), 32);
    }

    #[test]
    fn pallas_size() {
        assert_eq!(Pallas::size_bits(), 255);
        assert_eq!(Pallas::size_bytes(), 32);
    }

    // ------------------------------------------------------------------
    // Domain parameter retrieval (raw bytes)
    // ------------------------------------------------------------------

    #[test]
    fn secp256k1_domain_order() {
        let n = Secp256k1::size_bytes();
        let mut buf = [0u8; 32];
        Secp256k1::domain_parameter(CurveDomainParam::Order, &mut buf[..n]).map_err(err)?;
        // secp256k1 order starts with 0xFF..
        assert_eq!(buf[0], 0xFF);
    }

    #[test]
    fn secp256k1_generator_raw() {
        let n = Secp256k1::size_bytes();
        let mut gx = [0u8; 32];
        let mut gy = [0u8; 32];
        Secp256k1::generator(&mut gx[..n], &mut gy[..n]).map_err(err)?;
        // Generator x starts with 0x79 for secp256k1
        assert_eq!(gx[0], 0x79);
    }

    // ------------------------------------------------------------------
    // EcPoint alloc + generator_bn
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_alloc_generator_on_curve() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;

        // Generator must be on the curve
        let on_curve = g.is_on_curve().map_err(err)?;
        assert_eq!(on_curve, true);

        // Generator must not be infinity
        let at_inf = g.is_at_infinity().map_err(err)?;
        assert_eq!(at_inf, false);
    }

    // ------------------------------------------------------------------
    // Init from raw bytes + export round-trip
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_init_export_roundtrip() {
        let n = Secp256k1::size_bytes();
        let mut gx_orig = [0u8; 32];
        let mut gy_orig = [0u8; 32];
        Secp256k1::generator(&mut gx_orig[..n], &mut gy_orig[..n]).map_err(err)?;

        let _lock = BnLock::acquire(32).map_err(err)?;
        let mut p = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        p.init(&gx_orig[..n], &gy_orig[..n]).map_err(err)?;

        let mut gx_out = [0u8; 32];
        let mut gy_out = [0u8; 32];
        p.export(&mut gx_out[..n], &mut gy_out[..n]).map_err(err)?;

        assert_eq!(gx_out, gx_orig);
        assert_eq!(gy_out, gy_orig);
    }

    // ------------------------------------------------------------------
    // Compress / decompress round-trip
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_compress_decompress() {
        let n = Secp256k1::size_bytes();
        let _lock = BnLock::acquire(32).map_err(err)?;

        // Start with the generator
        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;

        // Compress
        let mut compressed = [0u8; 32];
        let sign = g.compress(&mut compressed[..n]).map_err(err)?;

        // Decompress into a new point
        let mut p = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        p.decompress(&compressed[..n], sign).map_err(err)?;

        // The decompressed point must equal the original
        let equal = g.cmp(&p).map_err(err)?;
        assert_eq!(equal, true);
    }

    // ------------------------------------------------------------------
    // Point comparison
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_cmp_equal() {
        let _lock = BnLock::acquire(32).map_err(err)?;

        let mut g1 = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g1).map_err(err)?;

        let mut g2 = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g2).map_err(err)?;

        assert_eq!(g1.cmp(&g2).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Scalar multiplication: 1·G == G
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_scalarmul_identity() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let n = Secp256k1::size_bytes();

        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;

        // Keep a copy of G for comparison
        let mut g_copy = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g_copy).map_err(err)?;

        // scalar = 1 (big-endian, 32 bytes)
        let mut one = [0u8; 32];
        one[n - 1] = 1;
        g.scalarmul(&one[..n]).map_err(err)?;

        assert_eq!(g.cmp(&g_copy).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Scalar multiplication with BN: 2·G
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_scalarmul_bn_double() {
        let _lock = BnLock::acquire(32).map_err(err)?;

        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;

        // scalar = 2 as BN
        let k = Bn::alloc(32).map_err(err)?;
        k.set_u32(2).map_err(err)?;
        g.scalarmul_bn(k).map_err(err)?;

        // Verify result is on curve and not at infinity
        assert_eq!(g.is_on_curve().map_err(err)?, true);
        assert_eq!(g.is_at_infinity().map_err(err)?, false);
    }

    // ------------------------------------------------------------------
    // Point addition: G + G == 2·G
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_add_equals_double() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let n = Secp256k1::size_bytes();

        // Compute 2·G via scalarmul
        let mut two_g_scalar = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut two_g_scalar).map_err(err)?;
        let mut two = [0u8; 32];
        two[n - 1] = 2;
        two_g_scalar.scalarmul(&two[..n]).map_err(err)?;

        // Compute G + G via add
        let mut g1 = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g1).map_err(err)?;
        let mut g2 = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g2).map_err(err)?;

        let mut sum = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        sum.add(&g1, &g2).map_err(err)?;

        assert_eq!(sum.cmp(&two_g_scalar).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Point negation: neg produces a valid on-curve point
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_neg_on_curve() {
        let _lock = BnLock::acquire(32).map_err(err)?;

        let mut neg_g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut neg_g).map_err(err)?;
        neg_g.neg().map_err(err)?;

        // Negated generator must still be on the curve
        assert_eq!(neg_g.is_on_curve().map_err(err)?, true);
        // … but different from the original generator
        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;
        assert_eq!(neg_g.cmp(&g).map_err(err)?, false);
    }

    // ------------------------------------------------------------------
    // Ed25519 generator export
    // ------------------------------------------------------------------

    #[test]
    fn ed25519_generator_export() {
        let n = Ed25519::size_bytes();
        let _lock = BnLock::acquire(32).map_err(err)?;
        let mut g = EcPoint::new(CurvesId::Ed25519).map_err(err)?;
        Ed25519::generator_bn(&mut g).map_err(err)?;

        // Export and verify coordinates are non-zero
        let mut gx = [0u8; 32];
        let mut gy = [0u8; 32];
        g.export(&mut gx[..n], &mut gy[..n]).map_err(err)?;
        // Ed25519 generator Gy starts with 0x66 (big-endian)
        assert_eq!(gy[0], 0x66);
    }

    // ------------------------------------------------------------------
    // Secp256r1 generator on curve
    // ------------------------------------------------------------------

    #[test]
    fn secp256r1_generator_on_curve() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let mut g = EcPoint::new(CurvesId::Secp256r1).map_err(err)?;
        Secp256r1::generator_bn(&mut g).map_err(err)?;

        assert_eq!(g.is_on_curve().map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Domain parameter BN retrieval
    // ------------------------------------------------------------------

    #[test]
    fn secp256k1_domain_parameter_bn_order() {
        let _lock = BnLock::acquire(32).map_err(err)?;

        // Get the order as raw bytes
        let n = Secp256k1::size_bytes();
        let mut order_bytes = [0u8; 32];
        Secp256k1::domain_parameter(CurveDomainParam::Order, &mut order_bytes[..n]).map_err(err)?;

        // Get the order as a BN and export it
        let order_bn = Bn::alloc(32).map_err(err)?;
        Secp256k1::domain_parameter_bn(CurveDomainParam::Order, order_bn.raw()).map_err(err)?;
        let mut order_bn_bytes = [0u8; 32];
        order_bn.export(&mut order_bn_bytes[..n]).map_err(err)?;

        // Both should match
        assert_eq!(order_bytes, order_bn_bytes);
    }

    // ------------------------------------------------------------------
    // Randomised scalar multiplication produces valid point
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_rnd_scalarmul_on_curve() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let n = Secp256k1::size_bytes();

        let mut g = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut g).map_err(err)?;

        // scalar = 42
        let mut k = [0u8; 32];
        k[n - 1] = 42;
        g.rnd_scalarmul(&k[..n]).map_err(err)?;

        assert_eq!(g.is_on_curve().map_err(err)?, true);
        assert_eq!(g.is_at_infinity().map_err(err)?, false);
    }

    // ------------------------------------------------------------------
    // Double scalar mul: k·G + r·G == (k+r)·G
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_double_scalarmul() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let n = Secp256k1::size_bytes();

        // k = 3, r = 5, so result should equal 8·G
        let mut k = [0u8; 32];
        k[n - 1] = 3;
        let mut r = [0u8; 32];
        r[n - 1] = 5;

        let mut p = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut p).map_err(err)?;
        let mut q = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut q).map_err(err)?;

        let mut result = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        result
            .double_scalarmul(&mut p, &mut q, &k[..n], &r[..n])
            .map_err(err)?;

        // Compute 8·G directly
        let mut eight = [0u8; 32];
        eight[n - 1] = 8;
        let mut expected = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        Secp256k1::generator_bn(&mut expected).map_err(err)?;
        expected.scalarmul(&eight[..n]).map_err(err)?;

        assert_eq!(result.cmp(&expected).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // EcPoint curve() accessor
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_curve_accessor() {
        let _lock = BnLock::acquire(32).map_err(err)?;
        let p = EcPoint::new(CurvesId::Secp256k1).map_err(err)?;
        assert_eq!(p.curve() as u8, CurvesId::Secp256k1 as u8);
    }
}
