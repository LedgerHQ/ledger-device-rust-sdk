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
/// on drop.  The BN lock is acquired transparently on the first
/// allocation and released when the last handle is dropped.
pub struct EcPoint {
    inner: cx_ecpoint_t,
}

impl EcPoint {
    /// Allocate a new EC point on the given curve.
    /// # Arguments
    /// * `curve` - The identifier of the elliptic curve to use for this point
    /// # Returns
    /// Returns a new `EcPoint` instance on success, or a `CxError` if allocation fails (e.g. invalid curve or BN context not locked).
    pub fn new(curve: CurvesId) -> Result<Self, CxError> {
        bn_retain(curve.size_bytes())?;
        let mut point = cx_ecpoint_t::default();
        let err = unsafe { cx_ecpoint_alloc(&mut point, curve as u8) };
        if err != CX_OK {
            bn_release();
            return Err(err.into());
        }
        Ok(Self { inner: point })
    }

    /// Curve identifier of this point.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to query
    /// # Returns
    /// Returns the `CurvesId` corresponding to the curve of this point.
    pub fn curve(&self) -> CurvesId {
        self.inner.curve.into()
    }

    /// Initialize the point from raw byte coordinates.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to initialize
    /// * `x` - The x-coordinate as a byte slice
    /// * `y` - The y-coordinate as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if initialization fails.
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
    /// # Arguments
    /// * `self` - The `EcPoint` instance to initialize
    /// * `x` - The x-coordinate as a BN handle
    /// * `y` - The y-coordinate as a BN handle
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if initialization fails.
    pub fn init_bn(&mut self, x: Bn, y: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_init_bn(&mut self.inner, x.raw(), y.raw()));
        Ok(())
    }

    /// Export the point coordinates to raw byte buffers.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to export
    /// * `x` - The buffer to receive the x-coordinate
    /// * `y` - The buffer to receive the y-coordinate
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if export fails.
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
    /// # Arguments
    /// * `self` - The `EcPoint` instance to export
    /// * `x` - The BN handle to receive the x-coordinate
    /// * `y` - The BN handle to receive the y-coordinate
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if export fails.
    pub fn export_bn(&self, x: &mut Bn, y: &mut Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_export_bn(&self.inner, x.raw_mut(), y.raw_mut()));
        Ok(())
    }

    /// Compress the point into a byte buffer.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to compress
    /// * `xy_compressed` - The buffer to receive the compressed point
    /// # Returns
    /// Returns the sign of the y-coordinate on success, or a `CxError` if compression fails.
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
    /// # Arguments
    /// * `self` - The `EcPoint` instance to initialize with the decompressed point
    /// * `xy_compressed` - The compressed point data as a byte slice
    /// * `sign` - The sign of the y-coordinate (as returned by `compress`)
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if decompression fails (e.g. invalid compressed data or sign).
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
    /// # Arguments
    /// * `self` - The `EcPoint` instance to store the result
    /// * `p` - The first point
    /// * `q` - The second point
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if addition fails.
    pub fn add(&mut self, p: &EcPoint, q: &EcPoint) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_add(&mut self.inner, &p.inner, &q.inner));
        Ok(())
    }

    /// Negate this point in place.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to negate
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if negation fails.
    pub fn neg(&mut self) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_neg(&mut self.inner));
        Ok(())
    }

    /// Scalar multiplication: `self = k · self` (scalar as raw bytes).
    /// # Arguments
    /// * `self` - The `EcPoint` instance to multiply
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
    pub fn scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_scalarmul(&mut self.inner, k.as_ptr(), k.len()));
        Ok(())
    }

    /// Scalar multiplication: `self = k · self` (scalar as BN handle).
    /// # Arguments
    /// * `self` - The `EcPoint` instance to multiply
    /// * `bn_k` - The scalar as a BN handle
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
    pub fn scalarmul_bn(&mut self, bn_k: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_scalarmul_bn(&mut self.inner, bn_k.raw()));
        Ok(())
    }

    /// Randomised scalar multiplication (side-channel resistant).
    /// # Arguments
    /// * `self` - The `EcPoint` instance to multiply
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
    pub fn rnd_scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_scalarmul(
            &mut self.inner,
            k.as_ptr(),
            k.len()
        ));
        Ok(())
    }

    /// Randomised scalar multiplication with BN scalar.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to multiply
    /// * `bn_k` - The scalar as a BN handle
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
    pub fn rnd_scalarmul_bn(&mut self, bn_k: Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_scalarmul_bn(&mut self.inner, bn_k.raw()));
        Ok(())
    }

    /// Randomised fixed-point scalar multiplication (side-channel resistant).
    /// # Arguments
    /// * `self` - The `EcPoint` instance to multiply
    /// * `k` - The scalar as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
    pub fn rnd_fixed_scalarmul(&mut self, k: &[u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecpoint_rnd_fixed_scalarmul(
            &mut self.inner,
            k.as_ptr(),
            k.len()
        ));
        Ok(())
    }

    /// Double scalar multiplication: `self = k·P + r·Q` (raw-byte scalars).
    /// # Arguments
    /// * `self` - The `EcPoint` instance to store the result
    /// * `p` - The first point P
    /// * `q` - The second point Q
    /// * `k` - The scalar k as a byte slice
    /// * `r` - The scalar r as a byte slice
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
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
    /// # Arguments
    /// * `self` - The `EcPoint` instance to store the result
    /// * `p` - The first point P
    /// * `q` - The second point Q
    /// * `bn_k` - The scalar k as a BN handle
    /// * `bn_r` - The scalar r as a BN handle
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if multiplication fails.
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
    /// # Arguments
    /// * `self` - The first `EcPoint` instance
    /// * `other` - The second `EcPoint` instance
    /// # Returns
    /// Returns `Ok(true)` if the points are equal, `Ok(false)` if they are not, or a `CxError` if the comparison fails.
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, other: &EcPoint) -> Result<bool, CxError> {
        let mut is_equal = false;
        check_cx_ok!(cx_ecpoint_cmp(&self.inner, &other.inner, &mut is_equal));
        Ok(is_equal)
    }

    /// Check whether this point lies on its curve.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to check
    /// # Returns
    /// Returns `Ok(true)` if the point is on the curve, `Ok(false)` if it is not, or a `CxError` if the check fails.
    pub fn is_on_curve(&self) -> Result<bool, CxError> {
        let mut on_curve = false;
        check_cx_ok!(cx_ecpoint_is_on_curve(&self.inner, &mut on_curve));
        Ok(on_curve)
    }

    /// Check whether this point is the point at infinity.
    /// # Arguments
    /// * `self` - The `EcPoint` instance to check
    /// # Returns
    /// Returns `Ok(true)` if the point is at infinity, `Ok(false)` if it is not, or a `CxError` if the check fails.
    pub fn is_at_infinity(&self) -> Result<bool, CxError> {
        let mut at_infinity = false;
        check_cx_ok!(cx_ecpoint_is_at_infinity(&self.inner, &mut at_infinity));
        Ok(at_infinity)
    }

    /// Access the inner `cx_ecpoint_t`.
    #[allow(dead_code)]
    fn as_raw(&self) -> &cx_ecpoint_t {
        &self.inner
    }

    /// Access the inner `cx_ecpoint_t` mutably.
    fn as_raw_mut(&mut self) -> &mut cx_ecpoint_t {
        &mut self.inner
    }
}

