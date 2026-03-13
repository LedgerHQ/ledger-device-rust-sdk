#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::{ApduHeader, StatusWords};
use ledger_device_sdk::nbgl::{NbglGlyph, NbglHomeAndSettings, init_comm};
use ledger_device_sdk::tlv::{TrustedNameOut, parse_trusted_name_tlv};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

pub enum Instruction {
    GetVersion = 0x01,
    GetAppName = 0x02,
    ParseTlv = 0x21,
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = StatusWords;
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match value.ins {
            1 => Ok(Instruction::GetVersion),
            2 => Ok(Instruction::GetAppName),
            0x21 => Ok(Instruction::ParseTlv),
            _ => Err(StatusWords::NothingReceived),
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn sample_main() {
    let comm = init_comm(&COMM);

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
            "Trusted Name Example App",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        )
        .tagline("Send APDU from apdu folder");

    home.show_and_return();

    loop {
        let cmd = comm.next_command();
        let ins = cmd.decode::<Instruction>();
        match ins {
            Ok(Instruction::GetVersion) => {
                ledger_device_sdk::log::info!("GetVersion");
                let version = [0, 1, 0]; // version 0.1.0
                let _ = cmd.reply(&version, StatusWords::Ok);
            }
            Ok(Instruction::GetAppName) => {
                let app_name = b"Trusted Name Example";
                let _ = cmd.reply(app_name, StatusWords::Ok);
            }
            Ok(Instruction::ParseTlv) => {
                ledger_device_sdk::log::info!("Starting TLV Parsing");
                let buffer = cmd.get_data();

                let mut out = TrustedNameOut::default();
                match parse_trusted_name_tlv(buffer, &mut out) {
                    Ok(()) => {
                        ledger_device_sdk::log::info!("TLV Parsing successful");
                        ledger_device_sdk::log::info!("{}", out.trusted_name.as_str());
                        ledger_device_sdk::log::info!("{}", out.address.as_str());
                        let _ = cmd.reply(&[], StatusWords::Ok);
                    }
                    Err(err) => {
                        ledger_device_sdk::log::info!("TLV Parsing failed");
                        let _ = cmd.reply(&[], err);
                    }
                }
            }
            Err(sw) => {
                let _ = cmd.reply(&[], sw);
            }
        }
    }
}
