use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use crate::seph::PacketTypes;

mod event;
use event::RawEvent;

mod bolos;
pub(crate) mod callbacks;
use bolos::handle_bolos_apdu;

use crate::io_legacy::is_bolos_apdu_allowed_in_flight;
pub use crate::io_legacy::{ApduHeader, Reply, StatusWords};

use crate::io_callbacks::nbgl_register_callbacks;

use ledger_secure_sdk_sys::seph as sys_seph;

/// Default buffer size for `Comm` when no custom size is specified.
pub const DEFAULT_BUF_SIZE: usize = 273;

/// Set once a `Comm` has been created. It is never reset, so that the only
/// `Comm` that can ever exist is the one registered with NBGL by [`init_comm`].
// SAFETY: the runtime is single-threaded, so direct reads/writes are safe.
static mut COMM_CREATED: bool = false;

/// Static storage container for a `Comm<N>` instance.
///
/// This type provides safe static storage for `Comm` instances, ensuring that
/// the pointer registered with NBGL callbacks remains valid for the lifetime
/// of the application.
///
/// Use the [`define_comm!`] macro to declare instances of this type.
///
/// # Example
///
/// ```ignore
/// ledger_device_sdk::define_comm!(COMM);
/// // or with custom buffer size:
/// ledger_device_sdk::define_comm!(COMM, 512);
/// ```
pub struct CommStorage<const N: usize = DEFAULT_BUF_SIZE> {
    inner: UnsafeCell<MaybeUninit<Comm<N>>>,
}

// SAFETY: single-threaded runtime; `inner` is only written by `init`, which
// consumes a `Comm`. As at most one `Comm` is ever created (see
// COMM_CREATED), write access to `inner` happens at most once.
unsafe impl<const N: usize> Sync for CommStorage<N> {}

impl<const N: usize> CommStorage<N> {
    /// Creates a new uninitialized `CommStorage`.
    ///
    /// This is a const fn, suitable for use in static declarations.
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Initializes the storage with a `Comm<N>` instance and returns a static reference.
    ///
    /// This can happen only once in total, across all `CommStorage` instances:
    /// it consumes the only `Comm` that [`Comm::new`] lets the application
    /// create.
    ///
    /// # Safety
    ///
    /// This method must be called on a static `CommStorage` instance to ensure
    /// the returned reference has a `'static` lifetime.
    pub fn init(&'static self, comm: Comm<N>) -> &'static mut Comm<N> {
        // SAFETY: `comm` is the only `Comm` that can ever exist (see
        // COMM_CREATED), so this point is reached at most once, and no other
        // reference to `inner` exists. The storage is static, so the returned
        // reference is valid for 'static.
        unsafe {
            let ptr = self.inner.get();
            (*ptr).write(comm);
            (*ptr).assume_init_mut()
        }
    }
}

/// Declares a static `CommStorage` with the given name and optional buffer size.
///
/// This macro creates static storage for a `Comm<N>` instance. The storage must be
/// initialized using [`init_comm`] before use.
///
/// # Usage
///
/// ```ignore
/// // With default buffer size (273 bytes):
/// ledger_device_sdk::define_comm!(COMM);
///
/// // With custom buffer size:
/// ledger_device_sdk::define_comm!(COMM, 512);
/// ```
///
/// # Example
///
/// ```ignore
/// use ledger_device_sdk::{define_comm, init_comm};
///
/// define_comm!(COMM);
///
/// fn main() {
///     let comm = init_comm(&COMM);
///     // Use comm...
/// }
/// ```
#[macro_export]
macro_rules! define_comm {
    ($name:ident) => {
        static $name: $crate::io::CommStorage<{ $crate::io::DEFAULT_BUF_SIZE }> =
            $crate::io::CommStorage::new();
    };
    ($name:ident, $size:expr) => {
        static $name: $crate::io::CommStorage<$size> = $crate::io::CommStorage::new();
    };
}

#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use ledger_secure_sdk_sys::buttons::{ButtonEvent, ButtonsState};

/// An event returned by [`Comm::next_event`].
pub enum Event<'a, const N: usize = DEFAULT_BUF_SIZE> {
    /// An APDU command for the application, to be answered before the next
    /// one can be received.
    Command(Command<'a, N>),
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    Button(ButtonEvent),
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    Touch,
    Ticker,
    /// An event the SDK handled entirely, such as a USB or BLE event, or an
    /// APDU it answered itself. There is nothing to do, but the application
    /// gets a chance to run, for instance to process data that callbacks of an
    /// application-side IO stack received during the event.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommError {
    Overflow,
    IoError,
}

pub struct Comm<const N: usize = DEFAULT_BUF_SIZE> {
    buf: [u8; N],
    expected_cla: Option<u8>,