impl Drop for EcPoint {
    fn drop(&mut self) {
        unsafe {
            cx_ecpoint_destroy(&mut self.inner);
        }
        bn_release();
    }
}

/// Implementations specific to the `CurvesId` enum itself (not tied to a specific curve type).
impl CurvesId {
    /// Curve size in **bits**.
    pub fn size_bits(&self) -> usize {
        let mut length = 0usize;
        unsafe {
            cx_ecdomain_size(u8::from(*self), &mut length);
        }
        length
    }

    /// Curve field-element size in **bytes**.
    pub fn size_bytes(&self) -> usize {
        let mut length = 0usize;
        unsafe {
            cx_ecdomain_parameters_length(u8::from(*self), &mut length);
        }
        length
    }

    /// Retrieve a specific domain parameter as raw bytes.
    pub fn domain_parameter(&self, id: CurveDomainParam, buf: &mut [u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecdomain_parameter(
            u8::from(*self),
            id as u8,
            buf.as_mut_ptr(),
            buf.len() as u32,
        ));
        Ok(())
    }

    /// Retrieve a specific domain parameter as a BN handle.
    /// Requires the BN context to be locked.
    /// # Arguments
    /// * `id` - The domain parameter identifier
    /// * `bn` - The BN handle to store the parameter
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the retrieval fails.
    pub fn domain_parameter_bn(&self, id: CurveDomainParam, bn: &mut Bn) -> Result<(), CxError> {
        check_cx_ok!(cx_ecdomain_parameter_bn(
            u8::from(*self),
            id as u8,
            *bn.raw_mut()
        ));
        Ok(())
    }

