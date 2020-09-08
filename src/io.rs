use crate::bindings::*;
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

    pub fn io_exch(&mut self, flags: u8) {
        let mut apdu_buf = apdu_buffer_t {
            buf: self.apdu_buffer.as_mut_ptr(),
            len: 260,
        };
        self.rx = unsafe{ io_exchange(flags, &mut apdu_buf, self.tx as u16) } as usize;
        self.tx = 0;
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

#[no_mangle]
pub extern "C" fn io_exchange_al(channel: u8, tx_len: u16, apdubuf: *mut u8) -> u16 {
    if channel == CHANNEL_SPI as u8 {
        if tx_len != 0 {
            unsafe { io_seph_recv(apdubuf, tx_len, 0) };
            1
        } else {
            unsafe { io_seph_recv(apdubuf, 256, 0) }
        }
    } else {
        0
    }
}
