#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(ledger_device_sdk::testing::sdk_test_runner)]

use core::panic::PanicInfo;
use ledger_device_sdk::testing::{debug_print, test_panic, to_hex, TestType};
use testmacro::test_item as test;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic(info);
}

#[no_mangle]
extern "C" fn sample_main() {
    test_main();
    ledger_device_sdk::exit_app(0);
}

/// Integration tests: ECC module
use ledger_device_sdk::ecc::{make_bip32_path, Secp256k1, Secp256r1, SeedDerive, Stark256};

const PATH: [u32; 5] = make_bip32_path(b"m/44'/123'/0'/0/0");

#[test]
fn test_secp256k1() {
    let key: [u8; 32] = [0u8; 32];

    let _priv_key = Secp256k1::new();
    let _priv_key = Secp256k1::from(&key);
    let (priv_key, cc) = Secp256k1::derive_from_path(&PATH);

    let hash = b"Not your Keys, not your Coins";

    let signature = priv_key.deterministic_sign(hash).unwrap();

    let pub_key = priv_key.public_key().unwrap();

    assert!(pub_key.verify((&signature.0, signature.1), hash));
}

#[test]
fn test_secp256r1() {
    let key: [u8; 32] = [0u8; 32];

    let _priv_key = Secp256r1::new();
    let _priv_key = Secp256r1::from(&key);
    let (priv_key, cc) = Secp256r1::derive_from_path(&PATH);

    let hash = b"Not your Keys, not your Coins";

    let signature = priv_key.deterministic_sign(hash).unwrap();

    let pub_key = priv_key.public_key().unwrap();

    assert!(pub_key.verify((&signature.0, signature.1), hash));
}

#[test]
fn test_stark256() {
    let key: [u8; 32] = [0u8; 32];

    let _priv_key = Stark256::new();
    let _priv_key = Stark256::from(&key);
    let (priv_key, cc) = Stark256::derive_from_path(&PATH);

    let hash = b"Not your Keys, not your Coins";

    let signature = priv_key.deterministic_sign(hash).unwrap();

    let pub_key = priv_key.public_key().unwrap();

    assert!(pub_key.verify((&signature.0, signature.1), hash));
}
