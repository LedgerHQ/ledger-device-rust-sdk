use crate::seph::{self, PacketTypes};

pub use crate::io_legacy::{ApduHeader, Event, Reply, StatusWords};

use crate::io_callbacks::nbgl_register_callbacks;

#[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
use crate::seph::ItcUxEvent;

use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use crate::buttons::ButtonEvent;
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_secure_sdk_sys::buttons::{get_button_event, ButtonsState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommError {
    Overflow,
    IoError,
}

pub const DEFAULT_BUF_SIZE: usize = 273;

pub struct Comm<const N: usize = DEFAULT_BUF_SIZE> {
    buf: [u8; N],

    apdu_type: u8,
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
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
        let mut comm = Self {
            buf: [0; N],
            apdu_type: PacketTypes::PacketTypeNone as u8,
            #[cfg(not(any(target_os = "stax", target_os = "flex")))]
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
        };

        unsafe {
            CURRENT_COMM = (&mut comm as *mut Comm<N>) as *mut core::ffi::c_void;
        }
        nbgl_register_callbacks(
            next_event_ahead_impl::<N>,
            fetch_apdu_header_impl::<N>,
            reply_status_impl::<N>,
        );

        comm
    }

    /// Receive into the internal buffer. Returns a read-only guard.
    pub fn recv(&mut self, check_se_event: bool) -> Result<Rx<'_, N>, CommError> {
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
    pub fn begin_tx(&mut self) -> Tx<'_, N> {
        Tx { comm: self, len: 0 }
    }

    /// Send directly from an external slice, bypassing the internal buffer.
    pub fn send<T: Into<Reply>>(&mut self, data: &[u8], reply: T) -> Result<(), CommError> {
        self.begin_tx().extend(data)?.send(reply).unwrap();
        Ok(())
    }

    pub fn try_next_event(&mut self) -> DecodedEvent<N> {
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
                    return Command::new(self, header, offset, length);
                }
                // Explicitly convert ApduError -> StatusWords so Into<Reply> is resolved
                DecodedEventType::ApduError(e) => self.send(&[], StatusWords::from(e)).unwrap(),
                _ => {}
            }
        }
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

// TODO: we might not need to split DecodedEvent from DecodedEventType. Simplify.
pub struct DecodedEvent<const N: usize> {
    event_type: DecodedEventType,
}

impl<const N: usize> DecodedEvent<N> {
    pub fn new(comm: &mut Comm<N>, len: usize) -> Self {
        let pt = comm.buf[0];
        use crate::seph::PacketTypes;

        let event_type = match PacketTypes::from(pt) {
            PacketTypes::PacketTypeSeph | PacketTypes::PacketTypeSeEvent => {
                // Copy out SEPH payload (like original) or reinterpret in place.
                // Optimization: we can borrow slice without copying because io_buffer stores
                // [packet_type][payload...].
                Self::decode_seph_event(comm, 1)
            }
            PacketTypes::PacketTypeRawApdu
            | PacketTypes::PacketTypeUsbHidApdu
            | PacketTypes::PacketTypeUsbWebusbApdu
            | PacketTypes::PacketTypeBleApdu => Self::decode_apdu(comm, pt, 1, len),

            _ => DecodedEventType::Ignored,
        };
        Self { event_type }
    }

    pub fn into_type(self) -> DecodedEventType {
        self.event_type
    }
    pub fn from_type(event_type: DecodedEventType) -> Self {
        Self { event_type }
    }

