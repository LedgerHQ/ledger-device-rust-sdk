use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

use crate::io::Reply;
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

impl UxEvent {
    #[allow(unused)]
    pub fn request(&self, val: Option<u32>) -> u32 {
        unsafe {
            //let mut params = bolos_ux_params_t::default();
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
                Self::DelayLock => {
                    #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
                    {
                        G_ux_params.u = bolos_ux_params_s__bindgen_ty_1 {
                            lock_delay: bolos_ux_params_s__bindgen_ty_1__bindgen_ty_3 {
                                delay_ms: val.unwrap_or(10000),
                            },
                        };
                    }
                    Self::DelayLock as u8
                }
                Self::LastID => panic!("Unknown UX Event"),
            };

            os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);

            match self {
                Self::ValidatePIN => Self::block(),
                _ => os_sched_last_status(TASK_BOLOS_UX as u32) as u32,
            }
        }
    }

    pub fn block() -> u32 {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            if unsafe { os_sched_is_running(TASK_SUBTASKS_START as u32) }
                != BOLOS_TRUE.try_into().unwrap()
            {
                let mut spi_buffer = [0u8; 256];
                sys_seph::send_general_status();
                sys_seph::seph_recv(&mut spi_buffer, 0);
                UxEvent::Event.request(None);
            } else {
                unsafe { os_sched_yield(BOLOS_UX_OK as u8) };
            }
            ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        }
        ret
    }

    pub fn block_and_get_event<T>(comm: &mut Comm) -> (u32, Option<Event<T>>)
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        let mut event = None;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            if unsafe { os_sched_is_running(TASK_SUBTASKS_START as u32) }
                != BOLOS_TRUE.try_into().unwrap()
            {
                let mut spi_buffer = [0u8; 256];
                seph::send_general_status();
                seph::seph_recv(&mut spi_buffer, 0);
                event = comm.decode_event(&mut spi_buffer);

                UxEvent::Event.request(None);

                if let Option::Some(Event::Command(_)) = event {
                    return (ret, event);
                }
            } else {
                unsafe { os_sched_yield(BOLOS_UX_OK as u8) };
            }
            ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        }
        (ret, event)
    }
}
