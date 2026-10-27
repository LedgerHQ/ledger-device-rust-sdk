// We suppress dead code warnings, as many of the members defined in this module are not used
// with io_new. This can be removed once the migration to io_new is completed.
#![cfg_attr(feature = "io_new", allow(dead_code))]

#[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
use ledger_secure_sdk_sys::buttons::{ButtonEvent, ButtonsState, get_button_event};
use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

use crate::io_callbacks::nbgl_register_callbacks;
use crate::seph;

#[cfg(any(
    target_os = "nanox",
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p"
))]
use crate::seph::ItcUxEvent;

use core::convert::{Infallible, TryFrom};
use core::ops::{Index, IndexMut};

#[cfg(any(
    target_os = "nanox",
    target_os = "stax",
    target_os = "flex",
    target_os = "apex_p"
))]
unsafe extern "C" {
    pub unsafe static mut G_ux_params: bolos_ux_params_t;
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum StatusWords {
    Ok = 0x9000,
    NothingReceived = 0x6982,
    BadCla = 0x6e00,
    BadIns = 0x6e01,
    BadP1P2 = 0x6e02,
    BadLen = 0x6e03,
    UserCancelled = 0x6985,
    Unknown = 0x6d00,
    Panic = 0xe000,
    DeviceLocked = 0x5515,
    CmdNotAccepted = 0x6901,
}

#[derive(Debug)]
#[repr(u8)]
pub enum SyscallError {
    InvalidParameter = 2,
    Overflow,
    Security,
    InvalidCrc,
    InvalidChecksum,
    InvalidCounter,
    NotSupported,
    InvalidState,
    Timeout,
    InvalidPkiCertificate,
    Unspecified,
}

impl From<u32> for SyscallError {
    fn from(e: u32) -> SyscallError {
        match e {
            2 => SyscallError::InvalidParameter,
            3 => SyscallError::Overflow,
            4 => SyscallError::Security,
            5 => SyscallError::InvalidCrc,
            6 => SyscallError::InvalidChecksum,
            7 => SyscallError::InvalidCounter,
            8 => SyscallError::NotSupported,
            9 => SyscallError::InvalidState,
            10 => SyscallError::Timeout,
            11 => SyscallError::InvalidPkiCertificate,
            _ => SyscallError::Unspecified,
        }
    }
}

#[repr(u32)]
pub enum PkiLoadCertificateError {
    InvalidStructureType,
    IncorrectCertificateVersion,
    IncorrectCertificateValidity,
    IncorrectCertificateValidityIndex,
    UnknownSignerKeyId,
    UnknownSignatureAlgorithm,
    UnknownPublicKeyId,
    UnknownPublicKeyUsage,
    IncorrectEllipticCurveId,
    IncorrectSignatureAlgorithmAssociatedToPublicKey,
    UnknownTargetDevice,
    UnknownCertificateTag,
    FailedToHashData,
    ExpectedKeyUsageDoesNotMatchCertificateKeyUsage,
    FailedToVerifySignature,
    TrustedNameBufferTooSmall,
}

impl From<u32> for PkiLoadCertificateError {
    fn from(e: u32) -> PkiLoadCertificateError {
        match e {
            0x422F => PkiLoadCertificateError::InvalidStructureType,
            0x4230 => PkiLoadCertificateError::IncorrectCertificateVersion,
            0x4231 => PkiLoadCertificateError::IncorrectCertificateValidity,
            0x4232 => PkiLoadCertificateError::IncorrectCertificateValidityIndex,
            0x4233 => PkiLoadCertificateError::UnknownSignerKeyId,
            0x4234 => PkiLoadCertificateError::UnknownSignatureAlgorithm,
            0x4235 => PkiLoadCertificateError::UnknownPublicKeyId,
            0x4236 => PkiLoadCertificateError::UnknownPublicKeyUsage,
            0x4237 => PkiLoadCertificateError::IncorrectEllipticCurveId,
            0x4238 => PkiLoadCertificateError::IncorrectSignatureAlgorithmAssociatedToPublicKey,
            0x4239 => PkiLoadCertificateError::UnknownTargetDevice,
            0x422D => PkiLoadCertificateError::UnknownCertificateTag,
            0x3301 => PkiLoadCertificateError::FailedToHashData,
            0x422E => PkiLoadCertificateError::ExpectedKeyUsageDoesNotMatchCertificateKeyUsage,
            0x5720 => PkiLoadCertificateError::FailedToVerifySignature,
            0x4118 => PkiLoadCertificateError::TrustedNameBufferTooSmall,
            _ => panic!("Unknown PKI Load Certificate Error"),
        }
    }
}

impl From<PkiLoadCertificateError> for SyscallError {
    fn from(_e: PkiLoadCertificateError) -> SyscallError {
        SyscallError::InvalidPkiCertificate
    }
}

/// Provide a type that will be used for replying
/// an APDU with either a StatusWord or an SyscallError
#[derive(Debug)]
#[repr(transparent)]
pub struct Reply(pub u16);

impl From<StatusWords> for Reply {
    fn from(sw: StatusWords) -> Reply {
        Reply(sw as u16)
    }
}

impl From<SyscallError> for Reply {
    fn from(exc: SyscallError) -> Reply {
        Reply(0x6800 + exc as u16)
    }
}

// Needed because some methods use `TryFrom<ApduHeader>::Error`, and for `ApduHeader` we have
// `Error` as `Infallible`. Since we need to convert such error in a status word (`Reply`) we need
// to implement this trait here.
impl From<Infallible> for Reply {
    fn from(_value: Infallible) -> Self {
        Reply(0x9000)
    }
}

/// Possible events returned by [`Comm::next_event`]
#[derive(Eq, PartialEq)]
pub enum Event<T> {
    /// APDU event
    Command(T),
    /// Button press or release event
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    Button(ButtonEvent),
    #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
    TouchEvent,
    /// Ticker
    Ticker,
}

/// Set once a [`CommStorage`] has been initialized.
// SAFETY: the runtime is single-threaded, so direct reads/writes are safe.
#[cfg(not(feature = "io_new"))]
static mut COMM_INITIALIZED: bool = false;

/// Static storage for the application's [`Comm`], declared with
/// [`define_comm!`](crate::define_comm) and initialized by
/// [`nbgl::init_static_comm`](crate::nbgl::init_static_comm).
#[cfg(not(feature = "io_new"))]
pub struct CommStorage {
    inner: core::cell::UnsafeCell<core::mem::MaybeUninit<Comm>>,
}

// SAFETY: single-threaded runtime; `inner` is only written once, guarded by
// COMM_INITIALIZED.
#[cfg(not(feature = "io_new"))]
unsafe impl Sync for CommStorage {}

#[cfg(not(feature = "io_new"))]
impl CommStorage {
    /// Creates a new uninitialized `CommStorage`, for use in a static
    /// declaration.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            inner: core::cell::UnsafeCell::new(core::mem::MaybeUninit::uninit()),
        }
    }

    /// Stores `comm` and returns a static reference to it.
    ///
    /// # Panics
    ///
    /// Panics if a `CommStorage` has already been initialized: only one `Comm`
    /// instance can exist.
    // A single mutable reference is ever handed out, guarded by COMM_INITIALIZED.
    #[allow(clippy::mut_from_ref)]
    pub fn init(&'static self, comm: Comm) -> &'static mut Comm {
        // SAFETY: single-threaded runtime; no concurrent access is possible.
        if unsafe { COMM_INITIALIZED } {
            panic!("CommStorage already initialized. Only one Comm instance can exist.");
        }
        unsafe { COMM_INITIALIZED = true };

        // SAFETY: this point is reached only once, and the storage is static.
        unsafe {
            let ptr = self.inner.get();
            (*ptr).write(comm);
            (*ptr).assume_init_mut()
        }
    }
}

