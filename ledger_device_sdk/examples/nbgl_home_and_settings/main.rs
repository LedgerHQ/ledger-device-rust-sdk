#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglGlyph, NbglHomeAndSettings};

mod settings;
use settings::Settings;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

#[repr(u16)]
#[derive(Clone, Copy, PartialEq)]
pub enum AppSW {
    Deny = 0x6985,
    WrongP1P2 = 0x6A86,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    TxDisplayFail = 0xB001,
    AddrDisplayFail = 0xB002,
    TxWrongLength = 0xB004,
    TxParsingFail = 0xB005,
    TxHashFail = 0xB006,
    TxSignFail = 0xB008,
    KeyDeriveFail = 0xB009,
    VersionParsingFail = 0xB00A,
    WrongApduLength = StatusWords::BadLen as u16,
    Ok = 0x9000,
}

impl From<AppSW> for Reply {
    fn from(sw: AppSW) -> Reply {
        Reply(sw as u16)
    }
}

pub enum Instruction {
    GetVersion,
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = AppSW;
     fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match (value.ins, value.p1, value.p2) {
            (1, 0, 0) => Ok(Instruction::GetVersion),
            (_, _, _) => Err(AppSW::InsNotSupported),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    
    let mut comm = Comm::new();

    init_comm(&mut comm);

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("./examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("./examples/crab_16x16.gif", NBGL));

    let settings_strings = [["Display Memo", "Allow display of transaction memo."]];
    let mut settings: Settings = Default::default();

    let mut home = NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .settings(settings.get_mut(), &settings_strings)
        .infos(
            "Example App",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        );

    // Display the home screen.
    home.show_and_return();

     loop {
        let _ins: Instruction = comm.next_command();
    }

}