    apdu_type: u8,
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    buttons: ButtonsState,
    // Pending APDU state (set by next_event_ahead callback path). When set, the buffer
    // currently holds an APDU event that must be consumed before any further io_rx call.
    pending_apdu: bool,
    pending_header: ApduHeader,
    pending_offset: usize,
    pending_length: usize,
    /// True from the moment a command is handed to the application until the
    /// application replies to it. This is what makes a second incoming APDU a
    /// *double* APDU rather than the normal way of receiving work.
    apdu_in_progress: bool,
}

impl<const N: usize> Comm<N> {
    /// Creates the application's `Comm`. Applications normally get it through
    /// [`init_comm`] instead, which also registers it with NBGL.
    ///
    /// # Panics
    ///
    /// Panics if a `Comm` has already been created: only one instance can ever
    /// exist, which is what makes holding a `&mut Comm` proof that nothing else
    /// is using the communication buffer.
    pub fn new() -> Self {
        // SAFETY: single-threaded runtime; no concurrent access is possible.
        if unsafe { COMM_CREATED } {
            panic!("Comm already created. Only one Comm instance can exist.");
        }
        unsafe { COMM_CREATED = true };

        Self {
            buf: [0; N],
            expected_cla: None,
            apdu_type: PacketTypes::PacketTypeNone as u8,
            #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
            buttons: ButtonsState::default(),
            pending_apdu: false,
            pending_header: ApduHeader {
                cla: 0,
                ins: 0,
                p1: 0,
                p2: 0,
            },
            pending_offset: 0,
            pending_length: 0,
            apdu_in_progress: false,
        }
    }

    /// Answer `sw` to an APDU that cannot be delivered to the application, on
    /// the transport it arrived on.
    ///
    /// This deliberately bypasses [`Comm::begin_response`]: it must stage
    /// nothing into the shared buffer and must leave `apdu_type`,
    /// `pending_apdu` and `apdu_in_progress` alone, as they belong to the
    /// command that is still being processed.
    pub(crate) fn reject_apdu<T: Into<Reply>>(&self, packet_type: u8, sw: T) {
        let sw: u16 = sw.into().0;
        let resp = sw.to_be_bytes();
        let _ = sys_seph::io_tx(packet_type, resp.as_ref(), resp.len());
    }

    pub(crate) fn nbgl_register_comm(&mut self) {
        // Register NBGL callbacks if not already set and record current Comm singleton.
        callbacks::set_comm::<N>(self);
        nbgl_register_callbacks(
            callbacks::next_event_ahead_impl::<N>,
            callbacks::fetch_apdu_header_impl::<N>,
            callbacks::reply_status_impl::<N>,
        );
    }