/// Declares a static [`CommStorage`] with the given name, to be initialized
/// with [`nbgl::init_static_comm`](crate::nbgl::init_static_comm).
///
/// ```ignore
/// ledger_device_sdk::define_comm!(COMM);
///
/// fn main() {
///     let comm = ledger_device_sdk::nbgl::init_static_comm(&COMM);
/// }
/// ```
#[cfg(not(feature = "io_new"))]
#[macro_export]
macro_rules! define_comm {
    ($name:ident) => {
        static $name: $crate::io::CommStorage = $crate::io::CommStorage::new();
    };
}

/// Manages the communication of the device: receives events such as button presses, incoming
/// APDU requests, and provides methods to build and transmit APDU responses.
pub struct Comm {
    pub apdu_buffer: [u8; 272],
    pub rx: usize,
    pub tx: usize,
    pub event_pending: bool,
    #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
    buttons: ButtonsState,
    /// Expected value for the APDU CLA byte.
    /// If defined, [`Comm`] will automatically reply with [`StatusWords::BadCla`] when an APDU
    /// with wrong CLA byte is received. If set to [`None`], all CLA are accepted.
    /// Can be set using [`Comm::set_expected_cla`] method.
    pub expected_cla: Option<u8>,

    pub apdu_type: u8,
    pub io_buffer: [u8; 273],
    pub rx_length: usize,
    pub tx_length: usize,
    /// When true, apdu_send skips the io_rx(false) call.
    /// Used when replying to BOLOS APDUs where io_rx(false) would deadlock.
    #[allow(dead_code)]
    skip_rx_on_send: bool,
    /// True from the moment a command is handed to the application until the
    /// application replies to it. This is what makes a second incoming APDU a
    /// *double* APDU rather than the normal way of receiving work.
    apdu_in_progress: bool,
}

