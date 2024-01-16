use crate::io::{ApduHeader, Comm, Event, Reply, StatusWords};
use const_zero::const_zero;
use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;
use ledger_secure_sdk_sys::*;
use crate::testing::debug_print;


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

pub struct NbglUI<'a> {
    comm: Option<&'a mut Comm>,
}

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
    review_pairs: [ledger_secure_sdk_sys::nbgl_layoutTagValue_t; 10],
    nb_pairs: u8,
    review_status: ReviewStatus,
}

const INFOTYPES: [*const ::core::ffi::c_char; 2] = [
    "Version\0".as_ptr() as *const ::core::ffi::c_char,
    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
];

static mut CTX: NbglContext = unsafe { const_zero!(NbglContext) };

impl<'a> NbglUI<'a> {
    pub fn new(comm: Option<&'a mut Comm>) -> NbglUI<'a> {
        NbglUI { comm }
    }

    pub fn app_name(self, app_name: &'static str) -> NbglUI<'a> {
        unsafe {
            CTX.name[..app_name.len()].copy_from_slice(app_name.as_bytes());
        }
        self
    }

    pub fn icon(self, icon: &'static [u8]) -> NbglUI<'a> {
        unsafe {
            CTX.icon = Some(icon);
        }
        self
    }

    pub fn info_contents(self, version: &str, author: &str) -> NbglUI<'a> {
        unsafe {
            CTX.info_contents[0][..version.len()].copy_from_slice(version.as_bytes());
            CTX.info_contents[1][..author.len()].copy_from_slice(author.as_bytes());
        }
        self
    }

    fn settings() {
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
                    .as_ptr() as *const *const ::core::ffi::c_char;
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
                transmute((|| Self::home()) as fn()),
                transmute(nav as fn(u8, *mut nbgl_pageContent_s) -> bool),
                transmute((|| exit_app(12)) as fn()),
            );
        }
    }

    fn home() {
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
                transmute((|| Self::settings()) as fn()),
                transmute((|| exit_app(12)) as fn()),
            );
        }
    }

    pub fn show_home(&mut self) {
        Self::home();
    }

    fn choice(confirm: bool) {
        let show_home = || {
            Self::home();
        };
        
        let result_string: *const ::core::ffi::c_char;
        unsafe {
            if confirm {
                CTX.review_status = ReviewStatus::Validate;
                result_string = "TRANSACTION\nSIGNED\0".as_ptr() as *const ::core::ffi::c_char;
            } else {
                CTX.review_status = ReviewStatus::Reject;
                result_string = "Transaction\nRejected\0".as_ptr() as *const ::core::ffi::c_char;
            }
        }

        unsafe {
            ledger_secure_sdk_sys::nbgl_useCaseStatus(
                result_string,
                confirm,
                transmute(show_home as fn()));
        }
    }
    
    fn static_review() {
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
                longPressText: "Hold to validate\0".as_ptr() as *const ::core::ffi::c_char,
                longPressToken: 0,
                tuneId: 0,
            };

            ledger_secure_sdk_sys::nbgl_useCaseStaticReview(
                &tag_value_list as *const nbgl_layoutTagValueList_t,
                &info_long_press as *const nbgl_pageInfoLongPress_t,
                "Reject tx\0".as_ptr() as *const ::core::ffi::c_char,
                // None,
                transmute((|confirm| Self::choice(confirm)) as fn(confirm: bool)),
            );
        }
    }
    
    pub fn show_review_and_wait_validation(&mut self, fields: &[Field]) -> bool {
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

            ledger_secure_sdk_sys::nbgl_useCaseReviewStart(
                &icon as *const nbgl_icon_details_t,
                "Review tx\0".as_ptr() as *const ::core::ffi::c_char,
                "Subtitle\0".as_ptr() as *const ::core::ffi::c_char,
                "Reject transaction\0".as_ptr() as *const ::core::ffi::c_char,
                transmute((|| Self::static_review()) as fn()),
                None,
            );

            loop {
                self.get_event::<DummyEvent>();

                if CTX.review_status == ReviewStatus::Validate {
                    debug_print("Validate\n");
                    return true;
                }
                else if CTX.review_status == ReviewStatus::Reject {
                    debug_print("Reject\n");
                    return false;
                }
            }
        }
    }

    fn get_event<T: TryFrom<ApduHeader>>(&mut self) -> Option<Event<T>>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        match &mut self.comm {
            None => None,
            Some(comm) => {
                if let Event::Command(ins) = comm.next_event() {
                    return Some(Event::Command(ins));
                }
                else
                {
                    return None;
                }
            }
        }
    }

    pub fn get_events<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        loop {
            if let Some(event) = self.get_event::<T>() {
                return event;
            }
        }
    }
}
