//! Callback integration for NBGL / IO handling extracted from `io_new`.
//!
//! This module holds the erased pointers to the `Comm` instance and the
//! generic callback wrappers that are registered through `nbgl_register_callbacks`.
//!
//! # Contract
//!
//! The NBGL callbacks receive events into, and answer BOLOS APDUs from, the
//! `Comm` buffer, which also holds the data of the command in flight. They
//! must therefore never run while the application can still read that data,
//! or while `Comm` is itself receiving or decoding an event.
//!
//! This is guaranteed by the borrow checker: every NBGL flow that polls events
//! takes a `&mut Comm`, and lends it to the callbacks with
//! [`Comm::lend_to_nbgl`] for as long as it is displayed. The callbacks reach
//! the `Comm` only through that loan, and panic if there is none, so a flow
//! that forgot to borrow the `Comm` fails loudly instead of aliasing it.

use crate::io_legacy::{ApduHeader, Reply, StatusWords, is_bolos_apdu_allowed_in_flight};

use super::bolos::handle_bolos_apdu;
use super::{Comm, RawEvent};

// Erased pointer to the Comm instance (generic parameter erased), set once by
// `init_comm`. Only the panic reply falls back to it, as a panic can happen
// outside of any NBGL flow.
static mut CURRENT_COMM: *mut core::ffi::c_void = core::ptr::null_mut();

// Erased pointer to the Comm lent by the NBGL flow being displayed, or null
// when no flow is. It is derived from the flow's `&mut Comm` and only used
// while that borrow is live.
static mut LENT_COMM: *mut core::ffi::c_void = core::ptr::null_mut();

// Type-erased panic reply function.
static mut PANIC_REPLY_FN: Option<fn(Reply)> = None;

pub(super) fn set_comm<const N: usize>(comm: &mut Comm<N>) {
    unsafe {
        CURRENT_COMM = (comm as *mut Comm<N>) as *mut core::ffi::c_void;
    }
}

#[allow(dead_code)]
pub(super) fn is_comm_null() -> bool {
    unsafe { CURRENT_COMM.is_null() }
}

/// Lends `comm` to the NBGL callbacks, returning the previous loan to be given
/// back to [`end_loan`].
#[cfg(any(
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p",
    feature = "nano_nbgl"
))]
pub(super) fn lend<const N: usize>(comm: &mut Comm<N>) -> *mut core::ffi::c_void {
    unsafe {
        let prev = LENT_COMM;
        LENT_COMM = (comm as *mut Comm<N>) as *mut core::ffi::c_void;
        prev
    }
}

#[cfg(any(
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p",
    feature = "nano_nbgl"
))]
pub(super) fn end_loan(prev: *mut core::ffi::c_void) {
    unsafe {
        LENT_COMM = prev;
    }
}

// Converts the lent pointer back to the concrete Comm<N> type.
// Panics if no NBGL flow lent its Comm.
unsafe fn get_lent_comm<const N: usize>() -> &'static mut Comm<N> {
    unsafe { (LENT_COMM as *mut Comm<N>).as_mut() }
        .expect("NBGL flow polled events without borrowing the Comm")
}

/// Register a type-erased panic handler for the current Comm instance.
pub fn register_panic_handler<const N: usize>() {
    unsafe {
        PANIC_REPLY_FN = Some(panic_reply_impl::<N>);
    }
}

/// Send a panic reply if a Comm instance is registered.
pub fn send_panic_reply(reply: Reply) {
    unsafe {
        if let Some(f) = PANIC_REPLY_FN {
            f(reply);
        }
        // If no panic handler is registered, silently skip (device is already panicking)
    }
}

fn panic_reply_impl<const N: usize>(reply: Reply) {
    // Prefer the loan of the flow being displayed, if any.
    let comm = unsafe {
        let ptr = if LENT_COMM.is_null() {
            CURRENT_COMM
        } else {
            LENT_COMM
        };
        (ptr as *mut Comm<N>).as_mut()
    }
    .expect("No Comm instance registered");
    let _ = comm.begin_response().send(reply);
}

