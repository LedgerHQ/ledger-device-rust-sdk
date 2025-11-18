//! NBGL module
//!
//! This module provides a safe Rust interface to the NBGL library.
//! It includes functions and structures to create and manage UI elements,
//! handle user interactions, and display information on Ledger devices.

use crate::io::{ApduHeader, Event};
use crate::io_callbacks::nbgl_next_event_ahead;
use crate::nvm::*;
use const_zero::const_zero;
extern crate alloc;
use alloc::ffi::CString;
use alloc::{vec, vec::Vec};
use core::ffi::{c_char, c_int};
use core::mem::transmute;
use ledger_secure_sdk_sys::*;

pub mod nbgl_action;
pub mod nbgl_address_review;
pub mod nbgl_advance_review;
pub mod nbgl_choice;
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub mod nbgl_generic_review;
pub mod nbgl_generic_settings;
pub mod nbgl_home_and_settings;
pub mod nbgl_keypad;
//pub mod nbgl_navigable_content;
pub mod nbgl_review;
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub mod nbgl_review_extended;
pub mod nbgl_review_status;
pub mod nbgl_spinner;
pub mod nbgl_status;
pub mod nbgl_streaming_review;

#[doc(inline)]
pub use nbgl_action::*;
#[doc(inline)]
pub use nbgl_address_review::*;
#[doc(inline)]
pub use nbgl_advance_review::*;
#[doc(inline)]
pub use nbgl_choice::*;
#[doc(inline)]
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub use nbgl_generic_review::*;
#[doc(inline)]
pub use nbgl_generic_settings::*;
#[doc(inline)]
pub use nbgl_home_and_settings::*;
#[doc(inline)]
pub use nbgl_keypad::*;
#[doc(inline)]
pub use nbgl_review::*;
//pub use nbgl_navigable_content::*; // integration issue
#[doc(inline)]
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub use nbgl_review_extended::*;
#[doc(inline)]
pub use nbgl_review_status::*;
#[doc(inline)]
pub use nbgl_spinner::*;
#[doc(inline)]
pub use nbgl_status::*;
#[doc(inline)]
pub use nbgl_streaming_review::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SyncNbgl {
    UxSyncRetApproved = 0x00,
    UxSyncRetRejected = 0x01,
    UxSyncRetQuitted = 0x02,
    UxSyncRetApduReceived = 0x03,
    UxSyncRetSkipped = 0x04,
    UxSyncRetContinue = 0x05,
    UxSyncRetPinValidated = 0x06,
    UxSyncRetPinRejected = 0x07,
    UxSyncRetError = 0xFF,
}

impl From<u8> for SyncNbgl {
    fn from(val: u8) -> SyncNbgl {
        match val {
            0x00 => SyncNbgl::UxSyncRetApproved,
            0x01 => SyncNbgl::UxSyncRetRejected,
            0x02 => SyncNbgl::UxSyncRetQuitted,
            0x03 => SyncNbgl::UxSyncRetApduReceived,
            0x04 => SyncNbgl::UxSyncRetSkipped,
            0x05 => SyncNbgl::UxSyncRetContinue,
            0x06 => SyncNbgl::UxSyncRetPinValidated,
            0x07 => SyncNbgl::UxSyncRetPinRejected,
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
            SyncNbgl::UxSyncRetSkipped => 0x04,
            SyncNbgl::UxSyncRetContinue => 0x05,
            SyncNbgl::UxSyncRetPinValidated => 0x06,
            SyncNbgl::UxSyncRetPinRejected => 0x07,
            SyncNbgl::UxSyncRetError => 0xFF,
        }
    }
}

static mut G_RET: u8 = 0;
static mut G_ENDED: bool = false;
static mut G_CONFIRM_ASK_WHEN_TRUE: bool = false;
static mut G_CONFIRM_ASK_WHEN_FALSE: bool = false;

#[derive(Default)]
struct ConfirmationStrings {
    message: CString,
    submessage: CString,
    ok_text: CString,
    ko_text: CString,
}

// Store for custom confirmation screen
// when the user accepts or rejects
static mut G_CONFIRM_SCREEN: [Option<ConfirmationStrings>; 2] = [None, None];
const G_CONFIRM_SCREEN_WHEN_TRUE_IDX: usize = 0;
const G_CONFIRM_SCREEN_WHEN_FALSE_IDX: usize = 1;
const DEFAULT_CONFIRM_MESSAGE: &str = "Do you confirm this action?";
const DEFAULT_CONFIRM_SUBMESSAGE: &str = "This action is irreversible.";
const DEFAULT_CONFIRM_OK_TEXT: &str = "Confirm";
const DEFAULT_CONFIRM_KO_TEXT: &str = "Cancel";

trait SyncNBGL: Sized {
    fn ux_sync_init(&self) {
        unsafe {
            G_RET = SyncNbgl::UxSyncRetError.into();
            G_ENDED = false;
        }
    }

