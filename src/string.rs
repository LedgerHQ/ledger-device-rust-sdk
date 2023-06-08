#[derive(Debug, Copy, Clone)]
pub struct String<const N: usize> {
    pub arr: [u8; N],
    pub capacity: usize,
    pub len: usize
}

impl<const N: usize> String<N> {
    pub fn new() -> Self {
        Self {
            arr: [0u8; N],
            capacity: N,
            len: 0
        }
    }

    pub fn clear(&mut self) {
        self.arr.fill(0);
        self.len = 0;
    }

}

impl From<u8> for String<2> {
    fn from(val: u8) -> Self {
        let mut s = String::<2>::new();
        let mut i: usize = 0;
        for c in val.to_be_bytes().into_iter() {
            let (c0, c1) = byte_to_hex(c);
            s.arr[i] = c0 as u8;
            s.arr[i + 1] = c1 as u8;
            s.len += 2;
            i += 2;        
        }
        s
    }
}

impl From<u16> for String<4> {
    fn from(val: u16) -> Self {
        let mut s = String::<4>::new();
        let mut i: usize = 0;
        for c in val.to_be_bytes().into_iter() {
            let (c0, c1) = byte_to_hex(c);
            s.arr[i] = c0 as u8;
            s.arr[i + 1] = c1 as u8;
            s.len += 2;
            i += 2;        
        }
        s
    }
}

impl From<u32> for String<8> {
    fn from(val: u32) -> Self {
        let mut s = String::<8>::new();
        let mut i: usize = 0;
        for c in val.to_be_bytes().into_iter() {
            let (c0, c1) = byte_to_hex(c);
            s.arr[i] = c0 as u8;
            s.arr[i + 1] = c1 as u8;
            s.len += 2;
            i += 2;        
        }
        s
    }
}

impl From<[u8; 32]> for String<64> {
    fn from(arr: [u8; 32]) -> Self {
        let mut s = String::<64>::new();
        let mut i: usize = 0;
        for c in arr.into_iter() {
            let (c0, c1) = byte_to_hex(c);
            s.arr[i] = c0 as u8;
            s.arr[i + 1] = c1 as u8;
            s.len += 2;
            i += 2;        
        }
        s
    }
}

impl<const N: usize> TryFrom<&str> for String<N> {
    type Error = &'static str;
    fn try_from(st: &str) -> Result<Self, Self::Error> {
        if N >= st.len() {
            let mut s = String::<N>::new();
            s.arr[..st.len()].copy_from_slice(st.as_bytes());
            s.len = st.len();
            Ok(s)
        }
        else {
            Err("String's capacity overflow!")
        }
    }
}

/// Output an uint256 as an decimal string
/// For instance:
///
/// let val: [u8; 32] = token amount (felt, 32 bytes);
/// let mut out: [u8; 100] = [0; 100];
/// let mut out_len: usize = 0;
/// uint256_to_integer(&val, &mut out[..], &mut out_len);
/// testing::debug_print(core::str::from_utf8(&out[..out_len]));
pub fn uint256_to_integer(value: &[u8; 32], out: &mut [u8], out_len: &mut usize) {
    // Special case when value is 0
    if *value == [0u8; 32] {
        if out.len() < 2 {
            return;
        }
        out[0] = b'0';
        *out_len = 1;
        return;
    }

    let mut n: [u16; 16] = [0u16; 16];
    for idx in 0..16 {
        n[idx] = u16::from_be_bytes([value[2 * idx], value[2 * idx + 1]]);
    }

    let mut pos: usize = out.len();
    while n != [0u16; 16] {
        if pos == 0 {
            return;
        }
        pos -= 1;
        let mut carry = 0u32;
        let mut rem: u32;
        for i in 0..16 {
            rem = ((carry << 16) | u32::from(n[i])) % 10;
            n[i] = (((carry << 16) | u32::from(n[i])) / 10) as u16;
            carry = rem;
        }
        out[pos] = u8::try_from(char::from_digit(carry, 10).unwrap()).unwrap(); 
    }
    out.copy_within(pos.., 0);
    *out_len = out.len() - pos;

    return;
}

/// Output an uint256 as a float string
/// For instance:
///
/// let val: [u8; 32] = token amount (felt, 32 bytes);
/// let mut out: [u8; 100] = [0; 100];
/// let mut out_len: usize = 0;
/// uint256_to_float(&val, &mut out[..], &mut out_len);
/// testing::debug_print(core::str::from_utf8(&out[..out_len]));
pub fn uint256_to_float(value: &[u8;32], decimals: usize, out: &mut [u8], out_len: &mut usize ) {
    
    let mut tmp: [u8; 100] = [0; 100];
    let mut len: usize = 0;

    uint256_to_integer(value, &mut tmp[..], &mut len);
    out.fill(b'0');

    if decimals == 0 {
        out[0..len].copy_from_slice(&tmp[..len]);
        *out_len = len;
        return;
    }

    if len <= decimals {
        out[1] = b'.';
        out[2 + decimals - len..2 + decimals].copy_from_slice(&tmp[..len]);
        *out_len = 2 + decimals;
    }
    else {
        let delta = len - decimals;
        let part = &tmp[0..len];
        let (ipart, dpart) = part.split_at(delta);
        out[0..delta].copy_from_slice(ipart);
        out[delta] = b'.';
        out[delta + 1..delta + 1 + dpart.len()].copy_from_slice(dpart);
        *out_len = ipart.len() + dpart.len() + 1;
    }
}

fn byte_to_hex(b: u8) -> (char, char) {
    let c0 = char::from_digit((b >> 4).into(), 16).unwrap();
    let c1 = char::from_digit((b & 0xf).into(), 16).unwrap();
    (c0,c1)
}