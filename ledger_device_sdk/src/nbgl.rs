use crate::io::{ApduHeader, Comm, Event, Reply};
use crate::nvm::*;
use const_zero::const_zero;
extern crate alloc;
use alloc::ffi::CString;
use alloc::{vec, vec::Vec};
use core::ffi::{c_char, c_int};
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

pub mod nbgl_address_review;
pub mod nbgl_choice;
#[cfg(any(target_os = "stax", target_os = "flex"))]
pub mod nbgl_generic_review;
pub mod nbgl_home_and_settings;
pub mod nbgl_review;
pub mod nbgl_review_status;
pub mod nbgl_spinner;
pub mod nbgl_status;
pub mod nbgl_streaming_review;

pub use nbgl_address_review::*;
pub use nbgl_choice::*;
#[cfg(any(target_os = "stax", target_os = "flex"))]
pub use nbgl_generic_review::*;
pub use nbgl_home_and_settings::*;
pub use nbgl_review::*;
pub use nbgl_review_status::*;
pub use nbgl_spinner::*;
pub use nbgl_status::*;
pub use nbgl_streaming_review::*;

static mut COMM_REF: Option<&mut Comm> = None;

#[derive(Copy, Clone)]
enum SyncNbgl {
    UxSyncRetApproved = 0x00,
    UxSyncRetRejected = 0x01,
    UxSyncRetQuitted = 0x02,
    UxSyncRetApduReceived = 0x03,
    UxSyncRetError = 0xFF,
}

impl From<u8> for SyncNbgl {
    fn from(val: u8) -> SyncNbgl {
        match val {
            0x00 => SyncNbgl::UxSyncRetApproved,
            0x01 => SyncNbgl::UxSyncRetRejected,
            0x02 => SyncNbgl::UxSyncRetQuitted,
            0x03 => SyncNbgl::UxSyncRetApduReceived,
            _ => SyncNbgl::UxSyncRetError,
        }
    }
}

impl From<SyncNbgl> for u8 {
    fn from(val: SyncNbgl) -> u8 {
        match val {
            SyncNbgl::UxSyncRetApproved => 0x00,
            SyncNbgl::UxSyncRetRejected => 0x01,
            SyncNbgl::UxSyncRetQuitted => 0x02,
            SyncNbgl::UxSyncRetApduReceived => 0x03,
            SyncNbgl::UxSyncRetError => 0xFF,
        }
    }
}

static mut G_RET: u8 = 0;
static mut G_ENDED: bool = false;

trait SyncNBGL: Sized {
    fn ux_sync_init(&self) {
        unsafe {
            G_RET = SyncNbgl::UxSyncRetError.into();
            G_ENDED = false;
        }
    }

    fn ux_sync_wait(&self, exit_on_apdu: bool) -> SyncNbgl {
        unsafe {
            if let Some(comm) = (*(&raw mut COMM_REF)).as_mut() {
                while !G_ENDED {
                    let apdu_received = comm.next_event_ahead::<ApduHeader>();
                    if exit_on_apdu && apdu_received {
                        return SyncNbgl::UxSyncRetApduReceived;
                    }
                }
                return G_RET.into();
            } else {
                panic!("COMM_REF not initialized");
            }
        }
    }
}

unsafe extern "C" fn choice_callback(confirm: bool) {
    G_RET = if confirm {
        SyncNbgl::UxSyncRetApproved.into()
    } else {
        SyncNbgl::UxSyncRetRejected.into()
    };
    G_ENDED = true;
}

unsafe extern "C" fn quit_callback() {
    G_RET = SyncNbgl::UxSyncRetQuitted.into();
    G_ENDED = true;
}

#[cfg(any(target_os = "stax", target_os = "flex"))]
unsafe extern "C" fn rejected_callback() {
    G_RET = SyncNbgl::UxSyncRetRejected.into();
    G_ENDED = true;
}

pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

struct CField {
    pub name: CString,
    pub value: CString,
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

pub enum TransactionType {
    Transaction,
    Message,
    Operation,
}

pub enum StatusType {
    Transaction,
    Message,
    Operation,
    Address,
}

impl StatusType {
    fn transaction_type(&self) -> Option<TransactionType> {
        match self {
            StatusType::Transaction => Some(TransactionType::Transaction),
            StatusType::Message => Some(TransactionType::Message),
            StatusType::Operation => Some(TransactionType::Operation),
            StatusType::Address => None,
        }
    }
}

trait ToMessage {
    fn to_message(&self, success: bool) -> nbgl_reviewStatusType_t;
}

impl TransactionType {
    fn to_c_type(&self, skippable: bool) -> nbgl_operationType_t {
        let mut tx_type = match self {
            TransactionType::Transaction => TYPE_TRANSACTION.into(),
            TransactionType::Message => TYPE_MESSAGE.into(),
            TransactionType::Operation => TYPE_OPERATION.into(),
        };
        if skippable {
            tx_type |= SKIPPABLE_OPERATION;
        }
        tx_type
    }
}

impl ToMessage for TransactionType {
    fn to_message(&self, success: bool) -> nbgl_reviewStatusType_t {
        match (self, success) {
            (TransactionType::Transaction, true) => STATUS_TYPE_TRANSACTION_SIGNED,
            (TransactionType::Transaction, false) => STATUS_TYPE_TRANSACTION_REJECTED,
            (TransactionType::Message, true) => STATUS_TYPE_MESSAGE_SIGNED,
            (TransactionType::Message, false) => STATUS_TYPE_MESSAGE_REJECTED,
            (TransactionType::Operation, true) => STATUS_TYPE_OPERATION_SIGNED,
            (TransactionType::Operation, false) => STATUS_TYPE_OPERATION_REJECTED,
        }
    }
}

impl ToMessage for StatusType {
    fn to_message(&self, success: bool) -> nbgl_reviewStatusType_t {
        match self {
            StatusType::Address => {
                if success {
                    STATUS_TYPE_ADDRESS_VERIFIED
                } else {
                    STATUS_TYPE_ADDRESS_REJECTED
                }
            }
            _ => self
                .transaction_type()
                .expect("Should be a transaction type")
                .to_message(success),
        }
    }
}

/// Initialize the global COMM_REF variable with the provided Comm instance.
/// This function should be called from the main function of the application.
/// The COMM_REF variable is used by the NBGL API to detect touch events and
/// APDU reception.
pub fn init_comm(comm: &mut Comm) {
    unsafe {
        COMM_REF = Some(transmute(comm));
    }
}

#[derive(Copy, Clone)]
pub enum TuneIndex {
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