    /// Lends this `Comm` to the NBGL callbacks while `f` displays a flow.
    ///
    /// The callbacks reach the `Comm` only through this loan. Since it takes
    /// `&mut self`, the borrow checker guarantees that nothing else uses the
    /// `Comm`, and in particular that nobody still reads the data of the
    /// command in flight, while the callbacks receive events into its buffer.
    #[cfg(any(
        target_os = "stax",
        target_os = "flex",
        target_os = "apex_p",
        feature = "nano_nbgl"
    ))]
    pub(crate) fn lend_to_nbgl<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let prev = callbacks::lend::<N>(self);
        let ret = f();
        callbacks::end_loan(prev);
        ret
    }

    /// Receive into the internal buffer. Returns a read-only guard.
    fn recv(&mut self, check_se_event: bool) -> Result<Rx<'_, N>, CommError> {
        let result = sys_seph::io_rx(&mut self.buf, check_se_event);
        if result < 0 {
            return Err(CommError::IoError);
        }
        Ok(Rx {
            comm: self,
            len: result as usize,
        })
    }

    /// Start building a message in the internal buffer. Returns a mutable guard.
    pub fn begin_response(&mut self) -> CommandResponse<'_, N> {
        CommandResponse { comm: self, len: 0 }
    }

    /// Send directly from an external slice, bypassing the internal buffer.
    pub fn send<T: Into<Reply>>(&mut self, data: &[u8], reply: T) -> Result<(), CommError> {
        self.begin_response().extend(data)?.send(reply).unwrap();
        Ok(())
    }

    /// Receive and decode the next event, without applying any APDU policy.
    ///
    /// An APDU left pending by the NBGL callbacks is returned first, as it is
    /// still in the buffer.
    pub(crate) fn recv_event(&mut self) -> RawEvent {
        if self.pending_apdu {
            self.pending_apdu = false;
            return RawEvent::Apdu {
                header: self.pending_header,
                offset: self.pending_offset,
                length: self.pending_length,
            };
        }
        self.recv(true).unwrap().decode_event()
    }

    /// Receive the next event, answering on the spot the APDUs that are not for
    /// the application, which then come out as [`RawEvent::Ignored`].
    ///
    /// The same policies apply whether an NBGL screen is displayed or not:
    /// - BOLOS APDUs (CLA = 0xB0) are handled internally, but while a command
    ///   is in flight only GET_VERSION is.
    /// - Any other APDU arriving while a command is in flight is a double
    ///   APDU, answered [`StatusWords::CmdNotAccepted`].
    /// - APDUs with an unexpected CLA (see [`Comm::set_expected_cla`]) are
    ///   answered [`StatusWords::BadCla`].
    /// - Malformed APDUs are answered with the matching status word, unless a
    ///   command is in flight: they are then double APDUs as well, except for
    ///   the BOLOS APDUs that are handled in flight.
    ///
    /// Answering an APDU leaves the state of the command in flight, if any,
    /// undisturbed.
    pub(crate) fn recv_filtered_event(&mut self) -> RawEvent {
        // Decoding an APDU overwrites `apdu_type` with the transport it arrived
        // on. Anything answered below is not the command in flight, so its
        // transport is restored before returning; otherwise the in-flight
        // command's response would go out on the intruder's channel.
        let in_flight_apdu_type = self.apdu_type;
        let in_progress = self.apdu_in_progress;

        let sw: Reply = match self.recv_event() {
            RawEvent::Apdu { header, .. }
                if header.cla == 0xB0
                    && (!in_progress
                        || is_bolos_apdu_allowed_in_flight(header.cla, header.ins)) =>
            {
                handle_bolos_apdu::<N>(self, header.ins, header.p1, header.p2);
                // The BOLOS reply must not be taken for the reply to the
                // command in flight.
                self.apdu_in_progress = in_progress;
                self.apdu_type = in_flight_apdu_type;
                return RawEvent::Ignored;
            }
            // Answered on the spot: while a screen is displayed, deferring to
            // the next poll loses it entirely if the screen completes in
            // between.
            RawEvent::Apdu { .. } if in_progress => StatusWords::CmdNotAccepted.into(),
            RawEvent::Apdu { header, .. }
                if self.expected_cla.is_some_and(|cla| header.cla != cla) =>
            {
                StatusWords::BadCla.into()
            }
            RawEvent::ApduError { header, .. }
                if in_progress
                    && !header.is_some_and(|h| is_bolos_apdu_allowed_in_flight(h.cla, h.ins)) =>
            {
                StatusWords::CmdNotAccepted.into()
            }
            RawEvent::ApduError { error, .. } => StatusWords::from(error).into(),
            event => return event,
        };
        self.reject_apdu(self.apdu_type, sw);
        self.apdu_type = in_flight_apdu_type;
        RawEvent::Ignored
    }

    /// Hand the APDU at `offset` in the buffer to the application.
    fn hand_over(&mut self, header: ApduHeader, offset: usize, length: usize) -> Command<'_, N> {
        // Any APDU arriving from now until the reply is a double APDU.
        self.apdu_in_progress = true;
        Command::new(self, header, offset, length)
    }

    /// Wait for the next event from the OS, and return it as seen by the
    /// application: a command, a UI event, or [`Event::Internal`].
    ///
    /// BOLOS APDUs, APDUs with an unexpected CLA (see
    /// [`Comm::set_expected_cla`]) and malformed APDUs are answered internally,
    /// and so are the APDUs that arrive before the previous command has been
    /// replied to: these are rejected as double APDUs.
    ///
    /// As it returns for every event, an application can do periodic work on
    /// [`Event::Ticker`] between commands:
    ///
    /// ```ignore
    /// loop {
    ///     match comm.next_event() {
    ///         Event::Command(command) => handle(command),
    ///         Event::Ticker => on_tick(),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub fn next_event(&mut self) -> Event<'_, N> {
        match self.recv_filtered_event() {
            RawEvent::Apdu {
                header,
                offset,
                length,
            } => Event::Command(self.hand_over(header, offset, length)),
            #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
            RawEvent::Button(button) => Event::Button(button),
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            RawEvent::Touch => Event::Touch,
            RawEvent::Ticker => Event::Ticker,
            RawEvent::ApduError { .. } | RawEvent::Ignored => Event::Internal,
        }
    }

    /// Wait for the next command for the application, skipping all other
    /// events.
    ///
    /// APDUs are filtered as in [`Comm::next_event`].
    pub fn next_command(&mut self) -> Command<'_, N> {
        // This loop works on `RawEvent` rather than on `next_event`: the
        // `Command` borrows the `Comm`, and a borrow returned from one
        // iteration would still be held by the next.
        loop {
            if let RawEvent::Apdu {
                header,
                offset,
                length,
            } = self.recv_filtered_event()
            {
                return self.hand_over(header, offset, length);
            }
        }
    }

    /// Defines `Comm::expected_cla` in order to automatically reject (with `StatusWords::BadCla`)
    /// incoming APDUs whose CLA byte differs from the given value.
    ///
    /// Usage:
    /// ```ignore
    /// let mut comm = Comm::new();
    /// comm.set_expected_cla(0xE0);
    /// ```
    pub fn set_expected_cla(&mut self, cla: u8) {
        self.expected_cla = Some(cla);
    }
}

