use crate::bindings::*;
use crate::seph;
use crate::buttons::{ButtonEvent, ButtonsState, get_button_event};
use core::ops::{Index, IndexMut};
use crate::bindings::G_io_app;

#[derive(Copy, Clone)]
pub enum StatusWords {
    OK = 0x9000,
    NothingReceived = 0x6982,
    BadCLA = 0x6e00,
    BadLen = 0x6e01,
    UserCancelled = 0x6e02,
    Unknown = 0x6d00,
    Panic = 0xe000,
}


extern "C" {
    pub fn io_usb_hid_send(
        sndfct: unsafe extern "C" fn(*mut u8, u16), 
        sndlength: u16, 
        apdu_buffer: *const u8);
}

/// Possible events returned by [`Comm::next_event`]
pub enum Event {
    /// APDU event
    Command(u8),
    /// Button press or release event
    Button(ButtonEvent)
}

pub struct Comm {
    pub apdu_buffer: [u8; 260],
    pub rx: usize,
    pub tx: usize,
    buttons: ButtonsState
}

impl Comm {
    pub fn new() -> Comm {
        Comm {
            apdu_buffer: [0u8; 260],
            rx: 0,
            tx: 0,
            buttons: ButtonsState::new()
        }
    }

    /// Send the currently held APDU
    // This is private. Users should call reply to set the satus word and
    // transmit the response.
    fn apdu_send(&mut self) {
        if !seph::is_status_sent() {
            seph::send_general_status()
        }
        let mut spi_buffer = [0u8; 128];
        while seph::is_status_sent() {
            seph::seph_recv(&mut spi_buffer, 0);
            seph::handle_event(&mut self.apdu_buffer, &spi_buffer);
        }

        match unsafe { G_io_app.apdu_state } {
            APDU_USB_HID => {
                unsafe {
                    io_usb_hid_send(io_usb_send_apdu_data, self.tx as u16, self.apdu_buffer.as_ptr());
                }
            },
            APDU_RAW => { 
                let len = (self.tx as u16).to_be_bytes();
                seph::seph_send(&[seph::SephTags::RawAPDU as u8, len[0], len[1]]);
                seph::seph_send(&self.apdu_buffer[..self.tx]);
            }
            _ => ()
        }
        self.tx = 0;
        self.rx = 0;
        unsafe {G_io_app.apdu_state = APDU_IDLE;}
    }

    /// Wait and return next button press event or APDU command.
    ///
    /// # Examples
    ///
    /// ```
    /// loop {
    ///     match comm.next_event() {
    ///         Event::Button(button) => { ... }
    ///         Event::Command(0xa4) => { ... }
    ///         Event::Command(0xb0) => { ... }
    ///         _ => { ... }
    ///     }
    /// }
    /// ```
    pub fn next_event(&mut self) -> Event {
        let mut spi_buffer = [0u8; 128];

        unsafe { 
           G_io_app.apdu_state = APDU_IDLE;
           G_io_app.apdu_media = IO_APDU_MEDIA_NONE;
           G_io_app.apdu_length = 0; 
        }

        loop {

            // Signal end of command stream from SE to MCU
            // And prepare reception
            if !seph::is_status_sent() {
                seph::send_general_status();
            }

            // Fetch the next message from the MCU
            let _rx = seph::seph_recv(&mut spi_buffer, 0);

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
                    let button_info = spi_buffer[3]>>1;
                    if let Some(btn_evt) = get_button_event(&mut self.buttons, button_info) {
                        return Event::Button(btn_evt)
                    }
                },
                seph::Events::USBEvent => {
                    if len == 1 {
                        seph::handle_usb_event(&spi_buffer);
                    }
                },
                seph::Events::USBXFEREvent => {
                    if len >= 3 {
                        seph::handle_usb_ep_xfer_event(&mut self.apdu_buffer, &spi_buffer);
                    }
                },
                seph::Events::CAPDUEvent => seph::handle_capdu_event(&mut self.apdu_buffer, &spi_buffer),
                seph::Events::TickerEvent => { // unsafe{ G_io_app.ms += 100; }
                    // crate::debug_write("ticker");
                },
                _ => ()
            }

            if unsafe{G_io_app.apdu_state } != APDU_IDLE && unsafe {G_io_app.apdu_length } > 0 {
                self.rx = unsafe {G_io_app.apdu_length as usize };
                return Event::Command(self.apdu_buffer[1])
            }
        }
    }

    /// Wait for the next Command event. Returns the APDU Instruction byte value
    /// for easy instruction matching. Discards received button events.
    ///
    /// # Examples
    /// ```
    /// loop {
    ///     match comm.next_command() {
    ///         0xa4 => { ... }
    ///         0xb0 => { ... }
    ///         _ => { ... }
    ///     }
    /// }
    /// ```
    pub fn next_command(&mut self) -> u8 {
        loop {
            if let Event::Command(ins) = self.next_event() { return ins }
        }
    }

    /// Set the Status Word of the response to the previous Command event, and
    /// transmit the response.
    ///
    /// # Arguments
    ///
    /// * `sw` - Status Word to be transmitted after the Data.
    pub fn reply(&mut self, sw: StatusWords) {
        // Append status word
        self.apdu_buffer[self.tx] = ((sw as u16) >> 8) as u8;
        self.apdu_buffer[self.tx + 1] = sw as u8;
        self.tx += 2;
        // Transmit the response
        self.apdu_send();
    }

    /// Set the Status Word of the response to `StatusWords::OK` (which is equal
    /// to `0x9000`, and transmit the response.
    pub fn reply_ok(&mut self) {
        self.reply(StatusWords::OK);
    }

    /// Return APDU Class and Instruction bytes as a tuple
    pub fn get_cla_ins(&self) -> (u8, u8) {
        (self.apdu_buffer[0], self.apdu_buffer[1])
    }

    /// Returns APDU parameter P1
    pub fn get_p1(&self) -> u8 {
        self.apdu_buffer[2]
    }

    /// Returns APDU parameter P2
    pub fn get_p2(&self) -> u8 {
        self.apdu_buffer[3]
    }

    pub fn get_data(&self) -> Result<&[u8], StatusWords> {
        let len = u16::from_le_bytes([self.apdu_buffer[2], self.apdu_buffer[3]]) as usize;
        match len {
            0 => Err(StatusWords::BadLen),
            _ => Ok(&self.apdu_buffer[4..4+len])
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