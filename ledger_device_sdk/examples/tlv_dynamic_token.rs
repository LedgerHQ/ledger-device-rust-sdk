#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{NbglGlyph, NbglHomeAndSettings};
use ledger_device_sdk::tlv_uc::{parse_dynamic_token_tlv, DynamicTokenOut};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
pub enum Instruction {
    GetVersion = 0x01,
    GetAppName = 0x02,
    ParseTlv = 0x03,
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = StatusWords;
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match value.ins {
            1 => Ok(Instruction::GetVersion),
            2 => Ok(Instruction::GetAppName),
            3 => Ok(Instruction::ParseTlv),
            _ => Err(StatusWords::NothingReceived),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    let mut home = NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .infos(
            "Dynamic Token Example App",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        )
        .tagline("Send APDU from apdu folder");

    home.show_and_return();

    loop {
        let ins: Instruction = comm.next_command();
        match ins {
            Instruction::GetVersion => {
                ledger_device_sdk::testing::debug_print("GetVersion\n");
                let version = [0, 1, 0]; // version 0.1.0
                comm.append(&version);
                comm.reply_ok();
            }
            Instruction::GetAppName => {
                let app_name = b"Dynamic Token Example";
                comm.append(app_name);
                comm.reply_ok();
            }
            Instruction::ParseTlv => {
                ledger_device_sdk::testing::debug_print("Starting TLV Parsing\n");
                let buffer = match comm.get_data() {
                    Ok(buf) => buf,
                    Err(_err) => {
                        ledger_device_sdk::testing::debug_print("Failed to get data: {}\n");
                        break;
                    }
                };

                let mut out = DynamicTokenOut::default();
                match parse_dynamic_token_tlv(&buffer, &mut out) {
                    Ok(()) => {
                        ledger_device_sdk::testing::debug_print("TLV Parsing successful\n");
                        comm.reply_ok();
                    }
                    Err(err) => {
                        ledger_device_sdk::testing::debug_print("TLV Parsing failed\n");
                        comm.reply(err);
                    }
                }
            }
        }
    }
}
