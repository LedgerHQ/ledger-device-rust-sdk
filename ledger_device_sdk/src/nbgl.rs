use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;
use crate::io::{self, ApduHeader, Comm, Event, Reply};
use ledger_secure_sdk_sys::seph;
use ledger_secure_sdk_sys::*;

pub struct Home<'a> {
    app_name: &'static str,
    icon: Option<&'a nbgl_icon_details_t>,
    tagline: Option<&'static str>,
    with_settings: bool,
    top_right_cb: Option<fn()>,
    quit_cb: Option<fn()>,
    comm: Option<&'a mut Comm>,
}

const infoTypes: [*const ::core::ffi::c_char; 2] = [
    "Version\0".as_ptr() as *const ::core::ffi::c_char,
    "Developer\0".as_ptr() as *const ::core::ffi::c_char,
];

const infoContents: [*const ::core::ffi::c_char; 2] = [
    env!("CARGO_PKG_VERSION").as_ptr() as *const ::core::ffi::c_char,
    "Ledger\0".as_ptr() as *const ::core::ffi::c_char,
];

fn settings_nav(page: u8, content: *mut nbgl_pageContent_s) -> bool
{
    if page == 0 {
        unsafe {
            (*content).type_ = ledger_secure_sdk_sys::INFOS_LIST;
            (*content).__bindgen_anon_1.infosList.nbInfos = 2;
            (*content).__bindgen_anon_1.infosList.infoTypes = infoTypes.as_ptr();
            (*content).__bindgen_anon_1.infosList.infoContents = infoContents.as_ptr();
        }
    } else {
        return false;
    }
    true
}

fn settings() {
    unsafe {
        ledger_secure_sdk_sys::nbgl_useCaseSettings(
            "My App\0".as_ptr() as *const core::ffi::c_char,
            0 as u8,
            1 as u8,
            false as bool,
            transmute((|| home_nav()) as fn()),
            transmute((|arg1,arg2| settings_nav(arg1,arg2)) as fn(u8, *mut nbgl_pageContent_s) -> bool),
            transmute((|| exit_app(12)) as fn()),
        );
    }
}

fn home_nav() {
    unsafe {
        ledger_secure_sdk_sys::nbgl_useCaseHome(
            "My App\0".as_ptr() as *const core::ffi::c_char,
            core::ptr::null(),
            core::ptr::null(),
            true as bool,
            transmute((|| settings()) as fn()),
            transmute((|| exit_app(12)) as fn()),
        );
    }
}


impl<'a> Home<'a> {
    pub fn new(comm: Option<&'a mut Comm>) -> Home<'a> {
    
        Home {
            app_name: "AppName\0",
            icon: None,
            tagline: None,
            with_settings: false,
            top_right_cb: None,
            quit_cb: None,
            comm,
        }
    }

    pub fn app_name(self, app_name: &'static str) -> Home<'a> {
        Home { app_name, ..self }
    }

    pub fn icon(self, icon: &'a nbgl_icon_details_t) -> Home<'a> {
        Home {
            icon: Some(icon),
            ..self
        }
    }

    pub fn top_right_cb(self, top_right_cb: fn()) -> Home<'a> {
        Home {
            top_right_cb: Some(top_right_cb),
            ..self
        }
    }

    pub fn quit_cb(self, quit_cb: fn()) -> Home<'a> {
        Home {
            quit_cb: Some(quit_cb),
            ..self
        }
    }

    pub fn show(&mut self) 
    {
        unsafe {
            ledger_secure_sdk_sys::nbgl_useCaseHome(
                self.app_name.as_ptr() as *const core::ffi::c_char,
                self.icon.unwrap() as *const nbgl_icon_details_t,
                core::ptr::null(),
                true as bool,
                transmute((|| settings()) as fn()),
                transmute((|| exit_app(12)) as fn()),
            );
            
        }
    }

    pub fn get_events<T: TryFrom<ApduHeader>>(&mut self) -> Event<T> 
    where Reply: From<<T as TryFrom<ApduHeader>>::Error>
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

