#[cfg(target_os = "nanox")]
use crate::ble;
use ledger_secure_sdk_sys::buttons::{get_button_event, ButtonEvent, ButtonsState};
use ledger_secure_sdk_sys::seph as sys_seph;
use ledger_secure_sdk_sys::*;

#[cfg(feature = "ccid")]
use crate::ccid;
use crate::seph;
use core::convert::TryFrom;
use core::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum StatusWords {
    Ok = 0x9000,
    NothingReceived = 0x6982,
    BadCla = 0x6e00,
    BadIns = 0x6e01,
    BadP1P2 = 0x6e02,
    BadLen = 0x6e03,
    UserCancelled = 0x6e04,
    Unknown = 0x6d00,
    Panic = 0xe000,
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

extern "C" {
    pub fn io_usb_hid_send(
        sndfct: unsafe extern "C" fn(*mut u8, u16),
        sndlength: u16,
        apdu_buffer: *const u8,
    );
}

/// Possible events returned by [`Comm::next_event`]
#[derive(Eq, PartialEq)]
pub enum Event<T> {
    /// APDU event
    Command(T),
    /// Button press or release event
    Button(ButtonEvent),
    /// Ticker
    Ticker,
}

pub struct Comm {
    pub apdu_buffer: [u8; 260],
    pub rx: usize,
    pub tx: usize,
    buttons: ButtonsState,
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
    pub const fn new() -> Self {
        Self {
            apdu_buffer: [0u8; 260],
            rx: 0,
            tx: 0,
            buttons: ButtonsState::new(),
        }
    }

    /// Send the currently held APDU
    // This is private. Users should call reply to set the satus word and
    // transmit the response.
    fn apdu_send(&mut self) {
        if !sys_seph::is_status_sent() {
            sys_seph::send_general_status()
        }
        let mut spi_buffer = [0u8; 128];
        while sys_seph::is_status_sent() {
            sys_seph::seph_recv(&mut spi_buffer, 0);
            seph::handle_event(&mut self.apdu_buffer, &spi_buffer);
        }

        match unsafe { G_io_app.apdu_state } {
            APDU_USB_HID => unsafe {
                io_usb_hid_send(
                    io_usb_send_apdu_data,
                    self.tx as u16,
                    self.apdu_buffer.as_ptr(),
                );
            },
            APDU_RAW => {
                let len = (self.tx as u16).to_be_bytes();
                sys_seph::seph_send(&[sys_seph::SephTags::RawAPDU as u8, len[0], len[1]]);
                sys_seph::seph_send(&self.apdu_buffer[..self.tx]);
            }
            #[cfg(feature = "ccid")]
            APDU_USB_CCID => {
                ccid::send(&self.apdu_buffer[..self.tx]);
            }
            #[cfg(target_os = "nanox")]
            APDU_BLE => {
                ble::send(&self.apdu_buffer[..self.tx]);
            }
            _ => (),
        }
        self.tx = 0;
        self.rx = 0;
        unsafe {
            G_io_app.apdu_state = APDU_IDLE;
            G_io_app.apdu_media = IO_APDU_MEDIA_NONE;
            G_io_app.apdu_length = 0;
        }
    }

    /// Wait and return next button press event or APDU command.
    ///
    /// `T` can be an integer (usually automatically infered), which matches the
    /// Instruction byte of the APDU. In a more complex form, `T` can be any
    /// type which implements `TryFrom<u8>`. In particular, it is recommended to
    /// use an enumeration to enforce the compiler checking all possible
    /// commands are handled. Also, this method will automatically respond with
    /// an error status word if the Instruction byte is invalid (i.e. `try_from`
    /// failed).
    ///
    /// # Examples
    ///
    /// Simple use case with `T` infered as an `i32`:
    ///
    /// ```
    /// loop {
    ///     match comm.next_event() {
    ///         Event::Button(button) => { ... }
    ///         Event::Command(0xa4) => { ... }
    ///         Event::Command(0xb0) => { ... }
    ///         _ => { comm.reply(StatusWords::BadCLA) }
    ///     }
    /// }
    /// ```
    ///
    /// More complex example with an enumeration:
    ///
    /// ```
    /// enum Instruction {
    ///     Select,
    ///     ReadBinary
    /// }
    ///
    /// impl TryFrom<u8> for Instruction {
    ///     type Error = ();
    ///
    ///     fn try_from(v: u8) -> Result<Self, Self::Error> {
    ///         match v {
    ///             0xa4 => Ok(Self::Select),
    ///             0xb0 => Ok(Self::ReadBinary)
    ///             _ => Err(())
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Which can be used as the following:
    ///
    /// ```
    /// loop {
    ///     match comm.next_event() {
    ///         Event::Button(button) => { ... }
    ///         Event::Command(Instruction::Select) => { ... }
    ///         Event::Command(Instruction::ReadBinary) => { ... }
    ///     }
    /// }
    /// ```
    ///
    /// In this later example, invalid instruction byte error handling is
    /// automatically performed by the `next_event` method itself.
    pub fn next_event<T: TryFrom<ApduHeader>>(&mut self) -> Event<T> {
        let mut spi_buffer = [0u8; 128];

        unsafe {
            G_io_app.apdu_state = APDU_IDLE;
            G_io_app.apdu_media = IO_APDU_MEDIA_NONE;
            G_io_app.apdu_length = 0;
        }

        loop {
            // Signal end of command stream from SE to MCU
            // And prepare reception
            if !sys_seph::is_status_sent() {
                sys_seph::send_general_status();
            }

            // Fetch the next message from the MCU
            let _rx = sys_seph::seph_recv(&mut spi_buffer, 0);

            if let Some(value) = self.decode_event(&mut spi_buffer) {
                return value;
            }
        }
    }

    pub fn decode_event<T: TryFrom<ApduHeader>>(
        &mut self,
        spi_buffer: &mut [u8; 128],
    ) -> Option<Event<T>> {
        // message = [ tag, len_hi, len_lo, ... ]
        let tag = spi_buffer[0];
        let len = u16::from_be_bytes([spi_buffer[1], spi_buffer[2]]);

        // XXX: check whether this is necessary
        // if rx < 3 && rx != len+3 {
        //     unsafe {
        //        G_io_app.apdu_state = APDU_IDLE;
        //        G_io_app.apdu_length = 0;
        //     }
        //     return None
        // }

        // Treat all possible events.
        // If this is a button push, return with the associated event
        // If this is an APDU, return with the "received command" event
        // Any other event (usb, xfer, ticker) is silently handled
        match seph::Events::from(tag) {
            seph::Events::ButtonPush => {
                let button_info = spi_buffer[3] >> 1;
                if let Some(btn_evt) = get_button_event(&mut self.buttons, button_info) {
                    return Some(Event::Button(btn_evt));
                }
            }
            seph::Events::USBEvent => {
                if len == 1 {
                    seph::handle_usb_event(spi_buffer[3]);
                }
            }
            seph::Events::USBXFEREvent => {
                if len >= 3 {
                    seph::handle_usb_ep_xfer_event(&mut self.apdu_buffer, spi_buffer);
                }
            }
            seph::Events::CAPDUEvent => seph::handle_capdu_event(&mut self.apdu_buffer, spi_buffer),

            #[cfg(target_os = "nanox")]
            seph::Events::BleReceive => ble::receive(&mut self.apdu_buffer, spi_buffer),

            seph::Events::TickerEvent => return Some(Event::Ticker),
            _ => (),
        }

        if unsafe { G_io_app.apdu_state } != APDU_IDLE && unsafe { G_io_app.apdu_length } > 0 {
            self.rx = unsafe { G_io_app.apdu_length as usize };
            let res = T::try_from(*self.get_apdu_metadata());
            match res {
                Ok(ins) => {
                    return Some(Event::Command(ins));
                }
                Err(_) => {
                    // Invalid Ins code. Send automatically an error, mask
                    // the bad instruction to the application and just
                    // discard this event.
                    self.reply(StatusWords::BadIns);
                }
            }
        }
        None
    }

    /// Wait for the next Command event. Returns the APDU Instruction byte value
    /// for easy instruction matching. Discards received button events.
    ///
    /// Like `next_event`, `T` can be an integer, an enumeration, or any type
    /// which implements `TryFrom<u8>`.
    ///
    /// # Examples
    ///
    /// Simple use case with `T` infered as an `i32`:
    ///
    /// ```
    /// loop {
    ///     match comm.next_command() {
    ///         0xa4 => { ... }
    ///         0xb0 => { ... }
    ///         _ => { ... }
    ///     }
    /// }
    /// ```
    ///
    /// Other example with an enumeration:
    ///
    /// ```
    /// loop {
    ///     match comm.next_command() {
    ///         Instruction::Select => { ... }
    ///         Instruction::ReadBinary => { ... }
    ///     }
    /// }
    /// ```
    ///
    /// In this later example, invalid instruction byte error handling is
    /// automatically performed by the `next_command` method itself.
    pub fn next_command<T: TryFrom<ApduHeader>>(&mut self) -> T {
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
        self.apdu_buffer[self.tx] = (sw >> 8) as u8;
        self.apdu_buffer[self.tx + 1] = sw as u8;
        self.tx += 2;
        // Transmit the response
        self.apdu_send();
    }

    /// Set the Status Word of the response to `StatusWords::OK` (which is equal
    /// to `0x9000`, and transmit the response.
    pub fn reply_ok(&mut self) {
        self.reply(StatusWords::Ok);
    }

    /// Return APDU Metadata
    pub fn get_apdu_metadata(&self) -> &ApduHeader {
        assert!(self.apdu_buffer.len() >= 4);
        let ptr = &self.apdu_buffer[0] as &u8 as *const u8 as *const ApduHeader;
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
        &self.apdu_buffer[start..end]
    }

    pub fn append(&mut self, m: &[u8]) {
        for c in m.iter() {
            self.apdu_buffer[self.tx] = *c;
            self.tx += 1;
        }
    }
}

impl Index<usize> for Comm {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.apdu_buffer[idx]
    }
}

impl IndexMut<usize> for Comm {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.tx = idx.max(self.tx);
        &mut self.apdu_buffer[idx]
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