impl Default for Comm {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ApduHeader {
    /// Class
    pub cla: u8,
    /// Instruction
    pub ins: u8,
    /// Parameter 1
    pub p1: u8,
    /// Parameter 2
    pub p2: u8,
}

impl Comm {
    /// Creates a new [`Comm`] instance, which accepts any CLA APDU by default.
    pub fn new() -> Self {
        Self {
            apdu_buffer: [0u8; 272],
            rx: 0,
            tx: 0,
            event_pending: false,
            #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
            buttons: ButtonsState::new(),
            expected_cla: None,
            apdu_type: seph::PacketTypes::PacketTypeNone as u8,
            io_buffer: [0u8; 273],
            rx_length: 0,
            tx_length: 0,
            skip_rx_on_send: false,
            apdu_in_progress: false,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn nbgl_register_comm(&mut self) {
        // Register NBGL callbacks if not already set and record current Comm singleton.
        unsafe {
            CURRENT_COMM = self as *mut Comm;
        }
        nbgl_register_callbacks(
            default_nbgl_next_event_ahead,
            default_nbgl_fetch_apdu_header,
            default_nbgl_reply_status,
        );
    }

    /// Defines [`Comm::expected_cla`] in order to reply automatically [`StatusWords::BadCla`] when
    /// an incoming APDU has a CLA byte different from the given value.
    ///
    /// # Arguments
    ///
    /// * `cla` - Expected value for APDUs CLA byte.
    ///
    /// # Examples
    ///
    /// This method can be used when building an instance of [`Comm`]:
    ///
    /// ```
    /// let mut comm = Comm::new().set_expected_cla(0xe0);
    /// ```
    pub fn set_expected_cla(mut self, cla: u8) -> Self {
        self.expected_cla = Some(cla);
        self
    }

    /// Send the currently held APDU
    // This is private. Users should call reply to set the status word and
    // transmit the response.
    fn apdu_send(&mut self) {
        #[cfg(any(
            target_os = "stax",
            target_os = "flex",
            target_os = "apex_p",
            feature = "nano_nbgl"
        ))]
        if !self.skip_rx_on_send {
            let mut buffer: [u8; 273] = [0; 273];
            let status = sys_seph::io_rx(&mut buffer, false);
            if status > 0 {
                let packet_type = seph::PacketTypes::from(buffer[0]);
                let event = seph::Events::from(buffer[1]);
                if let (seph::PacketTypes::PacketTypeSeph, seph::Events::TickerEvent) =
                    (packet_type, event)
                {
                    unsafe {
                        ux_process_ticker_event();
                    }
                }
            }
        }
        self.skip_rx_on_send = false;
        if self.tx != 0 {
            sys_seph::io_tx(self.apdu_type, &self.apdu_buffer, self.tx);
            self.tx = 0;
        } else {
            sys_seph::io_tx(self.apdu_type, &self.io_buffer, self.tx_length);
        }
        self.tx_length = 0;
        self.rx_length = 0;
        // Replying completes the current command.
        self.apdu_in_progress = false;
    }

