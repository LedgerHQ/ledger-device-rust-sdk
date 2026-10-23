#![no_std]
#![no_main]

use include_gif::include_gif;
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use ledger_device_sdk::nbgl::NbglChoice;
#[cfg(not(any(target_os = "nanosplus", target_os = "nanox")))]
use ledger_device_sdk::nbgl::NbglConfirm;
use ledger_device_sdk::nbgl::{HomeAction, NbglGlyph, NbglHomeAndSettings, NbglStatus, init_comm};
// use ledger_device_sdk::nvm::*;
// use ledger_device_sdk::NVMData;
use core::sync::atomic::{AtomicBool, Ordering};
use ledger_device_sdk::io::{ApduHeader, Comm, DEFAULT_BUF_SIZE, Event, StatusWords};

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

/// Set by the action's callbacks, cleared by the main loop once it has run
/// the follow-up.
///
/// NBGL calls them while `Comm` is decoding a button or touch event, so they
/// must not use `Comm` themselves, nor block. They only record what is due,
/// and the main loop acts on it once that event has been handled.
static PENDING: AtomicBool = AtomicBool::new(false);

/// Runs when the home screen's action button is touched.
///
/// Draws a confirmation modal over the home screen. Being a modal it needs
/// something underneath, which is why it is raised from here rather than from
/// the top of `sample_main`: dismissing it through the footer simply reveals the
/// home screen again, and reports nothing back.
#[cfg(not(any(target_os = "nanosplus", target_os = "nanox")))]
fn on_action() {
    NbglConfirm::new()
        .message("Delete this account?")
        .sub_message("This cannot be undone.")
        .texts("Delete", "Cancel")
        .show_and_return(on_confirm);
}

/// Runs when the confirmation modal's button is touched.
///
/// There is no counterpart for the footer: `nbgl_useCaseConfirm` reports only
/// confirmation, and dismissal needs no handling here because the home screen is
/// already drawn behind the modal.
#[cfg(not(any(target_os = "nanosplus", target_os = "nanox")))]
fn on_confirm() {
    PENDING.store(true, Ordering::Relaxed);
}

/// Shows that the account was deleted, then the home screen again.
#[cfg(not(any(target_os = "nanosplus", target_os = "nanox")))]
fn run_pending(comm: &mut Comm<DEFAULT_BUF_SIZE>, home: &mut NbglHomeAndSettings) {
    NbglStatus::new().text("Account deleted").show(comm, true);

    // `show` returns once the status page times out after 3s. Nothing is drawn
    // at that point, so put the home screen back.
    home.show_and_return();
}

/// Runs when the home screen's action is selected.
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
fn on_action() {
    PENDING.store(true, Ordering::Relaxed);
}

/// Asks whether to delete the account, then shows the home screen again.
///
/// Nano has no modal: a confirmation replaces the home screen and its state,
/// and reports nothing when dismissed. A choice is used instead, which reports
/// both outcomes, and the home screen is drawn again whatever the answer.
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
fn run_pending(comm: &mut Comm<DEFAULT_BUF_SIZE>, home: &mut NbglHomeAndSettings) {
    if NbglChoice::new().show(
        comm,
        "Delete this account?",
        "This cannot be undone.",
        "Delete",
        "Cancel",
    ) {
        NbglStatus::new().text("Account deleted").show(comm, true);
    }
    home.show_and_return();
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
    let action = HomeAction::new("Delete account", on_action).icon(&FERRIS);

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

    home.show_and_return();

    loop {
        // Commands must be answered: this example does not implement any.
        if let Event::Command(cmd) = comm.next_event() {
            let _ = cmd.reply(&[], StatusWords::Unknown);
        }

        // The event that set it has been handled, so `Comm` is free again.
        if PENDING.load(Ordering::Relaxed) {
            PENDING.store(false, Ordering::Relaxed);
            run_pending(comm, &mut home);
        }
    }
}