pub enum ApduError {
    BadLen,
}

impl From<ApduError> for StatusWords {
    fn from(e: ApduError) -> Self {
        match e {
            ApduError::BadLen => StatusWords::BadLen,
        }
    }
}

pub struct Command<'a, const N: usize = DEFAULT_BUF_SIZE> {
    comm: &'a mut Comm<N>,
    header: ApduHeader,
    offset: usize,
    length: usize,
}

impl<'a, const N: usize> Command<'a, N> {
    pub(crate) fn new(
        comm: &'a mut Comm<N>,
        header: ApduHeader,
        offset: usize,
        length: usize,
    ) -> Self {
        Self {
            comm,
            header,
            offset,
            length,
        }
    }

    pub fn decode<T>(&self) -> Result<T, Reply>
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        T::try_from(self.header).map_err(Reply::from)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.comm.buf[self.offset..self.offset + self.length]
    }

    pub fn into_response(self) -> CommandResponse<'a, N> {
        CommandResponse {
            comm: self.comm,
            len: 0,
        }
    }

    pub fn into_comm(self) -> &'a mut Comm<N> {
        self.comm
    }

    pub fn reply<T: Into<Reply>>(self, data: &[u8], reply: T) -> Result<(), CommError> {
        self.into_response().extend(data)?.send(reply)?;
        Ok(())
    }
}

