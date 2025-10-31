#![no_std]
#![no_main]

use ledger_device_sdk::tag_to_flag_u64;
use ledger_device_sdk::tlv::tlv_generic::{
    parse, Handler, ParseCfg, Received, Result, Tag, TlvData,
};

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

// Tags
const TAG_0: Tag = 0x01;
const TAG_1: Tag = 0x02;
const TAG_2: Tag = 0x03;

tag_to_flag_u64!(TAG_0, TAG_1, TAG_2);

// Output type
#[derive(Default, Debug)]
struct Out {
    a: Vec<u8>,
    b: bool,
}

// Handlers
fn on_a(d: &TlvData<'_>, out: &mut Out) -> Result<bool> {
    out.a = d.as_bytes().to_vec();
    Ok(true)
}
fn on_b(_d: &TlvData<'_>, out: &mut Out) -> Result<bool> {
    out.b = true;
    Ok(true)
}

fn on_common(_d: &TlvData<'_>, _out: &mut Out) -> Result<bool> {
    Ok(true)
}

// Static handler table
static HANDLERS: &[Handler<Out>] = &[
    Handler {
        tag: TAG_0,
        unique: true,
        func: Some(on_a),
    },
    Handler {
        tag: TAG_1,
        unique: true,
        func: Some(on_b),
    },
    Handler {
        tag: TAG_2,
        unique: false,
        func: None,
    }, // accept & ignore
];

#[no_mangle]
extern "C" fn sample_main() {
    let payload: &[u8] = &[0x01, 0x03, b'a', b'b', b'c', 0x02, 0x01, b'x'];

    let mut out = Out::default();

    let mut received = Received::new(tag_to_flag_u64);

    let mut cfg = ParseCfg::new(HANDLERS);
    cfg.common = Some(on_common);

    parse(&cfg, payload, &mut out, &mut received).unwrap();

    // Check that mandatory TAGs were received
    let mandatory_tags = tag_to_flag_u64(TAG_0) | tag_to_flag_u64(TAG_1);
    if received.flags & mandatory_tags != mandatory_tags {
        ledger_device_sdk::testing::debug_print("Received flags: ");
        ledger_device_sdk::testing::debug_print(format!("{:x}\n", received.flags).as_str());
        ledger_device_sdk::testing::debug_print("Missing mandatory tag\n");
        ledger_device_sdk::exit_app(1);
    }
    ledger_device_sdk::exit_app(0);
}
