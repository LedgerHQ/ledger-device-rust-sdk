use crate::io::{ApduHeader, Comm, Event, Reply, StatusWords};
use const_zero::const_zero;
use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;
use ledger_secure_sdk_sys::*;

struct DummyEvent;

impl TryFrom<ApduHeader> for DummyEvent {
    type Error = StatusWords;

    fn try_from(value: ApduHeader) -> Result<Self, Self::Error> {
        match value.ins {
            1 => Ok(Self),
            _ => Err(StatusWords::NothingReceived),
        }
    }
}

pub struct NbglHome<'a> {
    comm: &'a mut Comm,
}

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
    review_status: ReviewStatus,
}

const INFOTYPES: [*const ::core::ffi::c_char; 2] = [
    "Version\0".as_ptr() as *const ::core::ffi::c_char,
    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
];

fn process_touch_events() {
    if !seph::is_status_sent() {
        seph::send_general_status();
    }

    let mut buf = [0u8; 8];
    while seph::is_status_sent() {
        seph::seph_recv(&mut buf, 0);

        match buf[0] as u32 {
            SEPROXYHAL_TAG_FINGER_EVENT => {
                unsafe {
                    ux_process_finger_event(buf.as_mut_ptr());
                }
            }
            SEPROXYHAL_TAG_TICKER_EVENT => unsafe {
                ux_process_ticker_event();
            }
            _ => (),
        }
    }
}

fn get_events<T: TryFrom<ApduHeader>>(comm: &mut Comm) -> Event<T>
where
    Reply: From<<T as TryFrom<ApduHeader>>::Error>,
{
    loop {
        if let Event::Command(ins) = comm.next_event() {
            return Event::Command(ins);
        }
    }
}

fn wait_review_result() -> bool {
    loop {
        process_touch_events();
        unsafe {
            if CTX.review_status == ReviewStatus::Validate {
                return true;
            } else if CTX.review_status == ReviewStatus::Reject {
                return false;
            }
        }
    }
}

fn display_review_result_cbk(confirm: bool) {
    let result_string: *const ::core::ffi::c_char;
    unsafe {
        if confirm {
            CTX.review_status = ReviewStatus::Validate;
            result_string = CTX.review_confirm_string.as_ptr() as *const ::core::ffi::c_char;
        } else {
            CTX.review_status = ReviewStatus::Reject;
            result_string = CTX.review_reject_string.as_ptr() as *const ::core::ffi::c_char;
        }

        // We don't need the callback here as we're going to display the home screen from the main app loop
        ledger_secure_sdk_sys::nbgl_useCaseStatus(result_string, confirm, None);
    }
}

#[no_mangle]
pub static mut G_ux_params: bolos_ux_params_t = unsafe { const_zero!(bolos_ux_params_t) };

static mut CTX: NbglContext = unsafe { const_zero!(NbglContext) };

