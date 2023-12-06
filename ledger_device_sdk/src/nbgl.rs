use core::mem::transmute;
use ledger_secure_sdk_sys::nbgl_icon_details_t;

pub struct Home<'a> {
    app_name: &'static str,
    icon: Option<&'a nbgl_icon_details_t>,
    tagline: Option<&'static str>,
    with_settings: bool,
    top_right_cb: Option<fn()>,
    quit_cb: Option<fn()>,
}

impl<'a> Home<'a> {
    pub fn new() -> Home<'a> {
        Home {
            app_name: "AppName\0",
            icon: None,
            tagline: None,
            with_settings: false,
            top_right_cb: None,
            quit_cb: None,
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

    pub fn show(&self) {
        // let bop: unsafe extern "C" fn() = self.top_right_cb.unwrap() as unsafe extern "C" fn();
        unsafe {
            ledger_secure_sdk_sys::nbgl_useCaseHome(
                self.app_name.as_ptr() as *const core::ffi::c_char,
                self.icon.unwrap() as *const nbgl_icon_details_t,
                core::ptr::null(),
                self.with_settings,
                // match self.top_right_cb {
                //     None => None,
                //     Some(f) => Some(f),
                // },
                transmute(self.top_right_cb),
                // Some(bop),
                transmute(self.quit_cb),
            )
        }
    }
}

extern "C" fn do_nothing() {}
extern "C" fn exit() {
    ledger_secure_sdk_sys::exit_app(0);
}
pub fn home(image: &[u8]) {
    let icon = ledger_secure_sdk_sys::nbgl_icon_details_t {
        width: 64,
        height: 64,
        bpp: 2,
        isFile: true,
        bitmap: image.as_ptr(),
    };
    unsafe {
        // appName: *const ::core::ffi::c_char,
        // appIcon: *const nbgl_icon_details_t,
        // tagline: *const ::core::ffi::c_char,
        // withSettings: bool,
        // topRightCallback: nbgl_callback_t,
        // quitCallback: nbgl_callback_t
        ledger_secure_sdk_sys::nbgl_useCaseHome(
            "AppName\0".as_ptr() as *const core::ffi::c_char,
            &icon,
            core::ptr::null(),
            false,
            Some(do_nothing),
            Some(exit),
        );
    }
}