    /// Wait and return next button press event or APDU command.
    ///
    /// `T` can be any type built from a [`ApduHeader`] using the [`TryFrom<ApduHeader>`] trait.
    /// The conversion can embed complex parsing logic, including checks on CLA, INS, P1 and P2
    /// bytes, and may return an error with a status word for invalid APDUs.
    ///
    /// In particular, it is recommended to use an enumeration for the possible INS values.
    ///
    /// # Examples
    ///
    /// ```
    /// enum Instruction {
    ///     Select,
    ///     ReadBinary
    /// }
    ///
    /// impl TryFrom<ApduHeader> for Instruction {
    ///     type Error = StatusWords;
    ///
    ///     fn try_from(h: ApduHeader) -> Result<Self, Self::Error> {
    ///         match h.ins {
    ///             0xa4 => Ok(Self::Select),
    ///             0xb0 => Ok(Self::ReadBinary)
    ///             _ => Err(StatusWords::BadIns)
    ///         }
    ///     }
    /// }
    ///
    /// loop {
    ///     match comm.next_event() {
    ///         Event::Button(button) => { ... }
    ///         Event::Command(Instruction::Select) => { ... }
    ///         Event::Command(Instruction::ReadBinary) => { ... }
    ///     }
    /// }
    /// ```
    pub fn next_event<T>(&mut self) -> Event<T>
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        self.rx_length = 0;
        loop {
            let status = sys_seph::io_rx(&mut self.io_buffer, true);

            if status > 0
                && let Some(value) = self.decode_event(status)
            {
                return value;
            }
        }
    }

    /// Fetch and process one event while an NBGL screen is displayed, and report
    /// whether it was an APDU command.
    ///
    /// This is called from `ux_sync_wait` in two different situations:
    ///
    /// * no command is being processed (idle home screen): an incoming APDU is
    ///   the normal way of receiving work, so it is decoded and `true` is
    ///   returned so the caller can leave the screen and handle it.
    /// * a command is being processed (review, status, … screen): an incoming
    ///   APDU is a double APDU and is answered [`StatusWords::CmdNotAccepted`].
    pub fn next_event_ahead<T>(&mut self) -> bool
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let status = sys_seph::io_rx(&mut self.io_buffer, true);
        if status <= 0 {
            return false;
        }

        // An APDU received while a command is in flight is handled directly on
        // the received frame, whatever its CLA or length.
        if self.apdu_in_progress && Self::is_apdu_packet(self.io_buffer[0]) {
            self.handle_double_apdu(status as usize - 1);
            return false;
        }

        self.detect_apdu::<T>(status)
    }

    /// Answer an APDU of `rx` bytes, still in `io_buffer`, received while
    /// another command is in flight.
    ///
    /// Only BOLOS [`BOLOS_INS_GET_VERSION`] is answered inline, as `io_new`
    /// does, so that hosts can still identify the app while a screen is
    /// displayed. Everything else is answered [`StatusWords::CmdNotAccepted`].
    /// The reply goes out on the transport the APDU was received on, and the
    /// state of the command in flight is kept as is.
    fn handle_double_apdu(&mut self, rx: usize) {
        let received_apdu_type = self.io_buffer[0];

        if Self::is_device_locked() {
            self.reject_apdu(StatusWords::DeviceLocked);
            return;
        }

        if rx < 4 || !is_bolos_apdu_allowed_in_flight(self.io_buffer[1], self.io_buffer[2]) {
            self.reject_apdu(StatusWords::CmdNotAccepted);
            return;
        }

        // `get_data` cannot validate the length here as it works on the
        // in-flight command's `apdu_buffer`. BOLOS APDUs only use short Lc.
        let lc = self.io_buffer[5] as usize;
        let length_ok = rx == 4 || (rx == 5 && lc == 0) || (lc != 0 && rx == 5 + lc);
        if !length_ok {
            self.reject_apdu(StatusWords::BadLen);
            return;
        }

        // `handle_bolos_apdu` reads its request from `io_buffer` and replies
        // through `apdu_send`, which uses the response state: reply on the
        // received APDU's transport, then restore the command's state.
        let saved = (self.apdu_type, self.tx, self.tx_length, self.rx_length);
        self.apdu_type = received_apdu_type;
        self.tx = 0;
        self.tx_length = 0;
        self.skip_rx_on_send = true;
        handle_bolos_apdu(
            self,
            self.io_buffer[2],
            self.io_buffer[3],
            self.io_buffer[4],
        );
        (self.apdu_type, self.tx, self.tx_length, self.rx_length) = saved;
        self.skip_rx_on_send = false;
        self.apdu_in_progress = true;
    }

    /// True if a PIN is set and has not been validated yet.
    fn is_device_locked() -> bool {
        unsafe {
            os_perso_is_pin_set() == BOLOS_TRUE.try_into().unwrap()
                && os_global_pin_is_validated() != BOLOS_TRUE.try_into().unwrap()
        }
    }

    pub fn check_event<T>(&mut self) -> Option<Event<T>>
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        if self.event_pending {
            //let mut apdu_buffer = [0u8; 272];
            //apdu_buffer[0..272].copy_from_slice(&self.io_buffer[1..273]);
            self.event_pending = false;

            // Reject incomplete APDUs
            if self.rx_length < 5 {
                self.reply(StatusWords::BadLen);
                return None;
            }

            // Check for data length by using `get_data`
            if let Err(sw) = self.get_data() {
                self.reply(sw);
                return None;
            }

            // Manage BOLOS specific APDUs B0xxyyzz
            if self.io_buffer[1] == 0xB0 {
                self.skip_rx_on_send = true;
                handle_bolos_apdu(
                    self,
                    self.io_buffer[2],
                    self.io_buffer[3],
                    self.io_buffer[4],
                );
                return None;
            }

            // If CLA filtering is enabled, automatically reject APDUs with wrong CLA
            if let Some(cla) = self.expected_cla
                && self.io_buffer[1] != cla
            {
                self.reply(StatusWords::BadCla);
                return None;
            }

            let res = T::try_from(*self.get_apdu_metadata());
            match res {
                Ok(ins) => {
                    // The command is about to be handed to the application: any
                    // APDU arriving from now until the reply is a double APDU.
                    self.apdu_in_progress = true;
                    return Some(Event::Command(ins));
                }
                Err(sw) => {
                    // Invalid Ins code. Send automatically an error, mask
                    // the bad instruction to the application and just
                    // discard this event.
                    self.reply(sw);
                }
            }
        }
        None
    }

    #[allow(unused_mut)]
    pub fn process_event<T>(&mut self, mut seph_buffer: [u8; 272], length: i32) -> Option<Event<T>>
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let tag = seph_buffer[0];
        let _len: usize = u16::from_be_bytes([seph_buffer[1], seph_buffer[2]]) as usize;

        if (length as usize) < _len + 3 {
            self.reply(StatusWords::BadLen);
            return None;
        }

        match seph::Events::from(tag) {
            // BUTTON PUSH EVENT
            #[cfg(any(target_os = "nanosplus", target_os = "nanox"))]
            seph::Events::ButtonPushEvent => {
                #[cfg(feature = "nano_nbgl")]
                unsafe {
                    ux_process_button_event(seph_buffer.as_mut_ptr());
                }
                let button_info = seph_buffer[3] >> 1;
                if let Some(btn_evt) = get_button_event(&mut self.buttons, button_info) {
                    return Some(Event::Button(btn_evt));
                }
            }

            // SCREEN TOUCH EVENT
            #[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
            seph::Events::ScreenTouchEvent => unsafe {
                ux_process_finger_event(seph_buffer.as_mut_ptr());
                return Some(Event::TouchEvent);
            },

            // TICKER EVENT
            seph::Events::TickerEvent => {
                #[cfg(any(
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p",
                    feature = "nano_nbgl"
                ))]
                unsafe {
                    ux_process_ticker_event();
                }
                return Some(Event::Ticker);
            }

            // ITC EVENT
            seph::Events::ItcEvent => {
                #[cfg(any(
                    target_os = "nanox",
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p"
                ))]
                match ItcUxEvent::from(seph_buffer[3]) {
                    seph::ItcUxEvent::AskBlePairing => unsafe {
                        G_ux_params.ux_id = BOLOS_UX_ASYNCHMODAL_PAIRING_REQUEST;
                        G_ux_params.len = 20;
                        G_ux_params.u.pairing_request.type_ = seph_buffer[4];
                        G_ux_params.u.pairing_request.pairing_info_len = (_len - 2) as u32;
                        for i in 0..G_ux_params.u.pairing_request.pairing_info_len as usize {
                            G_ux_params.u.pairing_request.pairing_info[i] =
                                seph_buffer[5 + i] as core::ffi::c_char;
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
                        #[cfg(any(
                            target_os = "stax",
                            target_os = "flex",
                            target_os = "apex_p",
                            feature = "nano_nbgl"
                        ))]
                        unsafe {
                            nbgl_objAllowDrawing(true);
                            nbgl_screenRedraw();
                            nbgl_refresh();
                        }
                    }

                    _ => return None,
                }
                return None;
            }

            // DEFAULT EVENT
            _ => {
                #[cfg(any(
                    target_os = "stax",
                    target_os = "flex",
                    target_os = "apex_p",
                    feature = "nano_nbgl"
                ))]
                unsafe {
                    ux_process_default_event();
                }
                #[cfg(any(target_os = "nanox", target_os = "nanosplus"))]
                if !cfg!(feature = "nano_nbgl") {
                    crate::uxapp::UxEvent::Event.request();
                }
            }
        }
        None
    }

    pub fn decode_event<T>(&mut self, length: i32) -> Option<Event<T>>
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let packet_type = self.io_buffer[0];

        match seph::PacketTypes::from(packet_type) {
            seph::PacketTypes::PacketTypeSeph | seph::PacketTypes::PacketTypeSeEvent => {
                // SE or SEPH event
                let mut seph_buffer = [0u8; 272];
                seph_buffer[0..272].copy_from_slice(&self.io_buffer[1..273]);
                if let Some(event) = self.process_event(seph_buffer, length - 1) {
                    return Some(event);
                }
            }

            seph::PacketTypes::PacketTypeRawApdu
            | seph::PacketTypes::PacketTypeUsbHidApdu
            | seph::PacketTypes::PacketTypeUsbWebusbApdu
            | seph::PacketTypes::PacketTypeBleApdu => {
                if Self::is_device_locked() {
                    self.reply(StatusWords::DeviceLocked);
                    return None;
                }
                self.apdu_buffer[0..272].copy_from_slice(&self.io_buffer[1..273]);
                self.apdu_type = packet_type;
                self.rx_length = length as usize;
                self.rx = self.rx_length - 1;
                self.event_pending = true;
                return self.check_event();
            }

            _ => {}
        }
        None
    }

    fn detect_apdu<T>(&mut self, length: i32) -> bool
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        match self.decode_event::<T>(length) {
            Some(Event::Command(_)) => {
                self.rx_length = length as usize;
                self.rx = self.rx_length - 1;
                self.event_pending = true;
                true
            }
            _ => false,
        }
    }

    /// True if a received SEPH frame carries an APDU rather than an event.
    fn is_apdu_packet(packet_type: u8) -> bool {
        matches!(
            seph::PacketTypes::from(packet_type),
            seph::PacketTypes::PacketTypeRawApdu
                | seph::PacketTypes::PacketTypeUsbHidApdu
                | seph::PacketTypes::PacketTypeUsbWebusbApdu
                | seph::PacketTypes::PacketTypeBleApdu
        )
    }

    /// Answer `sw` to an APDU received while another command is in flight.
    ///
    /// The status word is sent on the intruder's own transport, taken from the
    /// received frame, and nothing owned by the in-flight command is touched:
    /// `apdu_buffer`, `apdu_type`, `rx`, `rx_length`, `tx`, `tx_length` and
    /// `event_pending` are all left as they were. This deliberately bypasses
    /// [`Comm::reply`]/`apdu_send`, which would flush any staged response bytes,
    /// reset the response state and consume a SEPH event.
    fn reject_apdu(&self, sw: StatusWords) {
        let sw = sw as u16;
        let resp = [(sw >> 8) as u8, sw as u8];
        sys_seph::io_tx(self.io_buffer[0], &resp, resp.len());
    }

    /// Wait for the next Command event. Discards received button events.
    ///
    /// Like `next_event`, `T` can be any type, an enumeration, or any type
    /// which implements `TryFrom<ApduHeader>`.
    ///
    /// # Examples
    ///
    /// ```
    /// enum Instruction {
    ///     Select,
    ///     ReadBinary
    /// }
    ///
    /// impl TryFrom<ApduHeader> for Instruction {
    ///     type Error = StatusWords;
    ///
    ///     fn try_from(h: ApduHeader) -> Result<Self, Self::Error> {
    ///         match h.ins {
    ///             0xa4 => Ok(Self::Select),
    ///             0xb0 => Ok(Self::ReadBinary)
    ///             _ => Err(StatusWords::BadIns)
    ///         }
    ///     }
    /// }
    ///
    /// loop {
    ///     match comm.next_command() {
    ///         Instruction::Select => { ... }
    ///         Instruction::ReadBinary => { ... }
    ///     }
    /// }
    /// ```
    pub fn next_command<T>(&mut self) -> T
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        loop {
            if let Event::Command(ins) = self.next_event() {
                return ins;
            }
        }
    }

    /// Appends the status word to whichever response buffer `apdu_send` will
    /// transmit.
    ///
    /// The two response paths of this module are selected by `tx`: a caller that
    /// filled `apdu_buffer` sets `tx`, and one that filled `io_buffer` sets
    /// `tx_length`. `apdu_send` honours that split, so the status word has to be
    /// written into the buffer the same test will pick - writing it into
    /// `io_buffer` unconditionally left it in a buffer that was never sent.
    fn append_status_word(&mut self, sw: u16) {
        if self.tx != 0 {
            self.apdu_buffer[self.tx] = (sw >> 8) as u8;
            self.apdu_buffer[self.tx + 1] = sw as u8;
            self.tx += 2;
        } else {
            self.io_buffer[self.tx_length] = (sw >> 8) as u8;
            self.io_buffer[self.tx_length + 1] = sw as u8;
            self.tx_length += 2;
        }
    }

    /// Set the Status Word of the response to the previous Command event, and
    /// transmit the response.
    ///
    /// # Arguments
    ///
    /// * `sw` - Status Word to be transmitted after the Data. Can be a
    ///   StatusWords, a SyscallError, or any type which can be converted to a
    ///   Reply.
    pub fn reply<T: Into<Reply>>(&mut self, reply: T) {
        // Append status word
        self.append_status_word(reply.into().0);
        // Transmit the response
        self.apdu_send();
    }

    pub fn swap_reply<T: Into<Reply>>(&mut self, reply: T) {
        self.reply(reply);
    }

    /// Set the Status Word of the response to `StatusWords::OK` (which is equal
    /// to `0x9000`, and transmit the response.
    pub fn reply_ok(&mut self) {
        self.reply(StatusWords::Ok);
    }

    pub fn swap_reply_ok(&mut self) {
        self.reply_ok();
    }

    /// Return APDU Metadata
    pub fn get_apdu_metadata(&self) -> &ApduHeader {
        assert!(self.io_buffer.len() >= 5);
        let ptr = &self.io_buffer[1] as &u8 as *const u8 as *const ApduHeader;
        unsafe { &*ptr }
    }

    pub fn get_data(&self) -> Result<&[u8], StatusWords> {
        if self.rx == 4 {
            Ok(&[]) // Conforming zero-data APDU
        } else {
            let first_len_byte = self.apdu_buffer[4] as usize;
            let get_data_from_buffer = |len, offset| {
                if len == 0 || len + offset > self.rx {
                    Err(StatusWords::BadLen)
                } else {
                    Ok(&self.apdu_buffer[offset..offset + len])
                }
            };
            match (first_len_byte, self.rx) {
                (0, 5) => Ok(&[]), // Non-conforming zero-data APDU
                (0, 6) => Err(StatusWords::BadLen),
                (0, _) => {
                    let len =
                        u16::from_be_bytes([self.apdu_buffer[5], self.apdu_buffer[6]]) as usize;
                    get_data_from_buffer(len, 7)
                }
                (len, _) => get_data_from_buffer(len, 5),
            }
        }
    }

    pub fn get(&self, start: usize, end: usize) -> &[u8] {
        &self.io_buffer[start..end]
    }

    pub fn append(&mut self, m: &[u8]) {
        self.io_buffer[self.tx_length..self.tx_length + m.len()].copy_from_slice(m);
        self.tx_length += m.len();
    }
}

