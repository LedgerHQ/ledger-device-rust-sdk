#![no_std]
#![no_main]

use ledger_device_sdk::io::*;
use ledger_device_sdk::nbgl::{init_comm, NbglGenericSettings};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);

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

    // Display the settings screen with info.
    let settings_strings = [["Switch title", "Switch subtitle"]];
    let mut settings: settings::Settings = Default::default();
    let mut menu = NbglGenericSettings::new()
        .title("Settings and Info")
        .settings(settings.get_mut(), &settings_strings)
        .info(&[("Field 1", "Value 1"), ("Field 2", "Value 2")]);

    menu.show();

    // Display the settings screen with settings only.
    let settings_strings = [
        ["Switch title 1", "Switch subtitle 1"],
        ["Switch title 2", "Switch subtitle 2"],
        ["Switch title 3", "Switch subtitle 3"],
    ];
    let mut settings: settings::Settings = Default::default();
    let mut menu = NbglGenericSettings::new()
        .title("Only Settings")
        .settings(settings.get_mut(), &settings_strings);

    menu.show();

    ledger_secure_sdk_sys::exit_app(0);
}