impl<'a> NbglHome<'a> {
    pub fn new(comm: &'a mut Comm) -> NbglHome<'a> {
        NbglHome { comm }
    }

    pub fn app_name(self, app_name: &'static str) -> NbglHome<'a> {
        unsafe {
            CTX.name[..app_name.len()].copy_from_slice(app_name.as_bytes());
        }
        self
    }

    pub fn icon(self, icon: &'static [u8]) -> NbglHome<'a> {
        unsafe {
            CTX.icon = Some(icon);
        }
        self
    }

    pub fn info_contents(self, version: &str, author: &str) -> NbglHome<'a> {
        unsafe {
            CTX.info_contents[0][..version.len()].copy_from_slice(version.as_bytes());
            CTX.info_contents[1][..author.len()].copy_from_slice(author.as_bytes());
        }
        self
    }

    pub fn show_home<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        Self::display_home_cbk();
        return get_events(self.comm);
    }

    fn display_home_cbk() {
        unsafe {
            let icon = nbgl_icon_details_t {
                width: 64,
                height: 64,
                bpp: 2,
                isFile: true,
                bitmap: CTX.icon.unwrap().as_ptr(),
            };
    
            ledger_secure_sdk_sys::nbgl_useCaseHome(
                CTX.name.as_ptr() as *const core::ffi::c_char,
                &icon as *const nbgl_icon_details_t,
                core::ptr::null(),
                true as bool,
                transmute((|| Self::display_settings_cbk()) as fn()),
                transmute((|| exit_app(12)) as fn()),
            );
        }
    }
    
    fn display_settings_cbk() {
        unsafe {
            let nav = |page: u8, content: *mut nbgl_pageContent_s| {
                if page == 0 {
                    (*content).type_ = ledger_secure_sdk_sys::INFOS_LIST;
                    (*content).__bindgen_anon_1.infosList.nbInfos = 2;
                    (*content).__bindgen_anon_1.infosList.infoTypes = INFOTYPES.as_ptr();
                    (*content).__bindgen_anon_1.infosList.infoContents = [
                        CTX.info_contents[0].as_ptr() as *const ::core::ffi::c_char,
                        CTX.info_contents[1].as_ptr() as *const ::core::ffi::c_char,
                    ]
                    .as_ptr()
                        as *const *const ::core::ffi::c_char;
                } else {
                    return false;
                }
                true
            };
    
            ledger_secure_sdk_sys::nbgl_useCaseSettings(
                CTX.name.as_ptr() as *const core::ffi::c_char,
                0 as u8,
                1 as u8,
                false as bool,
                transmute((|| Self::display_home_cbk()) as fn()),
                transmute(nav as fn(u8, *mut nbgl_pageContent_s) -> bool),
                transmute((|| exit_app(12)) as fn()),
            );
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

            CTX.review_status = ReviewStatus::Pending;
            let confirm_string = "TRANSACTION\nSIGNED\0";
            CTX.review_confirm_string[..confirm_string.len()]
                .copy_from_slice(confirm_string.as_bytes());
            let reject_string = "Transaction\nRejected\0";
            CTX.review_reject_string[..reject_string.len()]
                .copy_from_slice(reject_string.as_bytes());

            ledger_secure_sdk_sys::nbgl_useCaseReviewStart(
                &icon as *const nbgl_icon_details_t,
                "Review tx\0".as_ptr() as *const ::core::ffi::c_char,
                "Subtitle\0".as_ptr() as *const ::core::ffi::c_char,
                "Reject transaction\0".as_ptr() as *const ::core::ffi::c_char,
                transmute((|| Self::display_static_review_cbk()) as fn()),
                transmute((|| display_review_result_cbk(false)) as fn()),
            );

            return wait_review_result();
        }
    }

    fn display_static_review_cbk() {
        unsafe {
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

            // Using this icon causes a crash
            // let icon = nbgl_icon_details_t {
            //     width: 64,
            //     height: 64,
            //     bpp: 2,
            //     isFile: true,
            //     bitmap: CTX.icon.unwrap().as_ptr(),
            // };

            let info_long_press: nbgl_pageInfoLongPress_t = nbgl_pageInfoLongPress_t {
                text: "Validate tx\0".as_ptr() as *const ::core::ffi::c_char,
                icon: core::ptr::null(),
                longPressText: "Hold to sign\0".as_ptr() as *const ::core::ffi::c_char,
                longPressToken: 0,
                tuneId: 0,
            };

            ledger_secure_sdk_sys::nbgl_useCaseStaticReview(
                &tag_value_list as *const nbgl_layoutTagValueList_t,
                &info_long_press as *const nbgl_pageInfoLongPress_t,
                "Reject tx\0".as_ptr() as *const ::core::ffi::c_char,
                transmute(
                    (|confirm| display_review_result_cbk(confirm)) as fn(confirm: bool),
                ),
            );
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

            CTX.nb_pairs = 1;
            CTX.review_pairs[0] = nbgl_layoutTagValue_t {
                item: "\0".as_ptr() as *const ::core::ffi::c_char, // Unused
                value: address.as_ptr() as *const ::core::ffi::c_char,
                valueIcon: core::ptr::null(),
            };
            CTX.review_status = ReviewStatus::Pending;

            ledger_secure_sdk_sys::nbgl_useCaseReviewStart(
                &icon as *const nbgl_icon_details_t,
                "Verify address\0".as_ptr() as *const ::core::ffi::c_char,
                core::ptr::null(),
                "Cancel\0".as_ptr() as *const ::core::ffi::c_char,
                transmute((|| Self::display_address_confirmation_cbk()) as fn()),
                None,
            );

            return wait_review_result();
        }
    }

    fn display_address_confirmation_cbk() {
        unsafe {
            let confirm_string = "ADDRESS\nVERIFIED\0";
            CTX.review_confirm_string[..confirm_string.len()]
                .copy_from_slice(confirm_string.as_bytes());
            let reject_string = "Address verification\ncancelled\0";
            CTX.review_reject_string[..reject_string.len()]
                .copy_from_slice(reject_string.as_bytes());

            ledger_secure_sdk_sys::nbgl_useCaseAddressConfirmation(
                CTX.review_pairs[0].value as *const ::core::ffi::c_char,
                transmute(
                    (|confirm| display_review_result_cbk(confirm)) as fn(confirm: bool),
                ),
            );
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