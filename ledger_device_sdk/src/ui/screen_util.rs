#![allow(dead_code)]

use ledger_secure_sdk_sys;

pub fn draw(x_pos: i32, y_pos: i32, w: u32, h: u32, inv: bool, bmp: &[u8]) {
    let inverted = [inv as u32, !inv as u32];
    unsafe {
        ledger_secure_sdk_sys::bagl_hal_draw_bitmap_within_rect(
            x_pos,
            y_pos,
            w,
            h,
            2,
            inverted.as_ptr(),
            1,
            bmp.as_ptr(),
            w * h,
        );
    }
}

pub fn fulldraw(x_pos: i32, y_pos: i32, bmp: &[u8]) {
    draw(x_pos, y_pos, 128, 64, false, bmp);
}

pub fn screen_update() {
    unsafe {
        ledger_secure_sdk_sys::screen_update();
    }
}

#[cfg(not(feature = "speculos"))]
pub fn seph_setup_ticker(interval_ms: u16) {
    let ms = interval_ms.to_be_bytes();
    ledger_secure_sdk_sys::seph::io_tx(0x01, &[0x4e, 0, 2, ms[0], ms[1]], 5);
}