    fn decode_seph_event(comm: &mut Comm<N>, offset: usize) -> DecodedEventType {
        use crate::seph::Events;
        let seph_buffer = &comm.buf[offset..];
        let tag = seph_buffer[0];
        match Events::from(tag) {
            // BUTTON PUSH EVENT
            #[cfg(not(any(target_os = "stax", target_os = "flex")))]
            Events::ButtonPushEvent => {
                #[cfg(feature = "nano_nbgl")]
                unsafe {
                    ux_process_button_event(seph_buffer.as_ptr() as *mut u8); // the cast to mutable can be removed on more recent SDKs
                }
                let button_info = seph_buffer[3] >> 1;
                if let Some(btn_evt) = get_button_event(&mut comm.buttons, button_info) {
                    return DecodedEventType::Button(btn_evt);
                }
                DecodedEventType::Ignored
            }

            // SCREEN TOUCH EVENT
            #[cfg(any(target_os = "stax", target_os = "flex"))]
            Events::ScreenTouchEvent => unsafe {
                ux_process_finger_event(seph_buffer.as_ptr() as *mut u8); // the cast to mutable can be removed on more recent SDKs
                return DecodedEventType::Touch;
            },

            // TICKER EVENT
            Events::TickerEvent => {
                #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
                unsafe {
                    ux_process_ticker_event();
                }
                DecodedEventType::Ticker
            }

            // ITC EVENT
            seph::Events::ItcEvent => {
                let _len = u16::from_be_bytes([seph_buffer[1], seph_buffer[2]]) as usize;
                #[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
                match ItcUxEvent::from(seph_buffer[3]) {
                    seph::ItcUxEvent::AskBlePairing => unsafe {
                        G_ux_params.ux_id = BOLOS_UX_ASYNCHMODAL_PAIRING_REQUEST;
                        G_ux_params.len = 20;
                        G_ux_params.u.pairing_request.type_ = seph_buffer[4];
                        G_ux_params.u.pairing_request.pairing_info_len = (_len - 2) as u32;
                        for i in 0..G_ux_params.u.pairing_request.pairing_info_len as usize {
                            G_ux_params.u.pairing_request.pairing_info[i as usize] =
                                seph_buffer[5 + i] as i8;
                        }
                        G_ux_params.u.pairing_request.pairing_info
                            [G_ux_params.u.pairing_request.pairing_info_len as usize] = 0;
                        os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);
                    },

                    seph::ItcUxEvent::BlePairingStatus => unsafe {
                        G_ux_params.ux_id = BOLOS_UX_ASYNCHMODAL_PAIRING_STATUS;
                        G_ux_params.len = 0;
                        G_ux_params.u.pairing_status.pairing_ok = seph_buffer[4];
                        os_ux(&raw mut G_ux_params as *mut bolos_ux_params_t);
                    },

                    seph::ItcUxEvent::Redisplay => {
                        #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
                        unsafe {
                            nbgl_objAllowDrawing(true);
                            nbgl_screenRedraw();
                            nbgl_refresh();
                        }
                    }
                    _ => {}
                }
                DecodedEventType::Ignored
            }
            // DEFAULT EVENT
            _ => {
                #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
                unsafe {
                    ux_process_default_event();
                }
                #[cfg(any(target_os = "nanox", target_os = "nanosplus"))]
                if !cfg!(feature = "nano_nbgl") {
                    crate::uxapp::UxEvent::Event.request();
                }
                DecodedEventType::Ignored
            }
        }
    }

    fn decode_apdu(
        comm: &mut Comm<N>,
        packet_type: u8,
        offset: usize,
        io_len: usize,
    ) -> DecodedEventType {
        use ApduError::*;

        comm.apdu_type = packet_type;

        let apdu_buffer = &comm.buf[offset..];

        if io_len < 5 {
            return DecodedEventType::ApduError(BadLen);
        }

        let rx_len = io_len - 1;

        let header = ApduHeader {
            cla: apdu_buffer[0],
            ins: apdu_buffer[1],
            p1: apdu_buffer[2],
            p2: apdu_buffer[3],
        };
        if rx_len == 4 {
            return DecodedEventType::new_apdu(header, 4, 0);
        }
        let first_len_byte = apdu_buffer[4];

        match (first_len_byte, rx_len) {
            (0, 5) => {
                // Non-conforming zero-data APDU (TODO: per the standard, this should actually be read as a 256-byte long APDU; but that's likely to break things as lots)
                DecodedEventType::new_apdu(header, 4, 0)
            }
            (0, 6) => DecodedEventType::ApduError(BadLen),
            (0, _) => {
                let len = u16::from_be_bytes([apdu_buffer[5], apdu_buffer[6]]) as usize;
                if rx_len != len + 7 {
                    return DecodedEventType::ApduError(BadLen);
                }
                DecodedEventType::new_apdu(header, 1 + 7, len)
            }
            (len, _) => {
                if rx_len != len as usize + 5 {
                    return DecodedEventType::ApduError(BadLen);
                }
                DecodedEventType::new_apdu(header, 1 + 5, len as usize)
            }
        }
    }
}

/// High-level decoded event (no side effects).
pub enum DecodedEventType {
    Apdu {
        header: ApduHeader,
        offset: usize,
        length: usize,
    },
    ApduError(ApduError),
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    Button(ButtonEvent),
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    Touch,
    Ticker,
    // Events for which no additional handling is required after decoding it
    Ignored,
}

impl DecodedEventType {
    fn new_apdu(header: ApduHeader, offset: usize, length: usize) -> Self {
        Self::Apdu {
            header,
            offset,
            length,
        }
    }
}

