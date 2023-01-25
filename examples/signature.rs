#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

use nanos_sdk::ecc::{make_bip32_path, Secp256r1, SeedDerive};

const PATH: [u32; 5] = make_bip32_path(b"m/44'/123'/0'/0/0");

fn sign_message_const(m: &[u8], path: &[u32]) -> Result<([u8; 72], u32, u32), u32> {
    Ok(Secp256r1::derive_from_path(path).deterministic_sign(m)?)
}

#[no_mangle]
fn sample_main() {
    let _signature = sign_message_const(b"Hello world", &PATH).unwrap();
}
