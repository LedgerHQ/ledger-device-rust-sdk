use crate::io::{ApduHeader, Comm, Event, Reply};
use crate::nvm::*;
use const_zero::const_zero;
use core::cell::RefCell;
use core::ffi::{c_char, CStr};
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

#[no_mangle]
pub static mut G_ux_params: bolos_ux_params_t = unsafe { const_zero!(bolos_ux_params_t) };

static mut COMM_REF: Option<&mut Comm> = None;
const SETTINGS_SIZE: usize = 10;
static mut NVM_REF: Option<&mut AtomicStorage<[u8; SETTINGS_SIZE]>> = None;
static mut SWITCH_ARRAY: [nbgl_contentSwitch_t; SETTINGS_SIZE] =
    [unsafe { const_zero!(nbgl_contentSwitch_t) }; SETTINGS_SIZE];

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub struct NbglGlyph<'a> {
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub is_file: bool,
    pub bitmap: &'a [u8],
}

impl<'a> NbglGlyph<'a> {
    pub const fn new(
        bitmap: &'a [u8],
        width: u16,
        height: u16,
        bpp: u8,
        is_file: bool,
    ) -> NbglGlyph<'a> {
        NbglGlyph {
            width,
            height,
            bpp,
            is_file,
            bitmap,
        }
    }
    pub const fn from_include(packed: (&'a [u8], u16, u16, u8, bool)) -> NbglGlyph<'a> {
        NbglGlyph {
            width: packed.1,
            height: packed.2,
            bpp: packed.3,
            is_file: packed.4,
            bitmap: packed.0,
        }
    }
}

impl<'a> Into<nbgl_icon_details_t> for &NbglGlyph<'a> {
    fn into(self) -> nbgl_icon_details_t {
        let bpp = match self.bpp {
            1 => NBGL_BPP_1,
            2 => NBGL_BPP_2,
            4 => NBGL_BPP_4,
            _ => panic!("Invalid bpp"),
        };
        nbgl_icon_details_t {
            width: self.width,
            height: self.height,
            bpp,
            isFile: self.is_file,
            bitmap: self.bitmap.as_ptr() as *const u8,
        }
    }
}

/// Initialize the global COMM_REF variable with the provided Comm instance.
/// This function should be called from the main function of the application.
/// The COMM_REF variable is used by the NBGL API to detect touch events and
/// APDU reception.
pub fn init_comm(comm: &mut Comm) {
    unsafe {
        COMM_REF = Some(transmute(comm));
    }
}

/// IO function used in the synchronous NBGL C library to process
/// events (touch, buttons, etc.) or detect if an APDU was received.
/// It returns true if an APDU was received, false otherwise.
#[no_mangle]
pub extern "C" fn io_recv_and_process_event() -> bool {
    unsafe {
        if let Some(comm) = COMM_REF.as_mut() {
            let apdu_received = comm.next_event_ahead::<ApduHeader>();
            if apdu_received {
                return true;
            }
        }
    }
    false
}

/// Helper struct that converts strings to null-terminated c strings.
/// It uses an internal buffer to store the strings, with a maximum size of SIZE.
struct CStringHelper<const SIZE: usize = 64> {
    /// Internal buffer where strings are allocated.
    /// Stored in a [RefCell] because we want [CStringHelper::to_cstring] to be non-mutable.
    pub buffer: RefCell<[u8; SIZE]>,
    /// Index of the next string in the internal buffer.
    /// Stored in a [RefCell] because we want [CStringHelper::to_cstring] to be non-mutable.
    next: RefCell<usize>,
}

impl<const SIZE: usize> CStringHelper<SIZE> {
    pub fn new() -> Self {
        Self {
            buffer: RefCell::new([0u8; SIZE]),
            next: RefCell::new(0),
        }
    }

    pub fn flush(&self) {
        *self.next.borrow_mut() = 0; // Reset the next index to 0
    }

    pub fn to_cstring<'a>(&'a self, s: &str) -> Result<&'a CStr, ()> {
        let size = s.len();
        let mut buffer = self.buffer.borrow_mut();
        let next: usize = *self.next.borrow();
        // Verify there is enough space in the internal buffer.
        // +1 for the null byte
        if size + next + 1 > buffer.len() {
            // Not enough space remaining in the internal buffer.
            return Err(());
        }
        // Verify that the input string does not have null bytes already.
        if s.bytes().find(|c| *c == 0).is_some() {
            return Err(());
        }

        // Copy the input string to the internal buffer, and add null byte.
        buffer[next..next + size].copy_from_slice(s.as_bytes());
        buffer[next + size] = 0;
        let start = next;
        *self.next.borrow_mut() += size + 1;

        let buffer = self.buffer.as_ptr();
        let slice = unsafe { &(*buffer)[start..start + size + 1] };
        let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(slice) };
        Ok(cstr)
    }
}

