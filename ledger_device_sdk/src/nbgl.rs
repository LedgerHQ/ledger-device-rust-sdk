use crate::io::{ApduHeader, Comm, Event, Reply};
use const_zero::const_zero;
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

#[derive(PartialEq)]
pub enum ReviewStatus {
    Pending,
    Validate,
    Reject,
}

pub struct NbglHome<'a> {
    app_name: &'a str,
    info_contents: [&'a str; 2],
    glyph: Option<&'a NbglGlyph<'a>>,
}

pub struct NbglReview<'a> {
    title: &'a str,
    subtitle: &'a str,
    finish_title: &'a str,
    glyph: Option<&'a NbglGlyph<'a>>,
}

pub struct NbglAddressConfirm<'a> {
    glyph: Option<&'a NbglGlyph<'a>>,
    verify_str: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Into<nbgl_layoutTagValue_t> for Field<'a> {
    fn into(self) -> nbgl_layoutTagValue_t {
        nbgl_layoutTagValue_t {
            item: self.name.as_ptr() as *const i8,
            value: self.value.as_ptr() as *const i8,
            valueIcon: core::ptr::null() as *const nbgl_icon_details_t,
            _bitfield_align_1: [0; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0; 1usize]),
            __bindgen_padding_0: [0; 3usize],
        }
    }
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

#[no_mangle]
pub static mut G_ux_params: bolos_ux_params_t = unsafe { const_zero!(bolos_ux_params_t) };

static mut COMM_REF: Option<&mut Comm> = None;

impl<'a> NbglHome<'a> {
    pub fn new(comm: &mut Comm) -> NbglHome {
        unsafe {
            COMM_REF = Some(transmute(comm));
        }
        NbglHome {
            app_name: "App\0",
            info_contents: ["0.0.0\0", "Ledger\0"],
            glyph: None,
        }
    }

    pub fn app_name(self, app_name: &'a str) -> NbglHome<'a> {
        NbglHome { app_name, ..self }
    }

    pub fn glyph(self, icon: &'a NbglGlyph) -> NbglHome<'a> {
        NbglHome {
            glyph: Some(icon),
            ..self
        }
    }

    pub fn info_contents(self, version: &'a str, author: &'a str) -> NbglHome<'a> {
        NbglHome {
            info_contents: [version, author],
            ..self
        }
    }

    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let icon = if let Some(glyph) = self.glyph {
            &glyph.into() as *const nbgl_icon_details_t
        } else {
            core::ptr::null() as *const nbgl_icon_details_t
        };
        unsafe {
            let info_list: nbgl_contentInfoList_t = nbgl_contentInfoList_t {
                infoTypes: [
                    "Version\0".as_ptr() as *const ::core::ffi::c_char,
                    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
                ]
                .as_ptr(),
                infoContents: [
                    self.info_contents[0].as_ptr() as *const ::core::ffi::c_char,
                    self.info_contents[1].as_ptr() as *const ::core::ffi::c_char,
                ]
                .as_ptr(),
                nbInfos: 2,
            };

            let setting_contents: nbgl_genericContents_t = nbgl_genericContents_t {
                callbackCallNeeded: false,
                __bindgen_anon_1: nbgl_genericContents_t__bindgen_ty_1 {
                    contentsList: core::ptr::null(),
                },
                nbContents: 0,
            };
            loop {
                match ledger_secure_sdk_sys::ux_sync_homeAndSettings(
                    self.app_name.as_ptr() as *const ::core::ffi::c_char,
                    icon,
                    core::ptr::null(),
                    INIT_HOME_PAGE as u8,
                    &setting_contents as *const nbgl_genericContents_t,
                    &info_list as *const nbgl_contentInfoList_t,
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
                        panic!("Unexpected return value from sync_nbgl_useCaseHome");
                    }
                }
            }
        }
    }
}

impl<'a> NbglReview<'a> {
    pub fn new() -> NbglReview<'a> {
        NbglReview {
            title: "Please review\ntransaction\0",
            subtitle: "\0",
            finish_title: "Sign transaction\0",
            glyph: None,
        }
    }

    pub fn titles(
        self,
        title: &'a str,
        subtitle: &'a str,
        finish_title: &'a str,
    ) -> NbglReview<'a> {
        NbglReview {
            title,
            subtitle,
            finish_title,
            ..self
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglReview<'a> {
        NbglReview {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn show(&self, fields: &[nbgl_layoutTagValue_t]) -> bool {
        unsafe {
            let tag_value_list: nbgl_layoutTagValueList_t = nbgl_layoutTagValueList_t {
                pairs: fields.as_ptr(),
                callback: None,
                nbPairs: fields.len() as u8,
                startIndex: 0,
                nbMaxLinesForValue: 0,
                token: 0,
                smallCaseForValue: false,
                wrapping: false,
            };

            let icon = if let Some(glyph) = self.glyph {
                &glyph.into() as *const nbgl_icon_details_t
            } else {
                core::ptr::null() as *const nbgl_icon_details_t
            };

            let sync_ret = ledger_secure_sdk_sys::ux_sync_review(
                TYPE_TRANSACTION,
                &tag_value_list as *const nbgl_layoutTagValueList_t,
                icon,
                self.title.as_ptr() as *const ::core::ffi::c_char,
                self.subtitle.as_ptr() as *const ::core::ffi::c_char,
                self.finish_title.as_ptr() as *const ::core::ffi::c_char,
            );

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

// =================================================================================================

impl<'a> NbglAddressConfirm<'a> {
    pub fn new() -> NbglAddressConfirm<'a> {
        NbglAddressConfirm {
            verify_str: "Verify address\0",
            glyph: None,
        }
    }

    pub fn glyph(self, glyph: &'a NbglGlyph) -> NbglAddressConfirm<'a> {
        NbglAddressConfirm {
            glyph: Some(glyph),
            ..self
        }
    }

    pub fn verify_str(self, verify_str: &'a str) -> NbglAddressConfirm<'a> {
        NbglAddressConfirm { verify_str, ..self }
    }

    pub fn show(&mut self, address: &str) -> bool {
        unsafe {
            let icon = if let Some(glyph) = self.glyph {
                &glyph.into() as *const nbgl_icon_details_t
            } else {
                core::ptr::null() as *const nbgl_icon_details_t
            };

            let sync_ret = ux_sync_addressReview(
                address.as_ptr() as *const ::core::ffi::c_char,
                core::ptr::null(),
                icon,
                self.verify_str.as_ptr() as *const ::core::ffi::c_char,
                core::ptr::null(),
            );

            match sync_ret {
                ledger_secure_sdk_sys::UX_SYNC_RET_APPROVED => {
                    return true;
                }
                ledger_secure_sdk_sys::UX_SYNC_RET_REJECTED => {
                    return false;
                }
                _ => {
                    panic!("Unexpected return value from sync_nbgl_useCaseTransactionReview");
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