#[allow(dead_code)]
static mut CURRENT_COMM: *mut Comm = core::ptr::null_mut();

impl Drop for Comm {
    fn drop(&mut self) {
        // Unregister this instance from Nbgl.
        // SAFETY: single-threaded runtime; no concurrent access is possible.
        unsafe {
            if core::ptr::eq(CURRENT_COMM, self) {
                CURRENT_COMM = core::ptr::null_mut();
            }
        }
    }
}

/// The `Comm` instance registered with Nbgl.
///
/// # Panics
///
/// Panics if no instance is registered.
#[allow(dead_code)]
fn registered_comm() -> &'static mut Comm {
    // SAFETY: CURRENT_COMM is either null or points to the registered instance,
    // which unregisters itself when dropped. Single-threaded runtime.
    unsafe { CURRENT_COMM.as_mut() }.expect("No Comm instance registered")
}

#[allow(dead_code)]
fn default_nbgl_next_event_ahead() -> bool {
    registered_comm().next_event_ahead::<ApduHeader>()
}

#[allow(dead_code)]
fn default_nbgl_fetch_apdu_header() -> Option<ApduHeader> {
    let comm = registered_comm();
    if comm.event_pending && comm.rx_length >= 5 {
        return Some(*comm.get_apdu_metadata());
    }
    None
}