    fn ux_sync_wait(&self, exit_on_apdu: bool) -> SyncNbgl {
        // Poll until an NBGL callback signals completion or (optionally) an APDU arrives.
        while unsafe { !G_ENDED } {
            let apdu_received = nbgl_next_event_ahead();
            if exit_on_apdu && apdu_received {
                return SyncNbgl::UxSyncRetApduReceived;
            }
        }
        unsafe { G_RET.into() }
    }
}

unsafe extern "C" fn choice_callback(confirm: bool) {
    if G_CONFIRM_ASK_WHEN_TRUE || G_CONFIRM_ASK_WHEN_FALSE {
        let mut idx = 0usize;
        if G_CONFIRM_ASK_WHEN_TRUE && confirm {
            idx = G_CONFIRM_SCREEN_WHEN_TRUE_IDX;
            G_RET = SyncNbgl::UxSyncRetApproved.into();
        } else if G_CONFIRM_ASK_WHEN_FALSE && !confirm {
            idx = G_CONFIRM_SCREEN_WHEN_FALSE_IDX;
            G_RET = SyncNbgl::UxSyncRetRejected.into();
        }
        let screen = G_CONFIRM_SCREEN[idx].as_ref().unwrap();
        nbgl_useCaseConfirm(
            screen.message.as_ptr() as *const c_char,
            screen.submessage.as_ptr() as *const c_char,
            screen.ok_text.as_ptr() as *const c_char,
            screen.ko_text.as_ptr() as *const c_char,
            Some(confirm_choice_callback),
        );
    } else {
        G_RET = if confirm {
            SyncNbgl::UxSyncRetApproved.into()
        } else {
            SyncNbgl::UxSyncRetRejected.into()
        };
        G_ENDED = true;
    }
}

unsafe extern "C" fn confirm_choice_callback() {
    G_ENDED = true;
}

unsafe extern "C" fn skip_callback() {
    G_RET = SyncNbgl::UxSyncRetSkipped.into();
    G_ENDED = true;
}

unsafe extern "C" fn quit_callback() {
    G_RET = SyncNbgl::UxSyncRetQuitted.into();
    G_ENDED = true;
}

unsafe extern "C" fn continue_callback() {
    G_RET = SyncNbgl::UxSyncRetContinue.into();
    G_ENDED = true;
}

#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
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

impl From<&Field<'_>> for CField {
    fn from(field: &Field) -> CField {
        CField {
            name: CString::new((*field).name).unwrap(),
            value: CString::new((*field).value).unwrap(),
        }
    }
}

impl From<&CField> for nbgl_contentTagValue_t {
    fn from(field: &CField) -> nbgl_contentTagValue_t {
        nbgl_contentTagValue_t {
            item: (*field).name.as_ptr() as *const ::core::ffi::c_char,
            value: (*field).value.as_ptr() as *const ::core::ffi::c_char,
            ..Default::default()
        }
    }
}

// impl From<Field<'_>> for nbgl_contentTagValue_t {
//     fn from(field: Field) -> nbgl_contentTagValue_t {
//         let cfield: CField = field.into();
//         cfield.into()
//     }
// }

/// Glyph structure to represent icons for NBGL
pub struct NbglGlyph<'a> {
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub is_file: bool,
    pub bitmap: &'a [u8],
}

impl<'a> NbglGlyph<'a> {
    /// Creates a new instance of `NbglGlyph`.
    /// # Arguments
    /// * `bitmap` - A byte slice representing the bitmap data of the glyph.
    /// * `width` - The width of the glyph in pixels.
    /// * `height` - The height of the glyph in pixels.
    /// * `bpp` - Bits per pixel (1, 2, or 4).
    /// * `is_file` - A boolean indicating whether the bitmap is a file path.
    /// # Returns
    /// Returns a new instance of `NbglGlyph`.
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
    /// Creates a new instance of `NbglGlyph` from a packed representation.
    /// # Arguments
    /// * `packed` - A tuple containing the bitmap, width, height, bpp, and is_file flag.
    /// # Returns
    /// Returns a new instance of `NbglGlyph`.
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

/// Transaction types for NBGL review and status screens.    
pub enum TransactionType {
    Transaction,
    Message,
    Operation,
}

/// Status types for NBGL status screens.
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

#[cfg(not(feature = "io_new"))]
/// Initialize the global reference to the Comm instance used by Nbgl.
/// This function should be called from the main function of the application.
pub fn init_comm(comm: &mut crate::io::Comm) {
    comm.nbgl_register_comm();
}

#[cfg(feature = "io_new")]
/// Initialize the global reference to the Comm instance used by Nbgl.
/// This function should be called from the main function of the application.
pub fn init_comm<const N: usize>(comm: &mut crate::io::Comm<N>) {
    comm.nbgl_register_comm();
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
