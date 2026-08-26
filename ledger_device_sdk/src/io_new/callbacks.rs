//! Callback integration for NBGL / IO handling extracted from `io_new`.
//!
//! This module holds the erased pointer to the current `Comm` instance and the
//! generic callback wrappers that are registered through `nbgl_register_callbacks`.

use crate::io_legacy::{ApduHeader, Reply, StatusWords};

use super::bolos::handle_bolos_apdu;
use super::{Comm, DecodedEventType};

// Erased pointer to the Comm instance (generic parameter erased).
static mut CURRENT_COMM: *mut core::ffi::c_void = core::ptr::null_mut();

// Type-erased panic reply function.
static mut PANIC_REPLY_FN: Option<fn(Reply)> = None;

pub(super) fn set_comm<const N: usize>(comm: &mut Comm<N>) {
    unsafe {
        CURRENT_COMM = (comm as *mut Comm<N>) as *mut core::ffi::c_void;
    }
}

pub(super) fn clear_comm() {
    unsafe {
        CURRENT_COMM = core::ptr::null_mut();
    }
}

#[allow(dead_code)]
pub(super) fn is_comm_null() -> bool {
    unsafe { CURRENT_COMM.is_null() }
}

// Converts the pointer back to the concrete Comm<N> type.
unsafe fn get_comm<const N: usize>() -> &'static mut Comm<N> {
    unsafe { &mut *(CURRENT_COMM as *mut Comm<N>) }
}

/// Register a type-erased panic handler for the current Comm instance.
pub fn register_panic_handler<const N: usize>() {
    unsafe {
        PANIC_REPLY_FN = Some(panic_reply_impl::<N>);
    }
}

pub(super) fn clear_panic_handler() {
    unsafe {
        PANIC_REPLY_FN = None;
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
    let comm = unsafe { get_comm::<N>() };
    let _ = comm.begin_response().send(reply);
}

// Implementation wrappers specialized per const N.

/// Fetch and process one event while an NBGL screen is displayed, and report
/// whether it was an APDU command the caller should leave the screen for.
///
/// This is called from `ux_sync_wait` both while the application is idle (an
/// incoming APDU is then the normal way of receiving work) and while it is
/// processing a command (an incoming APDU is then a double APDU).
pub(super) fn next_event_ahead_impl<const N: usize>() -> bool {
    let comm = unsafe { get_comm::<N>() };

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

    match comm.next_event().into_type() {
        DecodedEventType::Apdu {
            header,
            offset,
            length,
        } => {
            // BOLOS internal APDUs (CLA = 0xB0) are answered inline whatever
            // the state, the way `next_command` does, so that OS level requests
            // keep working while a screen is displayed.
            if header.cla == 0xB0 {
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
        DecodedEventType::ApduError(e) => {
            let intruder_apdu_type = comm.apdu_type;
            comm.reject_apdu(intruder_apdu_type, StatusWords::from(e));
            comm.apdu_type = in_flight_apdu_type;
            false
        }
        _ => false,
    }
}

pub(super) fn fetch_apdu_header_impl<const N: usize>() -> Option<ApduHeader> {
    let comm = unsafe { get_comm::<N>() };
    if comm.pending_apdu {
        Some(comm.pending_header)
    } else {
        None
    }
}

pub(super) fn reply_status_impl<const N: usize>(reply: Reply) {
    let comm = unsafe { get_comm::<N>() };
    if comm.pending_apdu {
        comm.pending_apdu = false;
    }
    let _ = comm.begin_response().send(reply);
}