/// Immutable read view.
pub(crate) struct Rx<'a, const N: usize = DEFAULT_BUF_SIZE> {
    comm: &'a mut Comm<N>,
    len: usize,
}

impl<'a, const N: usize> core::ops::Deref for Rx<'a, N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, const N: usize> Rx<'a, N> {
    pub fn as_slice(&self) -> &[u8] {
        &self.comm.buf[..self.len]
    }

    /// Decode into a higher-level event. No replies are sent, but UX-related and other OS interactions are dealt with.
    pub fn decode_event(self) -> RawEvent {
        RawEvent::decode(self.comm, self.len)
    }
}

/// Mutable write view for building a send.
pub struct CommandResponse<'a, const N: usize = DEFAULT_BUF_SIZE> {
    comm: &'a mut Comm<N>,
    len: usize,
}

impl<'a, const N: usize> CommandResponse<'a, N> {
    /// Current staged length.
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn try_append(&mut self, src: &[u8]) -> Result<(), CommError> {
        let start = self.len;
        let end = start.checked_add(src.len()).ok_or(CommError::Overflow)?;
        // reserve 2 bytes for the status word
        if end > N - 2 {
            return Err(CommError::Overflow);
        }
        self.comm.buf[start..end].copy_from_slice(src);
        self.len = end;
        Ok(())
    }

    /// Append bytes, returning Self by value, in order to enable builder-style chaining.
    /// Leaves 2 bytes for the status word.
    pub fn extend(self, src: &[u8]) -> Result<Self, CommError> {
        let mut this = self;
        this.try_append(src)?;
        Ok(this)
    }

    /// Append bytes to the staged message. Returns a reference to Self.
    /// Reserves 2 bytes for the status word.
    pub fn append(&mut self, src: &[u8]) -> Result<&mut Self, CommError> {
        self.try_append(src)?;
        Ok(self)
    }

    /// Send the staged bytes, adding a status word based on the reply
    pub fn send<T: Into<Reply>>(mut self, reply: T) -> Result<&'a mut Comm<N>, CommError> {
        let sw: u16 = reply.into().0;
        self.append(sw.to_be_bytes().as_ref())?;
        let n = self.len;
        if 0 > sys_seph::io_tx(self.comm.apdu_type, self.comm.buf[..n].as_ref(), n) {
            return Err(CommError::IoError);
        }
        // Clear the pending APDU state after sending a reply, so the next
        // call to try_next_event will fetch a new event from io_rx.
        self.comm.pending_apdu = false;
        // Replying completes the current command.
        self.comm.apdu_in_progress = false;
        Ok(self.comm)
    }

    /// Clear staged bytes length.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// Initializes the `Comm` instance in static storage and registers NBGL callbacks.
///
/// This function combines the creation of a `Comm` instance, its storage in static memory,
/// and registration with NBGL in a single convenient call.
///
/// # Arguments
///
/// * `storage` - A reference to a static `CommStorage<N>` (typically created with [`define_comm!`])
///
/// # Usage
///
/// ```ignore
/// // With default buffer size (273 bytes):
/// ledger_device_sdk::define_comm!(COMM);
/// let comm = ledger_device_sdk::init_comm(&COMM);
///
/// // With custom buffer size:
/// ledger_device_sdk::define_comm!(COMM, 512);
/// let comm = ledger_device_sdk::init_comm(&COMM);
/// ```
///
/// # Returns
///
/// Returns `&'static mut Comm<N>` - a static mutable reference to the initialized `Comm` instance.
///
/// # Panics
///
/// Panics if called more than once (only one `Comm` instance can exist per application).
pub fn init_comm<const N: usize>(storage: &'static CommStorage<N>) -> &'static mut Comm<N> {
    let comm_ref = storage.init(Comm::new());
    comm_ref.nbgl_register_comm();
    callbacks::register_panic_handler::<N>();
    comm_ref
}