#[allow(dead_code)]
fn default_nbgl_reply_status(reply: Reply) {
    registered_comm().reply(reply);
}

pub(crate) const BOLOS_INS_GET_VERSION: u8 = 0x01;
pub(crate) const BOLOS_INS_QUIT: u8 = 0xa7;
pub(crate) const BOLOS_INS_SET_PKI_CERT: u8 = 0x06;
#[cfg(feature = "stack_usage")]
pub(crate) const BOLOS_INS_STACK_CONSUMPTION: u8 = 0x57;

/// True if the APDU with this `cla` / `ins` may be answered while another
/// command is in flight.
///
/// Only [`BOLOS_INS_GET_VERSION`] is.
pub(crate) fn is_bolos_apdu_allowed_in_flight(cla: u8, ins: u8) -> bool {
    cla == 0xB0 && ins == BOLOS_INS_GET_VERSION
}

// BOLOS APDU Handling (see https://developers.ledger.com/docs/connectivity/ledgerJS/open-close-info-on-apps)
fn handle_bolos_apdu(com: &mut Comm, ins: u8, p1: u8, p2: u8) {
    let _ = (p1, p2); // Some instructions may not use these parameters, avoid warnings
    match ins {
        // Get Information INS: retrieve App name and version
        BOLOS_INS_GET_VERSION => {
            unsafe {
                com.tx_length = 0;
                com.io_buffer[com.tx_length] = 0x01;
                com.tx_length += 1;
                let len = os_registry_get_current_app_tag(
                    BOLOS_TAG_APPNAME,
                    &mut com.io_buffer[com.tx_length + 1] as *mut u8,
                    (273 - com.tx_length - 2) as u32,
                );
                com.io_buffer[com.tx_length] = len as u8;
                com.tx_length += 1 + (len as usize);

                let len = os_registry_get_current_app_tag(
                    BOLOS_TAG_APPVERSION,
                    &mut com.io_buffer[com.tx_length + 1] as *mut u8,
                    (273 - com.tx_length - 2) as u32,
                );
                com.io_buffer[com.tx_length] = len as u8;
                com.tx_length += 1 + (len as usize);

                // to be fixed within io tasks
                // return OS flags to notify of platform's global state (pin lock etc)
                com.io_buffer[com.tx_length] = 1; // flags length
                com.tx_length += 1;
                com.io_buffer[com.tx_length] = os_flags() as u8;
                com.tx_length += 1;
            }
            com.reply_ok();
        }
        // Quit Application INS
        BOLOS_INS_QUIT => {
            com.reply_ok();
            crate::exit_app(0);
        }
        BOLOS_INS_SET_PKI_CERT => unsafe {
            let public_key = cx_ecfp_384_public_key_t::default();
            let err = os_pki_load_certificate(
                com.io_buffer[3],                // P1
                com.io_buffer[6..].as_mut_ptr(), // Data
                com.io_buffer[5] as usize,       // Length
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &public_key as *const cx_ecfp_384_public_key_t as *mut cx_ecfp_384_public_key_t,
            );
            if err != 0 {
                com.reply(SyscallError::from(PkiLoadCertificateError::from(err)));
            } else {
                com.reply_ok();
            }
        },
        #[cfg(feature = "stack_usage")]
        BOLOS_INS_STACK_CONSUMPTION => {
            crate::testing::handle_stack_consumption_apdu(p1, p2, com);
        }
        _ => {
            com.reply(StatusWords::BadIns);
        }
    }
}

