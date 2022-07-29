#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use nanos_sdk::ecc::{make_bip32_path, BrainpoolP384R1};

const PATH: [u32; 5] = make_bip32_path(b"m/44'/123'/0'/0/0");

fn sign_message_const(m: &[u8], path: &[u32]) -> Result<([u8; 104], u32), u32> {
    Ok(BrainpoolP384R1::from_bip32(path).deterministic_sign(m)?)
}

#[no_mangle]
fn sample_main() {
    let _signature = sign_message_const(b"Hello world", &PATH).unwrap();
}
