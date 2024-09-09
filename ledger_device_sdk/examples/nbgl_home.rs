#![no_std]
#![no_main]

// Force boot section to be embedded in
use ledger_device_sdk as _;

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglGlyph, NbglHomeAndSettings};
use ledger_device_sdk::nvm::*;
use ledger_device_sdk::NVMData;
use ledger_secure_sdk_sys::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit_app(1);
}

pub enum Instruction {
    GetVersion,
    GetAppName,
}

impl TryFrom<ApduHeader> for Instruction {
    type Error = StatusWords;

    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match value.ins {
            3 => Ok(Instruction::GetVersion),
            4 => Ok(Instruction::GetAppName),
            _ => Err(StatusWords::NothingReceived),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    unsafe {
        nbgl_refreshReset();
    }

    const SETTINGS_SIZE: usize = 10;
    #[link_section = ".nvm_data"]
    static mut DATA: NVMData<AtomicStorage<[u8; SETTINGS_SIZE]>> =
        NVMData::new(AtomicStorage::new(&[0u8; 10]));

    let mut comm = Comm::new();
    // Initialize reference to Comm instance for NBGL
    // API calls.
    init_comm(&mut comm);

    // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));

    let settings_strings = [["Switch title", "Switch subtitle"]];
    // Display the home screen.
    NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .settings(unsafe { DATA.get_mut() }, &settings_strings)
        .infos(
            "Example App",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        )
        .show::<Instruction>();
}
