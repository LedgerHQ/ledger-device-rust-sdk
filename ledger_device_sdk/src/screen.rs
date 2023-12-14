pub fn sdk_screen_clear() {
    unsafe {
        ledger_secure_sdk_sys::screen_clear();
    }
}

pub fn sdk_set_keepout(x: u32, y: u32, width: u32, height: u32) {
    unsafe {
        ledger_secure_sdk_sys::screen_set_keepout(x, y, width, height);
    }
}

pub fn sdk_screen_update() {
    unsafe {
        ledger_secure_sdk_sys::screen_update();
    }
}

pub fn sdk_bagl_hal_draw_bitmap_within_rect(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    inverted: bool,
    bitmap: &[u8],
) {
    let inverted = [inverted as u32, !inverted as u32];
    unsafe {
        ledger_secure_sdk_sys::bagl_hal_draw_bitmap_within_rect(
            x,
            y,
            width,
            height,
            2,
            inverted.as_ptr(),
            1,
            bitmap.as_ptr(),
            width * height,
        )
    }
}

pub fn sdk_bagl_hal_draw_rect(color: u32, x: i32, y: i32, width: u32, height: u32) {
    unsafe {
        ledger_secure_sdk_sys::bagl_hal_draw_rect(color, x, y, width, height);
    }
}