pub struct Command<'a, const N: usize> {
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

    pub fn into_tx(self) -> Tx<'a, N> {
        Tx {
            comm: self.comm,
            len: 0,
        }
    }

    pub fn into_comm(self) -> &'a mut Comm<N> {
        self.comm
    }

    pub fn reply<T: Into<Reply>>(self, data: &[u8], reply: T) -> Result<(), CommError> {
        self.into_tx().extend(data)?.send(reply)?;
        Ok(())
    }
}

/// Immutable read view.
pub struct Rx<'a, const N: usize> {
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
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn consume(self) -> () {}

    /// Decode into a higher-level event. No replies are sent, but UX-related and other OS interactions are dealt with.
    pub fn decode_event(self) -> DecodedEvent<N> {
        DecodedEvent::new(self.comm, self.len)
    }
}

/// Mutable write view for building a send.
pub struct Tx<'a, const N: usize> {
    comm: &'a mut Comm<N>,
    len: usize,
}

impl<'a, const N: usize> Tx<'a, N> {
    pub fn new(comm: &'a mut Comm<N>) -> Self {
        Self { comm, len: 0 }
    }

    /// Current staged length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Set staged length explicitly after writing into `buf_mut`.
    pub fn set_len(&mut self, n: usize) -> Result<(), CommError> {
        // reserve 2 bytes for the status word
        if n <= N - 2 {
            self.len = n;
            Ok(())
        } else {
            Err(CommError::Overflow)
        }
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
        Ok(self.comm)
    }

    /// Clear staged bytes length.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

// ===== NBGL callback integration =====

// Erased pointer to the Comm instance
static mut CURRENT_COMM: *mut core::ffi::c_void = core::ptr::null_mut();

// Converts the pointer back to the concrete Comm<N> type.
unsafe fn get_comm<const N: usize>() -> &'static mut Comm<N> {
    &mut *(CURRENT_COMM as *mut Comm<N>)
}

// Implementation wrappers specialized per const N.

fn next_event_ahead_impl<const N: usize>() -> bool {
    let comm = unsafe { get_comm::<N>() };
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

fn fetch_apdu_header_impl<const N: usize>() -> Option<ApduHeader> {
    let comm = unsafe { get_comm::<N>() };
    if comm.pending_apdu {
        Some(comm.pending_header)
    } else {
        None
    }
}

fn reply_status_impl<const N: usize>(reply: Reply) {
    let comm = unsafe { get_comm::<N>() };
    if comm.pending_apdu {
        comm.pending_apdu = false;
    }
    let _ = comm.begin_tx().send(reply);
}

// BOLOS APDU Handling
fn handle_bolos_apdu<const N: usize>(comm: &mut Comm<N>, ins: u8) {
    match ins {
        // Get Information INS: retrieve App name and version
        0x01 => {
            let mut tx = comm.begin_tx();
            let _ = tx.append(&[0x01]);
            const MAX_TAG_LENGTH: u8 = 32; // maximum length for the buffer containing app name/version.
            let mut tag_buf = [0u8; MAX_TAG_LENGTH as usize];

            // ---- App name ----

            let name_len = unsafe {
                os_registry_get_current_app_tag(
                    BOLOS_TAG_APPNAME,
                    tag_buf.as_mut_ptr(),
                    MAX_TAG_LENGTH as u32,
                )
            };

            if name_len > MAX_TAG_LENGTH.into() {
                let _ = tx.send(StatusWords::Panic); // this should never happen
                return;
            }

            let _ = tx.append(&[name_len as u8]);
            let _ = tx.append(&tag_buf[..name_len as usize]);

            // ---- App version ----

            let ver_len = unsafe {
                os_registry_get_current_app_tag(
                    BOLOS_TAG_APPVERSION,
                    tag_buf.as_mut_ptr(),
                    MAX_TAG_LENGTH as u32,
                )
            };

            if ver_len > MAX_TAG_LENGTH.into() {
                let _ = tx.send(StatusWords::Panic); // this should never happen
                return;
            }

            let _ = tx.append(&[ver_len as u8]);
            let _ = tx.append(&tag_buf[..ver_len as usize]);

            // ---- Flags ----
            let flags_byte = unsafe { os_flags() } as u8;
            // flags length (always 1 currently) then flags byte
            let _ = tx.append(&[1]);
            let _ = tx.append(&[flags_byte]);
            let _ = tx.send(StatusWords::Ok);
        }
        // Quit Application INS
        0xa7 => {
            let _ = comm.begin_tx().send(StatusWords::Ok);
            crate::exit_app(0);
        }
        // Unknown INS within BOLOS namespace
        _ => {
            let _ = comm.begin_tx().send(StatusWords::BadIns);
        }
    }
}
