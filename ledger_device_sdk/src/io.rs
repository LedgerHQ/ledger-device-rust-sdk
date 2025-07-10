#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_secure_sdk_sys::buttons::{get_button_event, ButtonEvent, ButtonsState};
use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

use crate::seph;

#[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
use crate::seph::ItcUxEvent;

use core::convert::{Infallible, TryFrom};
use core::ops::{Index, IndexMut};

#[cfg(any(target_os = "nanox", target_os = "stax", target_os = "flex"))]
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
            _ => SyscallError::Unspecified,
        }
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
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    Button(ButtonEvent),
    #[cfg(any(target_os = "stax", target_os = "flex"))]
    TouchEvent,
    /// Ticker
    Ticker,
}

/// Manages the communication of the device: receives events such as button presses, incoming
/// APDU requests, and provides methods to build and transmit APDU responses.
pub struct Comm {
    pub apdu_buffer: [u8; 272],
    pub rx: usize,
    pub tx: usize,
    pub event_pending: bool,
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
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
    pub const fn new() -> Self {
        Self {
            apdu_buffer: [0u8; 272],
            rx: 0,
            tx: 0,
            event_pending: false,
            #[cfg(not(any(target_os = "stax", target_os = "flex")))]
            buttons: ButtonsState::new(),
            expected_cla: None,
            apdu_type: seph::PacketTypes::PacketTypeNone as u8,
            io_buffer: [0u8; 273],
            rx_length: 0,
            tx_length: 0,
        }
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
    // This is private. Users should call reply to set the satus word and
    // transmit the response.
    fn apdu_send(&mut self) {
        #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
        {
            let mut buffer: [u8; 273] = [0; 273];
            let status = sys_seph::io_rx(&mut buffer, false);
            if status > 0 {
                let packet_type = seph::PacketTypes::from(buffer[0]);
                let event = seph::Events::from(buffer[1]);
                match (packet_type, event) {
                    (seph::PacketTypes::PacketTypeSeph, seph::Events::TickerEvent) => unsafe {
                        ux_process_ticker_event();
                    },
                    (_, _) => {}
                }
            }
        }
        if self.tx != 0 {
            sys_seph::io_tx(self.apdu_type, &self.apdu_buffer, self.tx);
            self.tx = 0;
        } else {
            sys_seph::io_tx(self.apdu_type, &self.io_buffer, self.tx_length);
        }
        self.tx_length = 0;
        self.rx_length = 0;
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

            if status > 0 {
                if let Some(value) = self.decode_event(status) {
                    return value;
                }
            }
        }
    }

    pub fn next_event_ahead<T>(&mut self) -> bool
    where
        T: TryFrom<ApduHeader>,
        Reply: From<<T as TryFrom<ApduHeader>>::Error>,
    {
        let status = sys_seph::io_rx(&mut self.io_buffer, true);

        if status > 0 {
            return self.detect_apdu::<T>(status);
        }
        return false;
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

            // Manage BOLOS specific APDUs B0xx0000
            if self.io_buffer[1] == 0xB0 && self.io_buffer[3] == 0x00 && self.io_buffer[4] == 0x00 {
                handle_bolos_apdu(self, self.io_buffer[2]);
                return None;
            }

            // If CLA filtering is enabled, automatically reject APDUs with wrong CLA
            if let Some(cla) = self.expected_cla {
                if self.io_buffer[1] != cla {
                    self.reply(StatusWords::BadCla);
                    return None;
                }
            }

            let res = T::try_from(*self.get_apdu_metadata());
            match res {
                Ok(ins) => {
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
            #[cfg(not(any(target_os = "stax", target_os = "flex")))]
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
            #[cfg(any(target_os = "stax", target_os = "flex"))]
            seph::Events::ScreenTouchEvent => unsafe {
                ux_process_finger_event(seph_buffer.as_mut_ptr());
                return Some(Event::TouchEvent);
            },

            // TICKER EVENT
            seph::Events::TickerEvent => {
                #[cfg(any(target_os = "stax", target_os = "flex", feature = "nano_nbgl"))]
                unsafe {
                    ux_process_ticker_event();
                }
                return Some(Event::Ticker);
            }

            // ITC EVENT
            seph::Events::ItcEvent => {
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

                    _ => return None,
                }
                return None;
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
                unsafe {
                    if os_perso_is_pin_set() == BOLOS_TRUE.try_into().unwrap()
                        && os_global_pin_is_validated() != BOLOS_TRUE.try_into().unwrap()
                    {
                        self.reply(StatusWords::DeviceLocked);
                        return None;
                    }
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
                return true;
            }
            _ => return false,
        }
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

    /// Set the Status Word of the response to the previous Command event, and
    /// transmit the response.
    ///
    /// # Arguments
    ///
    /// * `sw` - Status Word to be transmitted after the Data. Can be a
    ///   StatusWords, a SyscallError, or any type which can be converted to a
    ///   Reply.
    pub fn reply<T: Into<Reply>>(&mut self, reply: T) {
        let sw = reply.into().0;
        // Append status word
        self.io_buffer[self.tx_length] = (sw >> 8) as u8;
        self.io_buffer[self.tx_length + 1] = sw as u8;
        self.tx_length += 2;
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
                        u16::from_le_bytes([self.apdu_buffer[5], self.apdu_buffer[6]]) as usize;
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

// BOLOS APDU Handling (see https://developers.ledger.com/docs/connectivity/ledgerJS/open-close-info-on-apps)
fn handle_bolos_apdu(com: &mut Comm, ins: u8) {
    match ins {
        // Get Information INS: retrieve App name and version
        0x01 => {
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
        0xa7 => {
            com.reply_ok();
            crate::exit_app(0);
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
}