/// Callback triggered by the NBGL API when a setting switch is toggled.
unsafe fn settings_callback(token: ::core::ffi::c_int, _index: u8, _page: ::core::ffi::c_int) {
    let idx = token - FIRST_USER_TOKEN as i32;
    if idx < 0 || idx >= SETTINGS_SIZE as i32 {
        panic!("Invalid token.");
    }

    if let Some(data) = NVM_REF.as_mut() {
        let setting_idx: usize = idx as usize;
        let mut switch_values: [u8; SETTINGS_SIZE] = data.get_ref().clone();
        switch_values[setting_idx] = !switch_values[setting_idx];
        data.update(&switch_values);
        SWITCH_ARRAY[setting_idx].initState = switch_values[setting_idx] as nbgl_state_t;
    }
}

/// Informations fields name to display in the dedicated
/// page of the home screen.
const INFO_FIELDS: [*const c_char; 2] = [
    "Version\0".as_ptr() as *const c_char,
    "Developer\0".as_ptr() as *const c_char,
];

/// A wrapper around the synchronous NBGL ux_sync_homeAndSettings C API binding.
/// Used to display the home screen of the application, with an optional glyph,
/// information fields, and settings switches.  
pub struct NbglHomeAndSettings<'a> {
    app_name: *const c_char,
    icon: *const nbgl_icon_details_t,
    glyph: Option<&'a NbglGlyph<'a>>,
    info_contents: [*const c_char; 2],
    info_list: nbgl_contentInfoList_t,
    settings_contents: nbgl_content_t,
    nb_settings: u8,
    generic_contents: nbgl_genericContents_t,
    c_string_helper: CStringHelper<128>,
}

