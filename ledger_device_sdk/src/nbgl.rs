use crate::io::{ApduHeader, Comm, Event, Reply};
use const_zero::const_zero;
use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;
use ledger_secure_sdk_sys::*;

pub struct NbglHome;
pub struct NbglReview;
pub struct NbglAddressConfirm;

#[derive(PartialEq)]
pub enum ReviewStatus {
    Pending,
    Validate,
    Reject,
}

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

struct NbglContext {
    icon: Option<&'static [u8]>,
    name: [u8; 100],
    info_contents: [[u8; 20]; 2],
    review_confirm_string: [u8; 40],
    review_reject_string: [u8; 40],
    review_pairs: [ledger_secure_sdk_sys::nbgl_layoutTagValue_t; 10],
    nb_pairs: u8,
}

const INFOTYPES: [*const ::core::ffi::c_char; 2] = [
    "Version\0".as_ptr() as *const ::core::ffi::c_char,
    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
];

#[no_mangle]
pub extern "C" fn recv_and_process_event(return_on_apdu: bool) -> bool {
    unsafe {
        if let Some(comm) = COMM_REF.as_mut() {
            let apdu_received = comm.next_event_ahead::<ApduHeader>();
            if apdu_received && return_on_apdu {
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub static mut G_ux_params: bolos_ux_params_t = unsafe { const_zero!(bolos_ux_params_t) };

static mut CTX: NbglContext = unsafe { const_zero!(NbglContext) };
static mut COMM_REF: Option<&mut Comm> = None;

impl NbglHome {
    pub fn new(comm: &mut Comm) -> NbglHome {
        unsafe {
            COMM_REF = Some(transmute(comm));
        }
        NbglHome {}
    }

    pub fn app_name(self, app_name: &'static str) -> NbglHome {
        unsafe {
            CTX.name[..app_name.len()].copy_from_slice(app_name.as_bytes());
        }
        self
    }

    pub fn icon(self, icon: &'static [u8]) -> NbglHome {
        unsafe {
            CTX.icon = Some(icon);
        }
        self
    }

    pub fn info_contents(self, version: &str, author: &str) -> NbglHome {
        unsafe {
            CTX.info_contents[0][..version.len()].copy_from_slice(version.as_bytes());
            CTX.info_contents[1][..author.len()].copy_from_slice(author.as_bytes());
        }
        self
    }

    pub fn show<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        unsafe {
            loop {
                match ledger_secure_sdk_sys::sync_nbgl_useCaseHomeAndSettings() {
                    ledger_secure_sdk_sys::NBGL_SYNC_RET_RX_APDU => {
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

impl NbglReview {
    pub fn review_transaction(&mut self, fields: &[Field]) -> bool {
        unsafe {
            let icon = nbgl_icon_details_t {
                width: 64,
                height: 64,
                bpp: 2,
                isFile: true,
                bitmap: CTX.icon.unwrap().as_ptr(),
            };

            for (i, field) in fields.iter().enumerate() {
                if i >= CTX.review_pairs.len() {
                    break;
                }
                CTX.review_pairs[i] = nbgl_layoutTagValue_t {
                    item: field.name.as_ptr() as *const ::core::ffi::c_char,
                    value: field.value.as_ptr() as *const ::core::ffi::c_char,
                    valueIcon: core::ptr::null(),
                };
            }

            CTX.nb_pairs = if fields.len() < CTX.review_pairs.len() {
                fields.len()
            } else {
                CTX.review_pairs.len()
            } as u8;

            let confirm_string = "TRANSACTION\nSIGNED\0";
            CTX.review_confirm_string[..confirm_string.len()]
                .copy_from_slice(confirm_string.as_bytes());
            let reject_string = "Transaction\nRejected\0";
            CTX.review_reject_string[..reject_string.len()]
                .copy_from_slice(reject_string.as_bytes());

            let tag_value_list: nbgl_layoutTagValueList_t = nbgl_layoutTagValueList_t {
                pairs: CTX.review_pairs.as_mut_ptr() as *mut nbgl_layoutTagValue_t,
                callback: None,
                nbPairs: CTX.nb_pairs,
                startIndex: 2,
                nbMaxLinesForValue: 0,
                token: 0,
                smallCaseForValue: false,
                wrapping: false,
            };

            let sync_ret = ledger_secure_sdk_sys::sync_nbgl_useCaseTransactionReview(
                &tag_value_list as *const nbgl_layoutTagValueList_t,
                &icon as *const nbgl_icon_details_t,
                "Review tx\0".as_ptr() as *const ::core::ffi::c_char,
                "Subtitle\0".as_ptr() as *const ::core::ffi::c_char,
                "Tx confirmed\0".as_ptr() as *const ::core::ffi::c_char,
            );

            match sync_ret {
                ledger_secure_sdk_sys::NBGL_SYNC_RET_SUCCESS => {
                    ledger_secure_sdk_sys::sync_nbgl_useCaseStatus(
                        CTX.review_confirm_string.as_ptr() as *const ::core::ffi::c_char,
                        true,
                    );
                    return true;
                }
                _ => {
                    ledger_secure_sdk_sys::sync_nbgl_useCaseStatus(
                        CTX.review_reject_string.as_ptr() as *const ::core::ffi::c_char,
                        false,
                    );
                    return false;
                }
            }
        }
    }
}

// =================================================================================================

impl NbglAddressConfirm {
    pub fn verify_address(&mut self, address: &str) -> bool {
        unsafe {
            let icon = nbgl_icon_details_t {
                width: 64,
                height: 64,
                bpp: 2,
                isFile: true,
                bitmap: CTX.icon.unwrap().as_ptr(),
            };

            let sync_ret = sync_nbgl_useCaseAddressReview(
                address.as_ptr() as *const ::core::ffi::c_char,
                &icon as *const nbgl_icon_details_t,
                "Verify address\0".as_ptr() as *const ::core::ffi::c_char,
                core::ptr::null(),
            );

            match sync_ret {
                ledger_secure_sdk_sys::NBGL_SYNC_RET_SUCCESS => {
                    return true;
                }
                ledger_secure_sdk_sys::NBGL_SYNC_RET_REJECTED => {
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
