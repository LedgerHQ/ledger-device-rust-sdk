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
