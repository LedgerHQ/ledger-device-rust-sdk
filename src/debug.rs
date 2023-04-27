use core::arch::asm;

/// Debug 'print' function that uses ARM semihosting
/// Prints only strings with no formatting
pub fn print(s: &str) {
    let p = s.as_bytes().as_ptr();
    for i in 0..s.len() {
        let m = unsafe { p.add(i) };
        unsafe {
            asm!(
                "svc #0xab",
                in("r1") m,
                inout("r0") 3 => _,
            );
        }
    }
}

pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    ARR32([u8; 32])
}

pub fn to_hex_string<const N: usize>(val: Value) -> [u8; N] {
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

fn byte_to_hex(b: u8) -> (char, char) {
    let c0 = char::from_digit((b >> 4).into(), 16).unwrap();
    let c1 = char::from_digit((b & 0xf).into(), 16).unwrap();
    (c0,c1)
}