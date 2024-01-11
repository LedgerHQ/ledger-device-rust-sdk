use crate::io::{self, ApduHeader, Comm, Event, Reply};
use const_zero::const_zero;
use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;
use ledger_secure_sdk_sys::seph;
use ledger_secure_sdk_sys::*;

pub struct Home<'a> {
    comm: Option<&'a mut Comm>,
}

struct info_struct {
    icon: Option<&'static [u8]>,
    name: [u8; 100],
    infocontents: [[u8; 20]; 2],
}

const infoTypes: [*const ::core::ffi::c_char; 2] = [
    "Version\0".as_ptr() as *const ::core::ffi::c_char,
    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
];

static mut infos: info_struct = unsafe { const_zero!(info_struct) };

impl<'a> Home<'a> {
    pub fn new(comm: Option<&'a mut Comm>) -> Home<'a> {
        Home { comm }
    }

    pub fn app_name(self, app_name: &'static str) -> Home<'a> {
        unsafe {
            infos.name[..app_name.len()].copy_from_slice(app_name.as_bytes());
        }
        self
    }

    pub fn icon(self, icon: &'static [u8]) -> Home<'a> {
        unsafe {
            infos.icon = Some(icon);
        }
        self
    }

    pub fn info_contents(self, version: &str, author: &str) -> Home<'a> {
        unsafe {
            infos.infocontents[0][..version.len()].copy_from_slice(version.as_bytes());
            infos.infocontents[1][..author.len()].copy_from_slice(author.as_bytes());
        }
        self
    }

    fn settings() {
        unsafe {
            let nav = |page: u8, content: *mut nbgl_pageContent_s| {
                if page == 0 {
                    (*content).type_ = ledger_secure_sdk_sys::INFOS_LIST;
                    (*content).__bindgen_anon_1.infosList.nbInfos = 2;
                    (*content).__bindgen_anon_1.infosList.infoTypes = infoTypes.as_ptr();
                    (*content).__bindgen_anon_1.infosList.infoContents = [
                        infos.infocontents[0].as_ptr() as *const ::core::ffi::c_char,
                        infos.infocontents[1].as_ptr() as *const ::core::ffi::c_char,
                    ]
                    .as_ptr();
                } else {
                    return false;
                }
                true
            };

            ledger_secure_sdk_sys::nbgl_useCaseSettings(
                infos.name.as_ptr() as *const core::ffi::c_char,
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
                bitmap: infos.icon.unwrap().as_ptr(),
            };

            ledger_secure_sdk_sys::nbgl_useCaseHome(
                infos.name.as_ptr() as *const core::ffi::c_char,
                &icon as *const nbgl_icon_details_t,
                core::ptr::null(),
                true as bool,
                transmute((|| Self::settings()) as fn()),
                transmute((|| exit_app(12)) as fn()),
            );
        }
    }

    pub fn show(&mut self) {
        Self::home();
    }

    pub fn get_events<T: TryFrom<ApduHeader>>(&mut self) -> Event<T>
    where
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        loop {
            match &mut self.comm {
                None => (),
                Some(comm) => {
                    if let Event::Command(ins) = comm.next_event() {
                        return Event::Command(ins);
                    }
                }
            }
        }
    }
}
