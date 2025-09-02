extern crate proc_macro;

use image::*;
use proc_macro::TokenStream;
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
    let path = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        filename.value()
    );
    let mut grayscale_image: GrayImage = open(path).unwrap().to_luma8();
    let mut vec_output = Vec::new();

    match glyph_type {
        GlyphType::Bagl => {
            let packed = generate_bagl_glyph(&grayscale_image);
            write!(
                &mut vec_output,
                "(&{:?}, {}, {})",
                packed,
                grayscale_image.width(),
                grayscale_image.height()
            )
            .unwrap();
        }
        GlyphType::Nbgl => {
            let (buffer, bpp, is_file) = generate_nbgl_glyph(&mut grayscale_image);
            write!(
                &mut vec_output,
                "(&{:?}, {}, {}, {}, {})",
                buffer,
                grayscale_image.width(),
                grayscale_image.height(),
                bpp,
                is_file
            )
            .unwrap();
        }
    };

    let stream_output = std::str::from_utf8(&vec_output).unwrap();
    stream_output.parse().unwrap()
}

// Convert a frame into a bagl glyph : pack 8 pixels in a single byte.
// Each pixel is 1 bit, 0 for black, 1 for white.
fn generate_bagl_glyph(frame: &GrayImage) -> Vec<u8> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    // Number of pixels to be packed into bytes
    let size = width * height;
    let mut packed = Vec::with_capacity(size / 8);
    // Main loop, run through all pixels in the frame, by groups of 8
    for i in 0..size / 8 {
        let mut byte = 0;
        for j in 0..8 {
            // Compute linear index
            let idx = 8 * i + j;
            // Get x and y coordinates from linear index
            // Remainder of the division by width tells us how far we are on the x axis.
            let x = idx % width;
            // Integer division by width tells us how far we are on the y axis.
            let y = idx / width;
            let pixel = frame.get_pixel(x as u32, y as u32);
            // If pixel is not black (0), set the corresponding bit in the byte.
            let color = (pixel[0] != 0) as u8;
            // Set the j-th bit of the byte to the color of the pixel.
            byte |= color << j;
        }
        packed.push(byte);
    }
    // Remainder handling
    let remainder = size % 8;
    if remainder != 0 {
        let mut byte = 0;
        for j in 0..remainder {
            let x = (8 * (size / 8) + j) % width;
            let y = (8 * (size / 8) + j) / width;
            let pixel = frame.get_pixel(x as u32, y as u32);
            let color = (pixel[0] != 0) as u8;
            byte |= color << j;
        }
        packed.push(byte);
    }
    packed
}

fn image_to_packed_buffer(frame: &mut GrayImage, invert: bool) -> (Vec<u8>, u8) {
    // Count the number of colors in the image (max 16 supported)
    let mut color_count = std::collections::HashSet::new();
    for pixel in frame.pixels() {
        color_count.insert(pixel.0[0]);
    }
    let mut colors = std::cmp::min(16u8, color_count.len() as u8);

    // Round number of colors to a power of 2
    colors = colors.next_power_of_two();

    // Compute number of bits per pixel from number of colors (1, 2 or 4)
    let mut bits_per_pixel = std::cmp::min(4, (colors as f64).log(2.0).ceil() as u8);
    // 2 is not supported
    if bits_per_pixel == 2 {
        bits_per_pixel = 4;
    }

    // Invert if bpp is 1
    if bits_per_pixel == 1 && invert {
        for pixel in frame.pixels_mut() {
            pixel.0[0] = 255 - pixel.0[0];
        }
    }

    let width = frame.width();
    let height = frame.height();
    let base_threshold = (256 / colors as u32) as u8;
    let half_threshold = base_threshold / 2;
    let mut current_byte = 0 as u16;
    let mut current_bit = 0 as u16;
    let mut packed: Vec<u8> = Vec::new();

    for x in (0..width).rev() {
        for y in 0..height {
            let mut color: u16 = frame.get_pixel(x, y)[0] as u16;
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

fn generate_nbgl_glyph(frame: &mut GrayImage) -> (Vec<u8>, u8, bool) {
    // Special case for 14x14 images (Nano S+ and Nano X)
    if frame.width() == 14 && frame.height() == 14 {
        let (packed, bpp) = image_to_packed_buffer(frame, false);
        return (packed, bpp, false);
    }
    let (packed, bpp) = image_to_packed_buffer(frame, true);
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
        frame.width() as u8,
        (frame.width() >> 8) as u8,
        frame.height() as u8,
        (frame.height() >> 8) as u8,
        bpp_format << 4 | 1, // 1 is gzip compression type. We only support gzip.
        len as u8,
        (len >> 8) as u8,
        (len >> 16) as u8,
    ];

    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(&metadata);
    result.extend_from_slice(&compressed_image);

    (result, bpp, true)
}
