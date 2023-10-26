use ledger_sdk_sys::seph as sys_seph;
use ledger_sdk_sys::*;

use crate::io::{ApduHeader, Comm, Event};

pub use ledger_sdk_sys::BOLOS_UX_CANCEL;
pub use ledger_sdk_sys::BOLOS_UX_CONTINUE;
pub use ledger_sdk_sys::BOLOS_UX_ERROR;
pub use ledger_sdk_sys::BOLOS_UX_IGNORE;
pub use ledger_sdk_sys::BOLOS_UX_OK;
pub use ledger_sdk_sys::BOLOS_UX_REDRAW;

fn os_ux_rs(params: &bolos_ux_params_t) {
    unsafe { os_ux(params as *const bolos_ux_params_t as *mut bolos_ux_params_t) };
}

#[repr(u8)]
pub enum UxEvent {
    Event = BOLOS_UX_EVENT,
    Keyboard = BOLOS_UX_KEYBOARD,
    WakeUp = BOLOS_UX_WAKE_UP,
    ValidatePIN = BOLOS_UX_VALIDATE_PIN,
    LastID = BOLOS_UX_LAST_ID,
}

impl UxEvent {
    pub fn request(&self) -> u32 {
        let mut params = bolos_ux_params_t::default();
        params.ux_id = match self {
            Self::Event => Self::Event as u8,
            Self::Keyboard => Self::Keyboard as u8,
            Self::WakeUp => Self::WakeUp as u8,
            Self::ValidatePIN => {
                // Perform pre-wake up
                params.ux_id = Self::WakeUp as u8;
                os_ux_rs(&params);

                Self::ValidatePIN as u8
            }
            Self::LastID => Self::LastID as u8,
        };

        os_ux_rs(&params);

        match self {
            Self::ValidatePIN => Self::block(),
            _ => unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) as u32 },
        }
    }

    pub fn block() -> u32 {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            if unsafe { os_sched_is_running(TASK_SUBTASKS_START as u32) as u8 } != BOLOS_TRUE as u8
            {
                let mut spi_buffer = [0u8; 128];
                sys_seph::send_general_status();
                sys_seph::seph_recv(&mut spi_buffer, 0);
                UxEvent::Event.request();
            } else {
                unsafe { os_sched_yield(BOLOS_UX_OK as u8) };
            }
            ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        }
        ret
    }

    pub fn block_and_get_event<T: TryFrom<ApduHeader>>(comm: &mut Comm) -> (u32, Option<Event<T>>) {
        let mut ret = unsafe { os_sched_last_status(TASK_BOLOS_UX as u32) } as u32;
        let mut event = None;
        while ret == BOLOS_UX_IGNORE || ret == BOLOS_UX_CONTINUE {
            if unsafe { os_sched_is_running(TASK_SUBTASKS_START as u32) as u8 } != BOLOS_TRUE as u8
            {
                let mut spi_buffer = [0u8; 128];
                seph::send_general_status();
                seph::seph_recv(&mut spi_buffer, 0);
                event = comm.decode_event(&mut spi_buffer);

                UxEvent::Event.request();

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
