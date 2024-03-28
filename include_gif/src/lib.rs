extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use syn::{parse_macro_input, Ident, LitStr};

enum BppFormat {
    Bpp1 = 0,
    Bpp2 = 1,
    Bpp4 = 2,
}

enum GlyphType {
    Bagl,
    Nbgl,
}

enum Input {
    FileNameOnly(LitStr),
    FileNameAndType(LitStr, GlyphType),
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse the filename
        let filename = input.parse::<LitStr>()?;
        if input.is_empty() {
            Ok(Input::FileNameOnly(filename))
        } else {
            // Parse comma separator (won't be used, just to skip it)
            let _: syn::Token![,] = input.parse()?;
            // Parse the glyph type
            let glyph_type = input.parse::<Ident>()?;
            match glyph_type.to_string().as_str() {
                "BAGL" => Ok(Input::FileNameAndType(filename, GlyphType::Bagl)),
                "NBGL" => Ok(Input::FileNameAndType(filename, GlyphType::Nbgl)),
                _ => Err(syn::Error::new_spanned(glyph_type, "Invalid glyph type")),
            }
        }
    }
}

#[proc_macro]
pub fn include_gif(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    match input {
        Input::FileNameOnly(filename) => {
            // Default to Bagl if no type is specified.
            let generate_type = GlyphType::Bagl;
            generate_glyph(filename, generate_type)
        }
        Input::FileNameAndType(filename, generate_type) => generate_glyph(filename, generate_type),
    }
}

fn generate_glyph(filename: LitStr, glyph_type: GlyphType) -> TokenStream {
    let filename = filename.value();
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);

    let path = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        filename
    );
    let file = File::open(path).unwrap();
    let mut decoder = decoder.read_info(file).unwrap();

    let frame = decoder.read_next_frame().unwrap().unwrap().clone();
    let palette = decoder.palette().unwrap();
    let mut vec_output = Vec::new();

    match glyph_type {
        GlyphType::Bagl => {
            let packed = generate_bagl_glyph(&frame, &palette);
            write!(
                &mut vec_output,
                "(&{:?}, {}, {})",
                packed, frame.width, frame.height
            )
            .unwrap();
        }
        GlyphType::Nbgl => {
            let (compressed_buffer, bpp) = generate_nbgl_glyph(&frame, &palette);
            write!(
                &mut vec_output,
                "(&{:?}, {}, {}, {}, {})",
                compressed_buffer, frame.width, frame.height, bpp, true
            )
            .unwrap();
        }
    };

    let stream_output = std::str::from_utf8(&vec_output).unwrap();
    stream_output.parse().unwrap()
}

fn generate_bagl_glyph(frame: &gif::Frame, palette: &[u8]) -> Vec<u8> {
    let dimensions = frame.width * frame.height;
    let (size, remainder) = ((dimensions / 8) as usize, (dimensions % 8) as usize);

    let mut packed = Vec::new();
    for i in 0..size {
        let mut byte = 0;
        for j in 0..8 {
            let color = (palette[frame.buffer[8 * i + j] as usize * 3] != 0) as u8;
            byte |= color << j;
        }
        packed.push(byte);
    }
    let mut byte = 0;
    for j in 0..remainder {
        let color = (palette[frame.buffer[8 * size + j] as usize * 3] != 0) as u8;
        byte |= color << j;
    }
    packed.push(byte);
    packed
}

fn image_to_packed_buffer(frame: &gif::Frame, palette: &[u8]) -> (Vec<u8>, u8) {
    let mut colors = palette.iter().collect::<HashSet<_>>().len() as u8;

    // Exit/Panic if number of colors > 16
    if colors > 16 {
        panic!("Image has more than 16 colors");
    }
    // Round number of colors to a power of 2
    if !(colors != 0 && colors.count_ones() == 1) {
        colors = (2.0_f64.powf((colors as f64).log2().ceil())) as u8;
    }

    let mut bits_per_pixel: u8 = (colors as f32).log2().floor() as u8;
    match bits_per_pixel {
        0 => bits_per_pixel = 1,
        3 => bits_per_pixel = 4,
        _ => (),
    }

    let width = frame.width;
    let height = frame.height;
    let base_threshold = (256 / colors as u32) as u8;
    let half_threshold = base_threshold / 2;
    let mut current_byte = 0 as u16;
    let mut current_bit = 0 as u16;
    let mut packed: Vec<u8> = Vec::new();

    for x in (0..width).rev() {
        for y in 0..height {
            let pixel_index = ((y * width) + x) as usize;
            let mut color: u16 = palette[frame.buffer[pixel_index] as usize * 3] as u16;
            color = (color + half_threshold as u16) / base_threshold as u16;
            if color >= colors as u16 {
                color = colors as u16 - 1;
            }
            current_byte += color << ((8 - bits_per_pixel as u16) - current_bit);
            current_bit += bits_per_pixel as u16;
            if current_bit >= 8 {
                packed.push(current_byte as u8 & 0xFF);
                current_byte = 0;
                current_bit = 0;
            }
        }
    }
    if current_bit > 0 {
        packed.push(current_byte as u8 & 0xFF);
    }
    (packed, bits_per_pixel)
}

fn generate_nbgl_glyph(frame: &gif::Frame, palette: &[u8]) -> (Vec<u8>, u8) {
    let (packed, bpp) = image_to_packed_buffer(&frame, &palette);

    let mut compressed_image: Vec<u8> = Vec::new();
    let mut full_uncompressed_size = packed.len();
    let mut i = 0;

    while full_uncompressed_size > 0 {
        let chunk_size = std::cmp::min(2048, full_uncompressed_size);
        let tmp = &packed[i..i + chunk_size];

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(tmp).unwrap();
        let compressed_buffer = encoder.finish().unwrap();

        let compressed_len = compressed_buffer.len();
        let len_bytes: [u8; 2] = [
            (compressed_len & 0xFF) as u8,
            ((compressed_len >> 8) & 0xFF) as u8,
        ];

        compressed_image.extend_from_slice(&len_bytes);
        compressed_image.extend_from_slice(&compressed_buffer);

        full_uncompressed_size -= chunk_size;
        i += chunk_size;
    }

    let bpp_format: u8 = match bpp {
        1 => BppFormat::Bpp1 as u8,
        2 => BppFormat::Bpp2 as u8,
        4 => BppFormat::Bpp4 as u8,
        _ => panic!("Invalid bpp"),
    };
    
    let len = compressed_image.len();
    let metadata: [u8; 8] = [
        frame.width as u8,
        (frame.width >> 8) as u8,
        frame.height as u8,
        (frame.height >> 8) as u8,
        bpp_format << 4 | 1, // 1 is gzip compression type. We only support gzip.
        len as u8,
        (len >> 8) as u8,
        (len >> 16) as u8,
    ];

    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(&metadata);
    result.extend_from_slice(&compressed_image);

    (result, bpp)
}
