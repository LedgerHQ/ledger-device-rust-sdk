#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::hash::HashInit;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{NbglAction, NbglGlyph};
use ledger_device_sdk::ecc::CurvesId;
use ledger_device_sdk::pki::pki_verify_data;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
pub enum Instruction {
    GetVersion = 0x01,
    GetAppName = 0x02,
    CheckPki = 0x03
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = StatusWords;
    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match value.ins {
            1 => Ok(Instruction::GetVersion),
            2 => Ok(Instruction::GetAppName),
            3 => Ok(Instruction::CheckPki),
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

    // Create NBGL action
    let _action = NbglAction::new()
        .message("Press Stop to exit")
        .action_text("Stop")
        .glyph(&FERRIS);
    
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
                let app_name = b"PKI Example";
                comm.append(app_name);
                comm.reply_ok();
            }
            Instruction::CheckPki => {
                ledger_device_sdk::testing::debug_print("Starting PKI Check\n");
                let buffer = match comm.get_data() {
                    Ok(buf) => buf,
                    Err(_err) => {
                        ledger_device_sdk::testing::debug_print("Failed to get data: {}\n");
                        break;
                    }
                };

                let mut data = [0u8; 80];
                data.copy_from_slice(&buffer[..80]);

                let mut hasher = ledger_device_sdk::hash::sha2::Sha2_256::new();
                let mut hash_data = [0u8; 32];
                let _ = hasher.hash(&data[..], &mut hash_data);

                let mut signature = [0u8; 72];
                signature.copy_from_slice(&buffer[82..]); 
                
                match pki_verify_data(&mut hash_data[..], 8u8, CurvesId::Secp256k1, signature.as_mut_slice()) {
                    Ok(()) => {
                        ledger_device_sdk::testing::debug_print("PKI Check successful\n");
                        comm.reply_ok();
                    },
                    Err(err) => {
                        ledger_device_sdk::testing::debug_print("PKI Check failed\n");
                        comm.reply(err);
                    },
                }
            }
        }
    }
}