impl<'a> NbglHomeAndSettings<'a> {
    pub fn new() -> NbglHomeAndSettings<'a> {
        NbglHomeAndSettings {
            app_name: "Rust App\0".as_ptr() as *const c_char,
            icon: core::ptr::null(),
            glyph: None,
            info_contents: [
                "0.0.0\0".as_ptr() as *const c_char,
                "Ledger\0".as_ptr() as *const c_char,
            ],
            info_list: nbgl_contentInfoList_t {
                infoTypes: core::ptr::null(),
                infoContents: core::ptr::null(),
                nbInfos: 0,
            },
            c_string_helper: CStringHelper::<128>::new(),
            settings_contents: nbgl_content_t::default(),
            generic_contents: nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: core::ptr::null(),
                },
                nbContents: 0,
            },
            nb_settings: 0,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglHomeAndSettings<'a> {
        NbglHomeAndSettings {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn infos(self, app_name: &str, version: &str, author: &str) -> NbglHomeAndSettings<'a> {
        NbglHomeAndSettings {
            app_name: self.c_string_helper.to_cstring(app_name).unwrap().as_ptr() as *const c_char,
            info_contents: [
                self.c_string_helper.to_cstring(version).unwrap().as_ptr() as *const c_char,
                self.c_string_helper.to_cstring(author).unwrap().as_ptr() as *const c_char,
            ],
            ..self
        }
    }

    pub fn settings(
        self,
        nvm_data: &'a mut AtomicStorage<[u8; SETTINGS_SIZE]>,
        settings_strings: &[[&str; 2]],
    ) -> NbglHomeAndSettings<'a> {
        unsafe {
            NVM_REF = Some(transmute(nvm_data));
        }

        if settings_strings.len() > SETTINGS_SIZE {
            panic!("Too many settings.");
        }

        unsafe {
            for (i, setting) in settings_strings.iter().enumerate() {
                SWITCH_ARRAY[i].text = self
                    .c_string_helper
                    .to_cstring(setting[0])
                    .unwrap()
                    .as_ptr() as *const c_char;
                SWITCH_ARRAY[i].subText = self
                    .c_string_helper
                    .to_cstring(setting[1])
                    .unwrap()
                    .as_ptr() as *const c_char;
                SWITCH_ARRAY[i].initState = NVM_REF.as_mut().unwrap().get_ref()[i] as nbgl_state_t;
                SWITCH_ARRAY[i].token = (FIRST_USER_TOKEN + i as u32) as u8;
                SWITCH_ARRAY[i].tuneId = TuneIndex::TapCasual as u8;
            }
        }

        NbglHomeAndSettings {
            nb_settings: settings_strings.len() as u8,
            ..self
        }
    }

    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        unsafe {
            self.info_list.infoTypes = INFO_FIELDS.as_ptr() as *const *const c_char;
            self.info_list.infoContents = self.info_contents.as_ptr() as *const *const c_char;
            self.info_list.nbInfos = 2;

            if NVM_REF.is_some() {
                self.settings_contents = nbgl_content_t {
                    content: nbgl_content_u {
                        switchesList: nbgl_pageSwitchesList_s {
                            switches: &SWITCH_ARRAY as *const nbgl_contentSwitch_t,
                            nbSwitches: self.nb_settings,
                        },
                    },
                    contentActionCallback: transmute(
                        (|token, index, page| settings_callback(token, index, page))
                            as fn(::core::ffi::c_int, u8, ::core::ffi::c_int),
                    ),
                    type_: SWITCHES_LIST,
                };

                self.generic_contents = nbgl_genericContents_t {
                    callbackCallNeeded: false,
                    __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                        contentsList: &self.settings_contents as *const nbgl_content_t,
                    },
                    nbContents: 1,
                };
            }

            loop {
                if self.glyph.is_some() {
                    self.icon = &self.glyph.unwrap().into() as *const nbgl_icon_details_t;
                }
                match ledger_secure_sdk_sys::ux_sync_homeAndSettings(
                    self.app_name,
                    self.icon as *const nbgl_icon_details_t,
                    core::ptr::null(),
                    INIT_HOME_PAGE as u8,
                    &self.generic_contents as *const nbgl_genericContents_t,
                    &self.info_list as *const nbgl_contentInfoList_t,
                    core::ptr::null(),
                ) {
                    ledger_secure_sdk_sys::UX_SYNC_RET_APDU_RECEIVED => {
                        if let Some(comm) = COMM_REF.as_mut() {
                            if let Some(value) = comm.check_event() {
                                return value;
                            }
                        }
                    }
                    _ => {
                        panic!("Unexpected return value from ux_sync_homeAndSettings");
                    }
                }
            }
        }
    }
}

/// A wrapper around the synchronous NBGL ux_sync_review C API binding.
/// Used to display transaction review screens.
/// The maximum number of fields that can be displayed can be overriden by the
/// MAX_FIELD_NUMBER const parameter.
/// The maximum size of the internal buffer used to convert C strings can be overriden by the
/// STRING_BUFFER_SIZE const parameter.
pub struct NbglReview<
    'a,
    const MAX_FIELD_NUMBER: usize = 32,
    const STRING_BUFFER_SIZE: usize = 1024,