// Implementation wrappers specialized per const N.

/// Fetch and process one event while an NBGL screen is displayed, and report
/// whether it was an APDU command the caller should leave the screen for.
///
/// This is called from `ux_sync_wait` both while the application is idle (an
/// incoming APDU is then the normal way of receiving work) and while it is
/// processing a command (an incoming APDU is then a double APDU).
///
/// It overwrites the `Comm` buffer, which is sound because it only runs while
/// a flow has lent its `&mut Comm` (see the module documentation): the data of
/// a command in flight can no longer be read by then.
pub(super) fn next_event_ahead_impl<const N: usize>() -> bool {
    let comm = unsafe { get_lent_comm::<N>() };

    // Decoding an APDU overwrites `apdu_type` with the transport it arrived on.
    // Anything handled or rejected below is not the command the application is
    // working on, so its transport is restored before returning; otherwise the
    // in-flight command's response would go out on the intruder's channel.
    let in_flight_apdu_type = comm.apdu_type;

    // An APDU detected on an earlier iteration that nobody consumed means the
    // displayed screen does not exit on APDU. Answer it, so that polling — and
    // therefore the screen itself — keeps running. No command can be in flight
    // here, as one would have been rejected on the spot below.
    if comm.pending_apdu {
        comm.pending_apdu = false;
        comm.reject_apdu(in_flight_apdu_type, StatusWords::CmdNotAccepted);
        return false;
    }

    match comm.recv_event() {
        RawEvent::Apdu {
            header,
            offset,
            length,
        } => {
            // BOLOS internal APDUs (CLA = 0xB0) are answered inline, the way
            // `next_command` does, so that OS level requests keep working while
            // a screen is displayed. While a command is in flight only
            // GET_VERSION is: the others are handled as double APDUs below.
            if header.cla == 0xB0
                && (!comm.apdu_in_progress
                    || is_bolos_apdu_allowed_in_flight(header.cla, header.ins))
            {
                let in_progress = comm.apdu_in_progress;
                handle_bolos_apdu::<N>(comm, header.ins, header.p1, header.p2);
                // The BOLOS reply must not be taken for the reply to the
                // command the application is still processing.
                comm.apdu_in_progress = in_progress;
                comm.apdu_type = in_flight_apdu_type;
                return false;
            }
            // An APDU arriving while a command is still being processed is a
            // double APDU. Answer it on this very iteration: deferring to the
            // next one loses it entirely if the screen completes in between.
            if comm.apdu_in_progress {
                let intruder_apdu_type = comm.apdu_type;
                comm.reject_apdu(intruder_apdu_type, StatusWords::CmdNotAccepted);
                comm.apdu_type = in_flight_apdu_type;
                return false;
            }
            comm.pending_apdu = true;
            comm.pending_header = header;
            comm.pending_offset = offset;
            comm.pending_length = length;
            true
        }
        // Answer malformed APDUs instead of leaving the host without a status
        // word, as `next_command` does outside of screens.
        RawEvent::ApduError(e) => {
            let intruder_apdu_type = comm.apdu_type;
            comm.reject_apdu(intruder_apdu_type, StatusWords::from(e));
            comm.apdu_type = in_flight_apdu_type;
            false
        }
        _ => false,
    }
}

pub(super) fn fetch_apdu_header_impl<const N: usize>() -> Option<ApduHeader> {
    let comm = unsafe { get_lent_comm::<N>() };
    if comm.pending_apdu {
        Some(comm.pending_header)
    } else {
        None
    }
}

pub(super) fn reply_status_impl<const N: usize>(reply: Reply) {
    let comm = unsafe { get_lent_comm::<N>() };
    if comm.pending_apdu {
        comm.pending_apdu = false;
    }
    let _ = comm.begin_response().send(reply);
}
