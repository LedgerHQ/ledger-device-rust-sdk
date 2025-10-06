#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglGlyph, NbglHomeAndSettings};
// use ledger_device_sdk::nvm::*;
// use ledger_device_sdk::NVMData;

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

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

mod settings {

    use ledger_device_sdk::nvm::*;
    use ledger_device_sdk::NVMData;

    // This is necessary to store the object in NVM and not in RAM
    const SETTINGS_SIZE: usize = 10;
    #[link_section = ".nvm_data"]
    static mut DATA: NVMData<AtomicStorage<[u8; SETTINGS_SIZE]>> =
        NVMData::new(AtomicStorage::new(&[0u8; SETTINGS_SIZE]));

    #[derive(Clone, Copy)]
    pub struct Settings;

    impl Default for Settings {
        fn default() -> Self {
            Settings
        }
    }

    impl Settings {
        #[inline(never)]
        #[allow(unused)]
        pub fn get_mut(&mut self) -> &mut AtomicStorage<[u8; SETTINGS_SIZE]> {
            let data = &raw mut DATA;
            unsafe { (*data).get_mut() }
        }

        #[inline(never)]
        #[allow(unused)]
        pub fn get_ref(&mut self) -> &AtomicStorage<[u8; SETTINGS_SIZE]> {
            let data = &raw const DATA;
            unsafe { (*data).get_ref() }
        }

        #[allow(unused)]
        pub fn get_element(&self, index: usize) -> u8 {
            let data = &raw const DATA;
            let storage = unsafe { (*data).get_ref() };
            let settings = storage.get_ref();
            settings[index]
        }

        #[allow(unused)]
        // Not used in this boilerplate, but can be used to set a value in the settings
        pub fn set_element(&self, index: usize, value: u8) {
            let data = &raw mut DATA;
            let storage = unsafe { (*data).get_mut() };
            let mut updated_data = *storage.get_ref();
            updated_data[index] = value;
            unsafe {
                storage.update(&updated_data);
            }
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = Comm::new();
    init_comm(&mut comm);

    #[cfg(target_os = "apex_p")]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_48x48.png", NBGL));
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_64x64.gif", NBGL));
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    const FERRIS: NbglGlyph =
        NbglGlyph::from_include(include_gif!("examples/crab_14x14.png", NBGL));

    let settings_strings = [["Switch title", "Switch subtitle"]];
    let mut settings: settings::Settings = Default::default();

    // Display the home screen.
    let mut home = NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .settings(settings.get_mut(), &settings_strings)
        .infos(
            "Example App",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
        );

    home.show_and_return();

    loop {
        let _ins: Instruction = comm.next_command();
    }
}
