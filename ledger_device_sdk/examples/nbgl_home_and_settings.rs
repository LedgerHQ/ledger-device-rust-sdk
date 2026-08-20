#![no_std]
#![no_main]

use include_gif::include_gif;
use ledger_device_sdk::nbgl::{HomeAction, NbglGlyph, NbglHomeAndSettings, NbglStatus, init_comm};
// use ledger_device_sdk::nvm::*;
// use ledger_device_sdk::NVMData;
use ledger_device_sdk::io::{ApduHeader, Comm, DEFAULT_BUF_SIZE, StatusWords};

ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
ledger_device_sdk::define_comm!(COMM);

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

    use ledger_device_sdk::NVMData;
    use ledger_device_sdk::nvm::*;

    // This is necessary to store the object in NVM and not in RAM
    const SETTINGS_SIZE: usize = 10;
    #[unsafe(link_section = ".nvm_data")]
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

/// The action button's callback is a plain `fn()` with no user data, and
/// `init_comm` may only be called once, so the `Comm` obtained in `sample_main`
/// is parked here for the callback to borrow.
static mut COMM_REF: Option<&'static mut Comm<DEFAULT_BUF_SIZE>> = None;

/// Points at the home screen builder owned by `sample_main`, so the action
/// callback can draw it again.
///
/// Store only a reference handle in `.bss` and avoid raw-pointer dereference
/// in the callback.
static mut HOME_REF: Option<&'static mut NbglHomeAndSettings> = None;

/// Runs when the home screen's action button is touched.
fn on_action() {
    let comm = unsafe {
        #[allow(static_mut_refs)]
        COMM_REF.as_mut().unwrap()
    };
    NbglStatus::new()
        .text("Action button clicked")
        .show(comm, true);

    // `show` returns once the status page times out after 3s. Nothing is drawn
    // at that point, so put the home screen back.
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(home) = HOME_REF.as_mut() {
            home.show_and_return();
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

    let settings_strings = [["Switch title", "Switch subtitle"]];
    let mut settings: settings::Settings = Default::default();

    // Display the home screen.
    // `info_list` takes an arbitrary number of (type, content) pairs; the
    // shorter `.infos(app_name, version, author)` covers the usual
    // Version/Developer pair.
    let action = HomeAction::new("Start action", on_action).icon(&FERRIS);

    let mut home = NbglHomeAndSettings::new()
        .glyph(&FERRIS)
        .settings(settings.get_mut(), &settings_strings)
        .app_name("Example App")
        .info_list(&[
            ("Version", env!("CARGO_PKG_VERSION")),
            ("Developer", env!("CARGO_PKG_AUTHORS")),
            ("Copyright", "(c) 2026 Ledger"),
        ])
        .action(&action);

    // Both statics must be populated before the home screen goes up, since the
    // action button can be touched as soon as it does. `home` outlives them:
    // `sample_main` never returns.
    let comm = unsafe {
        COMM_REF = Some(comm);
        HOME_REF = &mut home as *mut NbglHomeAndSettings;
        #[allow(static_mut_refs)]
        COMM_REF.as_mut().unwrap()
    };

    home.show_and_return();

    loop {
        let _ins = comm.next_command();
    }
}
