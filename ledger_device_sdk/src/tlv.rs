//! Ledger TLV module
//!
//! Provides functions and types to parse TLV-encoded data.

use crate::io::Reply;

/// Tag type
pub type Tag = u32;

/// TLV parsing errors
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TlvError {
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid DER length
    InvalidDerLength,
    /// Length overflow
    LengthOverflow,
    /// Unknown tag
    UnknownTag,
    /// Handler failed  
    HandlerFailed,
    /// Duplicate unique tag
    DuplicateUniqueTag,
    /// Signature verification failed
    SignatureVerificationFailed,
}

impl From<TlvError> for Reply {
    fn from(exc: TlvError) -> Reply {
        Reply(0x7000 + exc as u16)
    }
}

/// Result type for TLV operations
pub type Result<T> = core::result::Result<T, TlvError>;

/// TLV data structure
#[derive(Copy, Clone, Debug)]
pub struct TlvData<'a> {
    /// Tag
    pub tag: Tag,
    /// Value
    pub value: &'a [u8],
    /// Raw TLV bytes (including tag and length)
    pub raw: &'a [u8],
}

/// TLV handler function type
pub type HandlerFn<O> = fn(&TlvData<'_>, &mut O) -> Result<bool>;

/// TLV handler structure
#[derive(Copy, Clone)]
pub struct Handler<O> {
    /// Tag
    pub tag: Tag,
    /// Is unique tag
    pub unique: bool,
    /// Handler function
    pub func: Option<HandlerFn<O>>, // None means "accept but do nothing"
}

/// Received tags tracker
struct Received {
    flags: u64,
    tag_to_flag: fn(Tag) -> u64,
}

/// Received tags tracker implementation
impl Received {
    /// Create a new Received tracker
    const fn new(map: fn(Tag) -> u64) -> Self {
        Self {
            flags: 0,
            tag_to_flag: map,
        }
    }
    /// Reset the received tags tracker
    pub fn reset(&mut self) {
        self.flags = 0;
    }
}

/// Set a unique tag as received
#[inline]
fn set_unique(received: &mut Received, tag: Tag) -> Result<()> {
    let f = (received.tag_to_flag)(tag);
    if f == 0 {
        return Ok(());
    }
    if (received.flags & f) != 0 {
        return Err(TlvError::DuplicateUniqueTag);
    }
    received.flags |= f;
    Ok(())
}

/// Decode a DER-encoded unsigned integer
fn der_u32(input: &[u8], off: &mut usize) -> Result<u32> {
    if *off >= input.len() {
        return Err(TlvError::UnexpectedEof);
    }
    let b0 = input[*off];
    *off += 1;
    if b0 & 0x80 == 0 {
        return Ok((b0 & 0x7F) as u32);
    }
    let n = (b0 & 0x7F) as usize;
    if n == 0 || n > 4 {
        return Err(TlvError::InvalidDerLength);
    }
    if *off + n > input.len() {
        return Err(TlvError::UnexpectedEof);
    }
    let mut v: u32 = 0;
    for _ in 0..n {
        v = (v << 8) | input[*off] as u32;
        *off += 1;
    }
    Ok(v)
}

/// TLV parsing configuration
pub struct ParseCfg<'a, O> {
    /// Handlers
    handlers: &'a [Handler<O>],
    /// Tag to flag mapping function
    tag_to_flag: fn(Tag) -> u64,
    /// Common handler (called before specific)
    pub common: Option<HandlerFn<O>>,
}

/// ParseCfg implementation
impl<'a, O> ParseCfg<'a, O> {
    /// Create a new ParseCfg
    pub const fn new(handlers: &'a [Handler<O>], map: fn(Tag) -> u64) -> Self {
        Self {
            handlers,
            tag_to_flag: map,
            common: None,
        }
    }
}

/// Parse TLV-encoded data
pub fn parse<'a, O>(cfg: &ParseCfg<'a, O>, payload: &'a [u8], tlv_out: &mut O) -> Result<()> {
    let mut received = Received::new(cfg.tag_to_flag);
    received.reset();

    let mut off = 0usize;
    while off < payload.len() {
        let tag_start = off;
        let tag = der_u32(payload, &mut off)?;
        let len = der_u32(payload, &mut off)? as usize;
        if off + len > payload.len() {
            return Err(TlvError::UnexpectedEof);
        }
        let val = &payload[off..off + len];
        let raw = &payload[tag_start..off + len];
        off += len;
        let data = TlvData {
            tag,
            value: val,
            raw,
        };
        if let Some(f) = cfg.common {
            if !f(&data, tlv_out)? {
                break;
            }
        }
        let h = cfg
            .handlers
            .iter()
            .find(|h| h.tag == tag)
            .ok_or(TlvError::UnknownTag)?;
        if let Some(f) = h.func {
            if !f(&data, tlv_out)? {
                break;
            }
        }
        if h.unique {
            set_unique(&mut received, tag)?;
        }
    }
    Ok(())
}

/// TlvData implementation
impl<'a> TlvData<'a> {
    /// Get the value as a byte slice
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.value
    }
    /// Get the value as a big-endian u64
    pub fn as_u64_be(&self) -> Result<u64> {
        match self.value.len() {
            1 => Ok(self.value[0] as u64),
            2 => Ok(u16::from_be_bytes([self.value[0], self.value[1]]) as u64),
            4 => Ok(
                u32::from_be_bytes([self.value[0], self.value[1], self.value[2], self.value[3]])
                    as u64,
            ),
            8 => Ok(u64::from_be_bytes([
                self.value[0],
                self.value[1],
                self.value[2],
                self.value[3],
                self.value[4],
                self.value[5],
                self.value[6],
                self.value[7],
            ])),
            _ => Err(TlvError::LengthOverflow),
        }
    }
    /// Get the value as a boolean
    pub fn as_bool(&self) -> Result<bool> {
        if self.value.len() != 1 {
            return Err(TlvError::LengthOverflow);
        }
        Ok(self.value[0] != 0)
    }
    /// Get the value as a UTF-8 string
    pub fn as_str(&self) -> Result<&'a str> {
        core::str::from_utf8(self.value).map_err(|_| TlvError::LengthOverflow)
    }
    /// Get the value as a byte slice with bounded length
    pub fn as_bounded(&self, min: usize, max: usize) -> Result<&'a [u8]> {
        if self.value.len() < min || self.value.len() > max {
            return Err(TlvError::LengthOverflow);
        }
        Ok(self.value)
    }
}

/// Macro to create a tag to flag mapping function
#[macro_export]
macro_rules! tag_to_flag_u64 {
    ($($tag:ident),+ $(,)?) => {
        const fn tag_to_flag_u64(t: Tag) -> u64 {
            let tags = [$($tag),+];
            let mut i = 0;
            while i < tags.len() {
                if tags[i] == t {
                    return 1 << i;
                }
                i += 1;
            }
            0
        }
    };
}
