// NBGL callback indirection layer
// These callbacks allow nbgl.rs and related modules to interact with the singleton of the
// communication interface without holding a reference to it.
// This decouples the NBGL layer from the concrete communication backend.

use crate::io::{ApduHeader, Reply};

pub type NbglNextEventAheadCb = fn() -> bool; // returns true if APDU detected
pub type NbglFetchApduHeaderCb = fn() -> Option<ApduHeader>;
pub type NbglReplyStatusCb = fn(reply: Reply);

/// Aggregated NBGL callbacks. Stored as a single static to reduce the number of
/// unsafe mutable statics and keep registration atomic.
struct NbglCallbacks {
    next_event_ahead: NbglNextEventAheadCb,
    fetch_apdu_header: NbglFetchApduHeaderCb,
    reply_status: NbglReplyStatusCb,
}

static mut NBGL_CALLBACKS: Option<NbglCallbacks> = None;

/// Public API to register NBGL callbacks.
/// Must be called before using higher-level NBGL features. This is automatically invoked
/// via `Comm::new_with_data_size`, but exposed for alternative backends.
pub fn nbgl_register_callbacks(
    next_event_ahead: NbglNextEventAheadCb,
    fetch_apdu_header: NbglFetchApduHeaderCb,
    reply_status: NbglReplyStatusCb,
) {
    unsafe {
        NBGL_CALLBACKS = Some(NbglCallbacks {
            next_event_ahead,
            fetch_apdu_header,
            reply_status,
        });
    }
}

fn get_callbacks() -> &'static NbglCallbacks {
    unsafe {
        #[allow(static_mut_refs)]
        NBGL_CALLBACKS
            .as_ref()
            .expect("NBGL callbacks not registered")
    }
}

/// Returns true if an APDU was detected (used by NBGL polling loop).
pub fn nbgl_next_event_ahead() -> bool {
    (get_callbacks().next_event_ahead)()
}

/// Fetches the current APDU header if any.
pub fn nbgl_fetch_apdu_header() -> Option<ApduHeader> {
    (get_callbacks().fetch_apdu_header)()
}

/// Send a status reply through the registered backend.
pub fn nbgl_reply_status(reply: Reply) {
    (get_callbacks().reply_status)(reply);
}
