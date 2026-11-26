use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

#[cfg(not(feature = "io_new"))]
use crate::io::Reply;

#[cfg(not(feature = "io_new"))]
use crate::io::{ApduHeader, Comm, Event};

pub use ledger_secure_sdk_sys::BOLOS_UX_CANCEL;
pub use ledger_secure_sdk_sys::BOLOS_UX_CONTINUE;
pub use ledger_secure_sdk_sys::BOLOS_UX_ERROR;
pub use ledger_secure_sdk_sys::BOLOS_UX_IGNORE;
pub use ledger_secure_sdk_sys::BOLOS_UX_OK;
pub use ledger_secure_sdk_sys::BOLOS_UX_REDRAW;

unsafe extern "C" {
    pub unsafe static mut G_ux_params: bolos_ux_params_t;
}

#[repr(u8)]
pub enum UxEvent {
    Event = BOLOS_UX_EVENT,
    Keyboard = BOLOS_UX_KEYBOARD,
    WakeUp = BOLOS_UX_WAKE_UP,
    ValidatePIN = BOLOS_UX_VALIDATE_PIN,
    DelayLock = BOLOS_UX_DELAY_LOCK,
    LastID = BOLOS_UX_DELAY_LOCK + 1,
}

/// Delay [`UxEvent::DelayLock`] asks for, in ms. See [`UxEvent::delay_lock`].
pub const DEFAULT_LOCK_DELAY_MS: u32 = 10_000;

impl UxEvent {
    #[allow(unused)]
    pub fn request(&self) -> u32 {
        if let Self::DelayLock = self {
            return Self::delay_lock(DEFAULT_LOCK_DELAY_MS);
        }
        unsafe {
            // None of these events carries parameters, but `len` (the number
            // of bytes of `u` the syscall copies) must still be set: the C UX
            // lib stores the last UX status in it (lib_ux_nbgl/ux.c).
            G_ux_params.len = 0;
            G_ux_params.ux_id = match self {
                Self::Event => Self::Event as u8,
                Self::Keyboard => Self::Keyboard as u8,
                Self::WakeUp => Self::WakeUp as u8,
                Self::ValidatePIN => {
                    // Perform pre-wake up
                    G_ux_params.ux_id = Self::WakeUp as u8;
                    os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);

                    Self::ValidatePIN as u8
                }
                // Handled above.
                Self::DelayLock => Self::DelayLock as u8,
                Self::LastID => panic!("Unknown UX Event"),
            };

            os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);

            match self {
                Self::ValidatePIN => Self::block(),
                _ => os_sched_last_status(TASK_BOLOS_UX as u32) as u32,
            }
        }
    }

    /// Asks the OS to push the auto-lock back to `delay_ms` from now, and
    /// returns the UX task status.
    ///
    /// Observed on Flex: the request only moves the auto-lock deadline when
    /// `now + delay_ms` is later than the current one, so a delay shorter
    /// than the time left does nothing. An app that streams APDUs for longer
    /// than the auto-lock setting needs a delay well above the gap between two
    /// requests: with about 5 s between requests, 10 s let a Flex lock
    /// mid-stream and 60 s did not.
    pub fn delay_lock(delay_ms: u32) -> u32 {
        unsafe {
            G_ux_params.ux_id = Self::DelayLock as u8;
            G_ux_params.len = 0;
            #[cfg(any(
                ledger_device = "apex_p",
                ledger_device = "stax",
                ledger_device = "flex",
                feature = "nano_nbgl"
            ))]
            {
                G_ux_params.u.lock_delay.delay_ms = delay_ms;
                // `lock_delay` holds a single u32.
                G_ux_params.len = core::mem::size_of::<u32>() as _;
            }
            #[cfg(not(any(
                ledger_device = "apex_p",
                ledger_device = "stax",
                ledger_device = "flex",
                feature = "nano_nbgl"
            )))]
            let _ = delay_ms;

            os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);
            os_sched_last_status(TASK_BOLOS_UX as u32) as u32
        }
    }

    pub fn block() -> u32 {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            let mut spi_buffer = [0u8; 256];
            sys_seph::io_rx(&mut spi_buffer, true);
            UxEvent::Event.request();
            ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        }
        ret
    }

    #[cfg(not(feature = "io_new"))]
    pub fn block_and_get_event<T>(comm: &mut Comm) -> (u32, Option<Event<T>>)
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        let mut event = None;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            let status = sys_seph::io_rx(&mut comm.rx_buffer, true);
            if status > 0 {
                event = comm.decode_event(status)
            }

            UxEvent::Event.request();

            if let Option::Some(Event::Command(_)) = event {
                return (ret, event);
            }
            ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        }
        (ret, event)
    }
}