> {
    title: &'a str,
    subtitle: &'a str,
    finish_title: &'a str,
    glyph: Option<&'a NbglGlyph<'a>>,
    icon: *const nbgl_icon_details_t,
    tag_value_array: [nbgl_layoutTagValue_t; MAX_FIELD_NUMBER],
    c_string_helper: CStringHelper<STRING_BUFFER_SIZE>,
}

impl<'a, const MAX_FIELD_NUMBER: usize, const STRING_BUFFER_SIZE: usize>
    NbglReview<'a, MAX_FIELD_NUMBER, STRING_BUFFER_SIZE>
{
    pub fn new() -> NbglReview<'a, MAX_FIELD_NUMBER, STRING_BUFFER_SIZE> {
        NbglReview {
            title: "Please review\ntransaction",
            subtitle: "To send CRAB",
            finish_title: "Sign transaction",
            glyph: None,
            icon: core::ptr::null(),
            tag_value_array: [nbgl_layoutTagValue_t::default(); MAX_FIELD_NUMBER],
            c_string_helper: CStringHelper::<STRING_BUFFER_SIZE>::new(),
        }
    }

    pub fn titles(
        self,
        title: &'a str,
        subtitle: &'a str,
        finish_title: &'a str,
    ) -> NbglReview<'a, MAX_FIELD_NUMBER, STRING_BUFFER_SIZE> {
        NbglReview {
            title,
            subtitle,
            finish_title,
            ..self
        }
    }

    pub fn glyph(
        self,
        glyph: &'a NbglGlyph,
    ) -> NbglReview<'a, MAX_FIELD_NUMBER, STRING_BUFFER_SIZE> {
        NbglReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(&mut self, fields: &[Field]) -> bool {
        unsafe {
            // Check if there are too many fields (more than MAX_FIELD_NUMBER).
            if fields.len() > self.tag_value_array.len() {
                panic!("Too many fields for this review instance.");
            }

            // Flush the internal buffer of the CStringHelper.
            self.c_string_helper.flush();

            // Fill the tag_value_array with the fields converted to nbgl_layoutTagValue_t
            // with proper c strings (ending with \0).
            for (i, field) in fields.iter().enumerate() {
                let name = self.c_string_helper.to_cstring(field.name).unwrap();
                let value = self.c_string_helper.to_cstring(field.value).unwrap();
                self.tag_value_array[i] = nbgl_layoutTagValue_t {
                    item: name.as_ptr() as *const i8,
                    value: value.as_ptr() as *const i8,
                    valueIcon: core::ptr::null() as *const nbgl_icon_details_t,
                    _bitfield_align_1: [0; 0],
                    _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
                    __bindgen_padding_0: [0; 3usize],
                }
            }

            // Create the tag_value_list with the tag_value_array.
            let tag_value_list: nbgl_layoutTagValueList_t = nbgl_layoutTagValueList_t {
                pairs: self.tag_value_array.as_ptr() as *const nbgl_layoutTagValue_t,
                callback: None,
                nbPairs: fields.len() as u8,
                startIndex: 0,
                nbMaxLinesForValue: 0,
                token: 0,
                smallCaseForValue: false,
                wrapping: false,
            };

            if self.glyph.is_some() {
                self.icon = &self.glyph.unwrap().into() as *const nbgl_icon_details_t;
            }

            // Show the review on the device.
            let sync_ret = ledger_secure_sdk_sys::ux_sync_review(
                TYPE_TRANSACTION,
                &tag_value_list as *const nbgl_layoutTagValueList_t,
                self.icon,
                self.c_string_helper
                    .to_cstring(self.title)
                    .unwrap()
                    .as_ptr() as *const c_char,
                self.c_string_helper
                    .to_cstring(self.subtitle)
                    .unwrap()
                    .as_ptr() as *const c_char,
                self.c_string_helper
                    .to_cstring(self.finish_title)
                    .unwrap()
                    .as_ptr() as *const c_char,
            );

            // Return true if the user approved the transaction, false otherwise.
            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_TRANSACTION_SIGNED);
                    return true;
                }
                _ => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_TRANSACTION_REJECTED);
                    return false;
                }
            }
        }
    }
}

