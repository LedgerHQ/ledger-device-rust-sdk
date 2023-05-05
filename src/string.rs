pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    ARR32([u8; 32])
}

pub fn to_utf8<const N: usize>(val: Value) -> [u8; N] {
    let mut hex: [u8; N]= [0u8; N];
    let mut i = 0;
    match val {
        Value::U8(b) => {
            for c in b.to_be_bytes().into_iter() {
                let (c0, c1) = byte_to_hex(c);
                hex[i] = c0 as u8;
                hex[i + 1] = c1 as u8;
                i += 2;        
            }
            return hex;
        }
        Value::U16(s) => {
            for c in s.to_be_bytes().into_iter() {
                let (c0, c1) = byte_to_hex(c);
                hex[i] = c0 as u8;
                hex[i + 1] = c1 as u8;
                i += 2;        
            }
            return hex;
        }
        Value::U32(l) => {
            for c in l.to_be_bytes().into_iter() {
                let (c0, c1) = byte_to_hex(c);
                hex[i] = c0 as u8;
                hex[i + 1] = c1 as u8;
                i += 2;        
            }
            return hex;
        }
        Value::ARR32(tab) => {
            for b in tab.into_iter() {
                let (c0, c1) = byte_to_hex(b);
                hex[i] = c0 as u8;
                hex[i + 1] = c1 as u8;
                i += 2; 
            }
            return hex;
        }
    }
}

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

pub fn uint256_to_float(value: &[u8;32], decimals: usize, out: &mut [u8], out_len: &mut usize ) {
    
    let mut tmp: [u8; 100] = [0; 100];
    let mut len: usize = 0;
    uint256_to_integer(value, &mut tmp[..], &mut len);

    out.fill(b'0');
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