//! Callback integration for NBGL / IO handling extracted from `io_new`.
//!
//! This module holds the erased pointer to the current `Comm` instance and the
//! generic callback wrappers that are registered through `nbgl_register_callbacks`.

use crate::io_legacy::{ApduHeader, Reply};

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
    &mut *(CURRENT_COMM as *mut Comm<N>)
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

pub(super) fn next_event_ahead_impl<const N: usize>() -> bool {
    let comm = unsafe { get_comm::<N>() };
    // If there's already a pending APDU, return true immediately without
    // fetching another event. This prevents consuming the same APDU repeatedly
    // when ux_sync_wait loops with exit_on_apdu=false.
    if comm.pending_apdu {
        return true;
    }
    match comm.next_event().into_type() {
        DecodedEventType::Apdu {
            header,
            offset,
            length,
        } => {
            comm.pending_apdu = true;
            comm.pending_header = header;
            comm.pending_offset = offset;
            comm.pending_length = length;
            return true;
        }
        _ => {}
    }
    false
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