/// A wrapper around the synchronous NBGL ux_sync_addressReview C API binding.
/// Used to display address confirmation screens.
pub struct NbglAddressReview<'a> {
    icon: *const nbgl_icon_details_t,
    glyph: Option<&'a NbglGlyph<'a>>,
    verify_str: &'a str,
}

impl<'a> NbglAddressReview<'a> {
    pub fn new() -> NbglAddressReview<'a> {
        NbglAddressReview {
            verify_str: "Verify address",
            icon: core::ptr::null(),
            glyph: None,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressReview<'a> {
        NbglAddressReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn verify_str(self, verify_str: &'a str) -> NbglAddressReview<'a> {
        NbglAddressReview { verify_str, ..self }
    }

    pub fn show(&mut self, address: &str) -> bool {
        unsafe {
            // Create CStringHelper instance and convert the address and verify_str into c strings.
            let c_string_helper = CStringHelper::<128>::new();
            let c_addr_str = c_string_helper.to_cstring(address).unwrap();
            let c_verif_str = c_string_helper.to_cstring(self.verify_str).unwrap();

            if self.glyph.is_some() {
                self.icon = &self.glyph.unwrap().into() as *const nbgl_icon_details_t;
            }

            // Show the address confirmation on the device.
            let sync_ret = ux_sync_addressReview(
                c_addr_str.as_ptr() as *const c_char,
                core::ptr::null(),
                self.icon,
                c_verif_str.as_ptr() as *const c_char,
                core::ptr::null(),
            );

            // Return true if the user approved the address, false otherwise.
            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_ADDRESS_VERIFIED);
                    return true;
                }
                ledger_secure_sdk_sys::UX_SYNC_RET_REJECTED => {
                    ledger_secure_sdk_sys::ux_sync_reviewStatus(STATUS_TYPE_ADDRESS_REJECTED);
                    return false;
                }
                _ => {
                    panic!("Unexpected return value from ux_sync_addressReview");
                }
            }
        }
    }
}

enum TuneIndex {
    Reserved,
    Boot,
    Charging,
    LedgerMoment,
    Error,
    Neutral,
    Lock,
    Success,
    LookAtMe,
    TapCasual,
    TapNext,
}

impl TryFrom<u8> for TuneIndex {
    type Error = ();
    fn try_from(index: u8) -> Result<TuneIndex, ()> {
        Ok(match index {
            TUNE_RESERVED => TuneIndex::Reserved,
            TUNE_BOOT => TuneIndex::Boot,
            TUNE_CHARGING => TuneIndex::Charging,
            TUNE_LEDGER_MOMENT => TuneIndex::LedgerMoment,
            TUNE_ERROR => TuneIndex::Error,
            TUNE_NEUTRAL => TuneIndex::Neutral,
            TUNE_LOCK => TuneIndex::Lock,
            TUNE_SUCCESS => TuneIndex::Success,
            TUNE_LOOK_AT_ME => TuneIndex::LookAtMe,
            TUNE_TAP_CASUAL => TuneIndex::TapCasual,
            TUNE_TAP_NEXT => TuneIndex::TapNext,
            _ => return Err(()),
        })
    }
}

// this is a mock that does nothing yet, but should become a direct translation
// of the C original. This was done to avoid compiling `os_io_seproxyhal.c` which
// includes many other things
#[no_mangle]
extern "C" fn io_seproxyhal_play_tune(tune_index: u8) {
    let index = TuneIndex::try_from(tune_index);
    if index.is_err() {
        return;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    exit_app(1);
}
