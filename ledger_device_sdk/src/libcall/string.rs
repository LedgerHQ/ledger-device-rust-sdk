#[derive(Debug, Copy, Clone)]
pub struct CustomString<const N: usize> {
    pub arr: [u8; N],
    pub capacity: usize,
    pub len: usize,
}

impl<const N: usize> Default for CustomString<N> {
    fn default() -> Self {
        Self {
            arr: [b'0'; N],
            capacity: N,
            len: 0,
        }
    }
}

impl<const N: usize> CustomString<N> {
    pub fn new() -> Self {
        Self {
            arr: [b'0'; N],
            capacity: N,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.arr.fill(0);
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.arr[..self.len]).unwrap()
    }

    pub fn copy_from(&mut self, s: &CustomString<N>) {
        self.arr[..s.len].copy_from_slice(&s.arr[..s.len]);
        self.len = s.len;
    }
}

impl From<u8> for CustomString<2> {
    fn from(val: u8) -> Self {
        let mut s = CustomString::<2>::new();
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

impl From<u16> for CustomString<4> {
    fn from(val: u16) -> Self {
        let mut s = CustomString::<4>::new();
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

impl From<u32> for CustomString<8> {
    fn from(val: u32) -> Self {
        let mut s = CustomString::<8>::new();
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

impl From<[u8; 32]> for CustomString<64> {
    fn from(arr: [u8; 32]) -> Self {
        let mut s = CustomString::<64>::new();
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

impl<const N: usize> TryFrom<&str> for CustomString<N> {
    type Error = &'static str;
    fn try_from(st: &str) -> Result<Self, Self::Error> {
        if N >= st.len() {
            let mut s = CustomString::<N>::new();
            s.arr[..st.len()].copy_from_slice(st.as_bytes());
            s.len = st.len();
            Ok(s)
        } else {
            Err("CustomString's capacity overflow!")
        }
    }
}

/// Output an uint256 as an decimal CustomString
/// For instance:
///
/// let val: [u8; 32] = token amount (32 bytes / 256 bits);
/// let s: CustomString<79> = uint256_to_integer(&val); // max number of decimal digits for Uint256 = 78 (+ 1 spare for '.')
/// testing::debug_print(s.print().unwrap());
pub fn uint256_to_integer(value: &[u8; 32]) -> CustomString<79> {
    let mut s: CustomString<79> = CustomString::new();

    // Special case when value is 0
    if *value == [0u8; 32] {
        s.arr[0] = b'0';
        s.len = 1;
        return s;
    }

    let mut n: [u16; 16] = [0u16; 16];
    for idx in 0..16 {
        n[idx] = u16::from_be_bytes([value[2 * idx], value[2 * idx + 1]]);
    }

    let mut pos: usize = s.capacity;
    while n != [0u16; 16] {
        if pos == 0 {
            return s;
        }
        pos -= 1;
        let mut carry = 0u32;
        let mut rem: u32;
        for i in 0..16 {
            rem = ((carry << 16) | u32::from(n[i])) % 10;
            n[i] = (((carry << 16) | u32::from(n[i])) / 10) as u16;
            carry = rem;
        }
        s.arr[pos] = u8::try_from(char::from_digit(carry, 10).unwrap()).unwrap();
    }
    s.arr.copy_within(pos.., 0);
    s.len = s.capacity - pos;
    s
}

/// Output an uint256 as a float string
pub fn uint256_to_float(value: &[u8; 32], decimals: usize) -> CustomString<79> {
    let mut s: CustomString<79> = uint256_to_integer(value);

    if decimals == 0 || s.arr[0] == b'0' {
        return s;
    }

    if s.len <= decimals {
        s.arr.copy_within(0..s.len, 2 + decimals - s.len);
        s.arr[0..2 + decimals - s.len].fill(b'0');
        s.arr[1] = b'.';
        s.len += 2 + decimals - s.len;
    } else {
        s.arr
            .copy_within(s.len - decimals..s.len, s.len - decimals + 1);
        s.arr[s.len - decimals] = b'.';
        s.len += 1;
    }
    s
}

fn byte_to_hex(b: u8) -> (char, char) {
    let c0 = char::from_digit((b >> 4).into(), 16).unwrap();
    let c1 = char::from_digit((b & 0xf).into(), 16).unwrap();
    (c0, c1)
}
