unsafe extern "C" {
    fn screen_clear();
    fn screen_update();
    fn screen_set_keepout(x: u32, y: u32, width: u32, height: u32);
    fn bagl_hal_draw_bitmap_within_rect(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color_count: u32,
        colors: *const u32,
        bit_per_pixel: u32,
        bitmap: *const u8,
        bitmap_length_bits: u32,
    );
    fn bagl_hal_draw_rect(color: u32, x: i32, y: i32, width: u32, height: u32);
}

pub fn sdk_screen_clear() {
    unsafe {
        screen_clear();
    }
}

pub fn sdk_set_keepout(x: u32, y: u32, width: u32, height: u32) {
    unsafe {
        screen_set_keepout(x, y, width, height);
    }
}

pub fn sdk_screen_update() {
    unsafe {
        screen_update();
    }
}

/// Number of pixels of a `width` x `height` 1-bit-per-pixel bitmap, if
/// `bitmap_len` bytes hold all of them.
pub(crate) fn bitmap_bits(width: u32, height: u32, bitmap_len: usize) -> Option<u32> {
    let bits = width.checked_mul(height)?;
    let bytes = usize::try_from(bits.div_ceil(8)).ok()?;
    (bytes <= bitmap_len).then_some(bits)
}

/// Draws a 1-bit-per-pixel bitmap. Nothing is drawn if `bitmap` is shorter
/// than `width * height` bits.
pub fn sdk_bagl_hal_draw_bitmap_within_rect(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    inverted: bool,
    bitmap: &[u8],
) {
    let Some(bits) = bitmap_bits(width, height, bitmap.len()) else {
        return;
    };
    let inverted = [inverted as u32, !inverted as u32];
    unsafe {
        bagl_hal_draw_bitmap_within_rect(
            x,
            y,
            width,
            height,
            2,
            inverted.as_ptr(),
            1,
            bitmap.as_ptr(),
            bits,
        )
    }
}

pub fn sdk_bagl_hal_draw_rect(color: u32, x: i32, y: i32, width: u32, height: u32) {
    unsafe {
        bagl_hal_draw_rect(color, x, y, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    #[test]
    fn bitmap_bits_requires_enough_bytes() {
        assert_eq!(bitmap_bits(128, 64, 1024), Some(8192));
        assert_eq!(bitmap_bits(128, 64, 1023), None);
        assert_eq!(bitmap_bits(3, 3, 2), Some(9));
        assert_eq!(bitmap_bits(3, 3, 1), None);
        assert_eq!(bitmap_bits(0, 64, 0), Some(0));
    }

    #[test]
    fn bitmap_bits_rejects_overflowing_dimensions() {
        assert_eq!(bitmap_bits(u32::MAX, 2, usize::MAX), None);
        assert_eq!(bitmap_bits(0x10000, 0x10000, usize::MAX), None);
    }
}
