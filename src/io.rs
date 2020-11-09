use crate::bindings::*;
use crate::seph;
use crate::buttons::{ButtonEvent, ButtonsState, get_button_event};
use core::ops::{Index, IndexMut};

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


/// App-visible events:
/// - APDU received (=command)
/// - Button press event
pub enum GlobalEvent {
    CommandReceived,
    ButtonEvent(ButtonEvent)
}

pub struct Comm {
    pub apdu_buffer: [u8; 260],
    pub rx: usize,
    pub tx: usize,
}

impl Comm {
    pub fn new() -> Comm {
        Comm {
            apdu_buffer: [0u8; 260],
            rx: 0,
            tx: 0,
        }
    }

    /// Send the currently held APDU
    pub fn apdu_send(&mut self) {
        if !seph::is_status_sent() {
            seph::send_general_status()
        }
        let len = (self.tx as u16).to_be_bytes();
        seph::seph_send(&[seph::SephTags::RawAPDU as u8, len[0], len[1]]);
        seph::seph_send(&self.apdu_buffer[..self.tx]);
        self.tx = 0;
        self.rx = 0;
        unsafe { seph::G_io_app.apdu_state = APDU_IDLE;}
    }

    /// Wait for either a button press or an APDU.
    /// Useful for implementing a main menu/welcome screen.
    pub fn wait_for_event(&mut self, mut buttons: &mut ButtonsState) -> GlobalEvent {
        let mut spi_buffer = [0u8; 128];

        unsafe { 
            seph::G_io_app.apdu_state = APDU_IDLE;
            seph::G_io_app.apdu_media = IO_APDU_MEDIA_NONE;
            seph::G_io_app.apdu_length = 0; 
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
            //         seph::G_io_app.apdu_state = APDU_IDLE;
            //         seph::G_io_app.apdu_length = 0;
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
                    if let Some(btn_evt) = get_button_event(&mut buttons, button_info) {
                        return GlobalEvent::ButtonEvent(btn_evt)
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

            if unsafe{ seph::G_io_app.apdu_state } != APDU_IDLE && unsafe { seph::G_io_app.apdu_length } > 0 {
                self.rx = unsafe { seph::G_io_app.apdu_length as usize };
                return GlobalEvent::CommandReceived
            }
        }
    }

    /// Wait for an APDU without treating button presses
    pub fn apdu_receive(&mut self) -> Option<usize> {
        // crate::debug_write("apdu_recv\n");
        let mut spi_buffer = [0u8; 128];
        unsafe { 
            seph::G_io_app.apdu_state = APDU_IDLE;
            seph::G_io_app.apdu_media = IO_APDU_MEDIA_NONE;
            seph::G_io_app.apdu_length = 0; 
        }
        self.rx = 0;
        loop 
        {
            if !seph::is_status_sent() {
                seph::send_general_status();
            }
            
            let rx = seph::seph_recv(&mut spi_buffer, 0);
            let len = u16::from_be_bytes([spi_buffer[1], spi_buffer[2]]);
            if rx < 3 && rx != len {
                unsafe {
                    seph::G_io_app.apdu_state = APDU_IDLE;
                    seph::G_io_app.apdu_length = 0;
                }
                return None
            }
            seph::handle_event(&mut self.apdu_buffer, &mut spi_buffer);
            if unsafe{ seph::G_io_app.apdu_state } != APDU_IDLE && unsafe { seph::G_io_app.apdu_length } > 0 {
                self.rx = unsafe { seph::G_io_app.apdu_length as usize };
                return Some(self.rx)
            }
        }
    }

    pub fn set_status_word(&mut self, sw: StatusWords) {
        self.apdu_buffer[self.tx] = ((sw as u16) >> 8) as u8;
        self.apdu_buffer[self.tx + 1] = sw as u8;
        self.tx += 2;
    }

    pub fn get_cla_ins(&self) -> (u8, u8) {
        (self.apdu_buffer[0], self.apdu_buffer[1])
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