impl Index<usize> for Comm {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.io_buffer[idx]
    }
}

impl IndexMut<usize> for Comm {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.tx_length = idx.max(self.tx_length);
        &mut self.io_buffer[idx]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_eq_err as assert_eq;
    use crate::testing::TestType;
    use testmacro::test_item as test;

    /// Basic "smoke test" that the casting is done correctly.
    #[test]
    fn apdu_metadata() {
        let c = Comm::new();
        let m = c.get_apdu_metadata();
        assert_eq!(m.cla, 0);
        assert_eq!(m.ins, 0);
        assert_eq!(m.p1, 0);
        assert_eq!(m.p2, 0);
    }

    /// A caller that filled `apdu_buffer` and set `tx` must get its status word
    /// in `apdu_buffer`, because that is the buffer `apdu_send` transmits when
    /// `tx` is non-zero. Writing it to `io_buffer` instead dropped the status
    /// word from every such response that carried data.
    #[test]
    fn status_word_follows_apdu_buffer() {
        let mut c = Comm::new();
        c.apdu_buffer[0] = 0xde;
        c.apdu_buffer[1] = 0xad;
        c.tx = 2;

        c.append_status_word(StatusWords::Ok as u16);

        assert_eq!(c.tx, 4);
        assert_eq!(c.apdu_buffer[2], 0x90);
        assert_eq!(c.apdu_buffer[3], 0x00);
        // The unused buffer must be left alone.
        assert_eq!(c.tx_length, 0);
        assert_eq!(c.io_buffer[0], 0);
        assert_eq!(c.io_buffer[1], 0);
    }

    /// The `io_buffer` path is selected when `tx` is zero, and keeps working.
    #[test]
    fn status_word_follows_io_buffer() {
        let mut c = Comm::new();
        c.io_buffer[0] = 0xde;
        c.io_buffer[1] = 0xad;
        c.tx_length = 2;

        c.append_status_word(StatusWords::Ok as u16);

        assert_eq!(c.tx_length, 4);
        assert_eq!(c.io_buffer[2], 0x90);
        assert_eq!(c.io_buffer[3], 0x00);
        assert_eq!(c.tx, 0);
    }
}