    /// Retrieve the generator point as raw bytes (`Gx`, `Gy`).
    /// Both buffers must have the same length (the field-element size).
    /// # Arguments
    /// * `gx` - The buffer to receive the x-coordinate of the generator
    /// * `gy` - The buffer to receive the y-coordinate of the generator
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the retrieval fails.
    pub fn generator(&self, gx: &mut [u8], gy: &mut [u8]) -> Result<(), CxError> {
        check_cx_ok!(cx_ecdomain_generator(
            u8::from(*self),
            gx.as_mut_ptr(),
            gy.as_mut_ptr(),
            gx.len(),
        ));
        Ok(())
    }

    /// Retrieve the generator point as an `EcPoint` (BN-based).
    /// Requires the BN context to be locked.
    /// # Arguments
    /// * `p` - The `EcPoint` instance to initialize with the generator point
    /// # Returns
    /// Returns `Ok(())` on success, or a `CxError` if the retrieval fails.
    pub fn generator_bn(&self, p: &mut EcPoint) -> Result<(), CxError> {
        check_cx_ok!(cx_ecdomain_generator_bn(u8::from(*self), p.as_raw_mut(),));
        Ok(())
    }
}

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
        let curve = CurvesId::Secp256k1;
        assert_eq!(curve.size_bits(), 256);
        assert_eq!(curve.size_bytes(), 32);
    }

    #[test]
    fn secp256r1_size() {
        let curve = CurvesId::Secp256r1;
        assert_eq!(curve.size_bits(), 256);
        assert_eq!(curve.size_bytes(), 32);
    }

    #[test]
    fn secp384r1_size() {
        let curve = CurvesId::Secp384r1;
        assert_eq!(curve.size_bits(), 384);
        assert_eq!(curve.size_bytes(), 48);
    }

    #[test]
    fn ed25519_size() {
        let curve = CurvesId::Ed25519;
        assert_eq!(curve.size_bits(), 256);
        assert_eq!(curve.size_bytes(), 32);
    }

    #[test]
    fn pallas_size() {
        let curve = CurvesId::Pallas;
        assert_eq!(curve.size_bits(), 255);
        assert_eq!(curve.size_bytes(), 32);
    }

    // ------------------------------------------------------------------
    // Domain parameter retrieval (raw bytes)
    // ------------------------------------------------------------------

    #[test]
    fn secp256k1_domain_order() {
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();
        let mut buf = [0u8; 32];
        curve
            .domain_parameter(CurveDomainParam::Order, &mut buf[..n])
            .map_err(err)?;
        // secp256k1 order starts with 0xFF..
        assert_eq!(buf[0], 0xFF);
    }

    #[test]
    fn secp256k1_generator_raw() {
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();
        let mut gx = [0u8; 32];
        let mut gy = [0u8; 32];
        curve.generator(&mut gx[..n], &mut gy[..n]).map_err(err)?;
        // Generator x starts with 0x79 for secp256k1
        assert_eq!(gx[0], 0x79);
    }

    // ------------------------------------------------------------------
    // EcPoint alloc + generator_bn
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_alloc_generator_on_curve() {
        let curve = CurvesId::Secp256k1;
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

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
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();
        let mut gx_orig = [0u8; 32];
        let mut gy_orig = [0u8; 32];
        curve
            .generator(&mut gx_orig[..n], &mut gy_orig[..n])
            .map_err(err)?;

        let mut p = EcPoint::new(curve).map_err(err)?;
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
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();

        // Start with the generator
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

        // Compress
        let mut compressed = [0u8; 32];
        let sign = g.compress(&mut compressed[..n]).map_err(err)?;

        // Decompress into a new point
        let mut p = EcPoint::new(curve).map_err(err)?;
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
        let curve = CurvesId::Secp256k1;
        let mut g1 = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g1).map_err(err)?;

        let mut g2 = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g2).map_err(err)?;

        assert_eq!(g1.cmp(&g2).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Scalar multiplication: 1·G == G
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_scalarmul_identity() {
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();

        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

        // Keep a copy of G for comparison
        let mut g_copy = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g_copy).map_err(err)?;

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
        let curve = CurvesId::Secp256k1;
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

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
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();

        // Compute 2·G via scalarmul
        let mut two_g_scalar = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut two_g_scalar).map_err(err)?;
        let mut two = [0u8; 32];
        two[n - 1] = 2;
        two_g_scalar.scalarmul(&two[..n]).map_err(err)?;

        // Compute G + G via add
        let mut g1 = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g1).map_err(err)?;
        let mut g2 = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g2).map_err(err)?;

        let mut sum = EcPoint::new(curve).map_err(err)?;
        sum.add(&g1, &g2).map_err(err)?;

        assert_eq!(sum.cmp(&two_g_scalar).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Point negation: neg produces a valid on-curve point
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_neg_on_curve() {
        let curve = CurvesId::Secp256k1;
        let mut neg_g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut neg_g).map_err(err)?;
        neg_g.neg().map_err(err)?;

        // Negated generator must still be on the curve
        assert_eq!(neg_g.is_on_curve().map_err(err)?, true);
        // … but different from the original generator
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;
        assert_eq!(neg_g.cmp(&g).map_err(err)?, false);
    }

    // ------------------------------------------------------------------
    // Ed25519 generator export
    // ------------------------------------------------------------------

    #[test]
    fn ed25519_generator_export() {
        let curve = CurvesId::Ed25519;
        let n = curve.size_bytes();
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

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
        let curve = CurvesId::Secp256r1;
        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

        assert_eq!(g.is_on_curve().map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // Domain parameter BN retrieval
    // ------------------------------------------------------------------

    #[test]
    fn secp256k1_domain_parameter_bn_order() {
        let curve = CurvesId::Secp256k1;
        // Get the order as raw bytes
        let n = curve.size_bytes();
        let mut order_bytes = [0u8; 32];
        curve
            .domain_parameter(CurveDomainParam::Order, &mut order_bytes[..n])
            .map_err(err)?;

        // Get the order as a BN and export it
        let mut order_bn = Bn::alloc(32).map_err(err)?;
        curve
            .domain_parameter_bn(CurveDomainParam::Order, &mut order_bn)
            .map_err(err)?;
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
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();

        let mut g = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut g).map_err(err)?;

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
        let curve = CurvesId::Secp256k1;
        let n = curve.size_bytes();

        // k = 3, r = 5, so result should equal 8·G
        let mut k = [0u8; 32];
        k[n - 1] = 3;
        let mut r = [0u8; 32];
        r[n - 1] = 5;

        let mut p = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut p).map_err(err)?;
        let mut q = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut q).map_err(err)?;

        let mut result = EcPoint::new(curve).map_err(err)?;
        result
            .double_scalarmul(&mut p, &mut q, &k[..n], &r[..n])
            .map_err(err)?;

        // Compute 8·G directly
        let mut eight = [0u8; 32];
        eight[n - 1] = 8;
        let mut expected = EcPoint::new(curve).map_err(err)?;
        curve.generator_bn(&mut expected).map_err(err)?;
        expected.scalarmul(&eight[..n]).map_err(err)?;

        assert_eq!(result.cmp(&expected).map_err(err)?, true);
    }

    // ------------------------------------------------------------------
    // EcPoint curve() accessor
    // ------------------------------------------------------------------

    #[test]
    fn ecpoint_curve_accessor() {
        let curve = CurvesId::Secp256k1;
        let p = EcPoint::new(curve).map_err(err)?;
        assert_eq!(p.curve() as u8, curve as u8);
    }
}
