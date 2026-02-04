use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;

use crate::seph::PacketTypes;

mod event;
pub use event::{DecodedEvent, DecodedEventType};

mod bolos;
pub(crate) mod callbacks;
use bolos::handle_bolos_apdu;

pub use crate::io_legacy::{ApduHeader, Event, Reply, StatusWords};

use crate::io_callbacks::nbgl_register_callbacks;

use ledger_secure_sdk_sys::seph as sys_seph;

/// Default buffer size for `Comm` when no custom size is specified.
pub const DEFAULT_BUF_SIZE: usize = 273;

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
    initialized: Cell<bool>,
}

// SAFETY: single-threaded runtime, and initialization protected by an atomic flag.
unsafe impl<const N: usize> Sync for CommStorage<N> {}

impl<const N: usize> CommStorage<N> {
    /// Creates a new uninitialized `CommStorage`.
    ///
    /// This is a const fn, suitable for use in static declarations.
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            initialized: Cell::new(false),
        }
    }

    /// Initializes the storage with a `Comm<N>` instance and returns a static reference.
    ///
    /// # Panics
    ///
    /// Panics if called more than once (the storage can only be initialized once).
    ///
    /// # Safety
    ///
    /// This method must be called on a static `CommStorage` instance to ensure
    /// the returned reference has a `'static` lifetime.
    pub fn init(&'static self, comm: Comm<N>) -> &'static mut Comm<N> {
        // Panic if already initialized
        if self.initialized.get() {
            panic!("CommStorage already initialized. Only one Comm instance can exist.");
        }
        self.initialized.set(true);

        // SAFETY: We just verified this is the first and only initialization.
        // The storage is static, so the reference is valid for 'static.
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
/// initialized using [`init_comm!`] before use.
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
///     let comm = init_comm!(COMM);
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
use crate::buttons::ButtonEvent;
#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use ledger_secure_sdk_sys::buttons::{get_button_event, ButtonsState};

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
}

impl<const N: usize> Comm<N> {
    pub fn new() -> Self {
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
        }
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

    pub fn try_next_event(&mut self) -> DecodedEvent<N> {
        // If there's a pending APDU from a callback (e.g., nbgl_next_event_ahead),
        // return it instead of calling recv() which would return 0.
        if self.pending_apdu {
            self.pending_apdu = false;
            return DecodedEvent::from_type(DecodedEventType::Apdu {
                header: self.pending_header,
                offset: self.pending_offset,
                length: self.pending_length,
            });
        }
        self.recv(true).unwrap().decode_event()
    }

    pub fn next_event(&mut self) -> DecodedEvent<N> {
        // Iteratively get and decode events until one is not Ignored
        // This was a bit tricky as the borrow checker doesn't like the straightforward implementation.
        // The helpers into_type() and from_type() help to avoid lifetime issues.
        loop {
            let ety = self.try_next_event().into_type();

            if !matches!(ety, DecodedEventType::Ignored) {
                // Re-borrow here to build the return value.
                return DecodedEvent::from_type(ety);
            }
        }
    }

    pub fn next_command(&mut self) -> Command<'_, N> {
        loop {
            let ety = self.next_event().into_type();
            match ety {
                DecodedEventType::Apdu {
                    header,
                    offset,
                    length,
                } => {
                    // Handle BOLOS internal APDUs (CLA = 0xB0, P1 = 0x00, P2 = 0x00) internally
                    // and continue looping until an application APDU arrives.
                    if header.cla == 0xB0 && header.p1 == 0x00 && header.p2 == 0x00 {
                        handle_bolos_apdu::<N>(self, header.ins);
                        continue;
                    }
                    // If CLA filtering is enabled, automatically reject APDUs with wrong CLA.
                    if let Some(cla) = self.expected_cla {
                        if header.cla != cla {
                            let _ = self.begin_response().send(StatusWords::BadCla);
                            continue;
                        }
                    }
                    return Command::new(self, header, offset, length);
                }
                // Explicitly convert ApduError -> StatusWords so Into<Reply> is resolved
                DecodedEventType::ApduError(e) => self.send(&[], StatusWords::from(e)).unwrap(),
                _ => {}
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
    pub fn new(comm: &'a mut Comm<N>, header: ApduHeader, offset: usize, length: usize) -> Self {
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
    pub fn decode_event(self) -> DecodedEvent<N> {
        DecodedEvent::new(self.comm, self.len)
    }
}

/// Mutable write view for building a send.
pub struct CommandResponse<'a, const N: usize = DEFAULT_BUF_SIZE> {
    comm: &'a mut Comm<N>,
    len: usize,
}

impl<'a, const N: usize> CommandResponse<'a, N> {
    pub fn new(comm: &'a mut Comm<N>) -> Self {
        Self { comm, len: 0 }
    }

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
        Ok(self.comm)
    }

    /// Clear staged bytes length.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl<const N: usize> Drop for Comm<N> {
    fn drop(&mut self) {
        callbacks::clear_comm();
        callbacks::clear_panic_handler();
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
