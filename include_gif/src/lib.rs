extern crate proc_macro;

use proc_macro::TokenStream;
use syn;
use std;
use std::fs::File;
use std::io::Write;

#[proc_macro]
pub fn include_gif(input: TokenStream) -> TokenStream {
    let filename = syn::parse_macro_input!(input as syn::LitStr);
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    let file = File::open(filename.value()).unwrap();
    let mut decoder = decoder.read_info(file).unwrap();

    let frame = decoder.read_next_frame().unwrap().unwrap().clone();
    let palette = decoder.palette().unwrap();
    let dimensions = frame.width * frame.height;
    let (size, remainder) = ((dimensions / 8) as usize, (dimensions % 8) as usize);
    
    let mut packed = Vec::new();
    for i in 0..size {
        let mut byte = 0;
        for j in 0..8 {
            let color = (palette[frame.buffer[8*i + j] as usize * 3] != 0) as u8;
            byte |= color << j;
        }
        packed.push(byte);
    }
    let mut byte = 0; 
    for j in 0..remainder {
        let color = (palette[frame.buffer[8*size + j] as usize * 3] != 0) as u8;
        byte |= color << j;
    }
    packed.push(byte);

    let mut b = Vec::new();
    write!(&mut b, "(&{:?}, {}, {})", packed, frame.width, frame.height).unwrap();
    let a = std::str::from_utf8(&b).unwrap();

    a.parse().unwrap()
}