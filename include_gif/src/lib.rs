extern crate proc_macro;

use proc_macro::TokenStream;
use std::io::Write;
use syn::{Ident, LitStr, parse_macro_input};

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

// ---------------------------------------------------------------------------
// Minimal grayscale image + PNG/GIF decoding
//
// This crate only needs to turn a PNG or GIF icon into an 8-bit grayscale
// buffer. Rather than depend on the full `image` crate (which pulls in a color
// management stack via `moxcms`/`pxfm`, `bytemuck`, ...), we decode directly
// with the `png` and `gif` crates and reproduce `image`'s `to_luma8()`
// conversion byte-for-byte. The conversions below mirror `image`'s
// `FromColor`/`FromPrimitive` impls (see `include_gif`'s snapshot test, which
// pins the output against the previous `image`-based implementation).
// ---------------------------------------------------------------------------

/// A single 8-bit grayscale pixel, mirroring `image::Luma<u8>`'s API.
#[derive(Clone, Copy)]
struct Luma(pub [u8; 1]);

impl core::ops::Index<usize> for Luma {
    type Output = u8;
    fn index(&self, i: usize) -> &u8 {
        &self.0[i]
    }
}

/// An 8-bit grayscale image, a drop-in subset of `image::GrayImage`.
struct GrayImage {
    width: u32,
    height: u32,
    pixels: Vec<Luma>,
}

impl GrayImage {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get_pixel(&self, x: u32, y: u32) -> Luma {
        self.pixels[(y * self.width + x) as usize]
    }

    fn pixels(&self) -> impl Iterator<Item = &Luma> {
        self.pixels.iter()
    }

    fn pixels_mut(&mut self) -> impl Iterator<Item = &mut Luma> {
        self.pixels.iter_mut()
    }
}

/// sRGB -> luma, 8-bit domain. Matches `image`'s integer `rgb_to_luma`:
/// `(2126*R + 7152*G + 722*B) / 10000` with truncating division.
fn luma8_from_rgb8(r: u8, g: u8, b: u8) -> u8 {
    ((2126 * r as u32 + 7152 * g as u32 + 722 * b as u32) / 10000) as u8
}

/// sRGB -> luma, 16-bit domain (used before downscaling 16-bit PNGs).
fn luma16_from_rgb16(r: u16, g: u16, b: u16) -> u16 {
    ((2126 * r as u64 + 7152 * g as u64 + 722 * b as u64) / 10000) as u16
}

/// 16-bit -> 8-bit sample, matching `image`'s `FromPrimitive<u16> for u8`:
/// `round(c * 255 / 65535)`, implemented as `(c + 128) / 257`.
fn u16_to_u8(c16: u16) -> u8 {
    ((c16 as u32 + 128) / 257) as u8
}

fn be16(p: &[u8]) -> u16 {
    ((p[0] as u16) << 8) | p[1] as u16
}

/// Decode an image file into 8-bit grayscale, dispatching on file extension.
/// Mirrors `image::open(path).to_luma8()` for the PNG and GIF inputs this
/// macro supports.
fn open_as_luma(path: &str) -> GrayImage {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".png") {
        decode_png(path)
    } else if lower.ends_with(".gif") {
        decode_gif(path)
    } else {
        panic!("include_gif: unsupported file extension (expected .png or .gif): {path}");
    }
}

fn decode_png(path: &str) -> GrayImage {
    let file = std::fs::File::open(path)
        .unwrap_or_else(|e| panic!("include_gif: cannot open {path}: {e}"));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    // Same transformation `image` applies: expand palette, sub-8-bit grayscale
    // and tRNS so we only ever see Grayscale/GrayscaleAlpha/Rgb/Rgba samples.
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder
        .read_info()
        .unwrap_or_else(|e| panic!("include_gif: invalid PNG {path}: {e}"));
    let mut buf = vec![
        0u8;
        reader
            .output_buffer_size()
            .unwrap_or_else(|| panic!("include_gif: PNG too large {path}"))
    ];
    let info = reader
        .next_frame(&mut buf)
        .unwrap_or_else(|e| panic!("include_gif: cannot decode PNG {path}: {e}"));
    let data = &buf[..info.buffer_size()];

    use png::{BitDepth::*, ColorType::*};
    let pixels: Vec<Luma> = match (info.color_type, info.bit_depth) {
        (Grayscale, Eight) => data.iter().map(|&v| Luma([v])).collect(),
        (GrayscaleAlpha, Eight) => data.chunks_exact(2).map(|p| Luma([p[0]])).collect(),
        (Rgb, Eight) => data
            .chunks_exact(3)
            .map(|p| Luma([luma8_from_rgb8(p[0], p[1], p[2])]))
            .collect(),
        (Rgba, Eight) => data
            .chunks_exact(4)
            .map(|p| Luma([luma8_from_rgb8(p[0], p[1], p[2])]))
            .collect(),
        (Grayscale, Sixteen) => data
            .chunks_exact(2)
            .map(|p| Luma([u16_to_u8(be16(p))]))
            .collect(),
        (GrayscaleAlpha, Sixteen) => data
            .chunks_exact(4)
            .map(|p| Luma([u16_to_u8(be16(&p[0..2]))]))
            .collect(),
        (Rgb, Sixteen) => data
            .chunks_exact(6)
            .map(|p| {
                Luma([u16_to_u8(luma16_from_rgb16(
                    be16(&p[0..2]),
                    be16(&p[2..4]),
                    be16(&p[4..6]),
                ))])
            })
            .collect(),
        (Rgba, Sixteen) => data
            .chunks_exact(8)
            .map(|p| {
                Luma([u16_to_u8(luma16_from_rgb16(
                    be16(&p[0..2]),
                    be16(&p[2..4]),
                    be16(&p[4..6]),
                ))])
            })
            .collect(),
        other => panic!("include_gif: unsupported PNG format {other:?} in {path}"),
    };

    let expected = (info.width as usize) * (info.height as usize);
    assert_eq!(
        pixels.len(),
        expected,
        "include_gif: decoded {} pixels but expected {}x{}={} in {path}",
        pixels.len(),
        info.width,
        info.height,
        expected
    );

    GrayImage {
        width: info.width,
        height: info.height,
        pixels,
    }
}

fn decode_gif(path: &str) -> GrayImage {
    let file = std::fs::File::open(path)
        .unwrap_or_else(|e| panic!("include_gif: cannot open {path}: {e}"));
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = options
        .read_info(file)
        .unwrap_or_else(|e| panic!("include_gif: invalid GIF {path}: {e}"));

    // Like `image`, composite the first frame onto a fully transparent
    // (luma 0) canvas of the logical screen size, honouring its offset.
    let canvas_w = reader.width() as u32;
    let canvas_h = reader.height() as u32;
    let mut pixels = vec![Luma([0u8]); (canvas_w * canvas_h) as usize];

    if let Some(frame) = reader
        .read_next_frame()
        .unwrap_or_else(|e| panic!("include_gif: cannot decode GIF {path}: {e}"))
    {
        let fw = frame.width as u32;
        let fh = frame.height as u32;
        let left = frame.left as u32;
        let top = frame.top as u32;
        assert!(
            left + fw <= canvas_w && top + fh <= canvas_h,
            "include_gif: frame {fw}x{fh} at ({left},{top}) exceeds {canvas_w}x{canvas_h} canvas in {path}"
        );
        for fy in 0..fh {
            for fx in 0..fw {
                let idx = ((fy * fw + fx) * 4) as usize;
                let luma = luma8_from_rgb8(
                    frame.buffer[idx],
                    frame.buffer[idx + 1],
                    frame.buffer[idx + 2],
                );
                let cx = left + fx;
                let cy = top + fy;
                pixels[(cy * canvas_w + cx) as usize] = Luma([luma]);
            }
        }
    }

    GrayImage {
        width: canvas_w,
        height: canvas_h,
        pixels,
    }
}

fn generate_glyph(filename: LitStr, glyph_type: GlyphType) -> TokenStream {
    let path = format!(
        "{}/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        filename.value()
    );
    let mut grayscale_image: GrayImage = open_as_luma(&path);
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
    let mut current_byte = 0u16;
    let mut current_bit = 0u16;
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
                packed.push(current_byte as u8);
                current_byte = 0;
                current_bit = 0;
            }
        }
    }
    if current_bit > 0 {
        packed.push(current_byte as u8);
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

#[cfg(test)]
mod tests {
    //! Snapshot test pinning the generated glyph bytes against the values
    //! produced by the previous `image`-crate-based implementation. The golden
    //! bytes were captured from that implementation for the fixtures under
    //! `testdata/`. Run with:
    //!   cargo test -p include_gif --target <host-triple>
    use super::*;

    fn fixture(name: &str) -> GrayImage {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/");
        open_as_luma(&format!("{path}{name}"))
    }

    fn bagl(name: &str) -> Vec<u8> {
        generate_bagl_glyph(&fixture(name))
    }

    fn nbgl(name: &str) -> (Vec<u8>, u8, bool) {
        generate_nbgl_glyph(&mut fixture(name))
    }

    #[test]
    fn crab_48x48_png() {
        let img = fixture("crab_48x48.png");
        assert_eq!((img.width(), img.height()), (48, 48));
        assert_eq!(
            bagl("crab_48x48.png"),
            include!("../testdata/golden/crab_48x48.png.bagl")
        );
        let (buf, bpp, is_file) = nbgl("crab_48x48.png");
        assert_eq!((bpp, is_file), (1, true));
        assert_eq!(buf, include!("../testdata/golden/crab_48x48.png.nbgl"));
    }

    #[test]
    fn crab_14x14_png() {
        let img = fixture("crab_14x14.png");
        assert_eq!((img.width(), img.height()), (14, 14));
        assert_eq!(
            bagl("crab_14x14.png"),
            include!("../testdata/golden/crab_14x14.png.bagl")
        );
        let (buf, bpp, is_file) = nbgl("crab_14x14.png");
        assert_eq!((bpp, is_file), (1, false));
        assert_eq!(buf, include!("../testdata/golden/crab_14x14.png.nbgl"));
    }

    #[test]
    fn crab_64x64_gif() {
        let img = fixture("crab_64x64.gif");
        assert_eq!((img.width(), img.height()), (64, 64));
        assert_eq!(
            bagl("crab_64x64.gif"),
            include!("../testdata/golden/crab_64x64.gif.bagl")
        );
        let (buf, bpp, is_file) = nbgl("crab_64x64.gif");
        assert_eq!((bpp, is_file), (4, true));
        assert_eq!(buf, include!("../testdata/golden/crab_64x64.gif.nbgl"));
    }

    #[test]
    fn icon_cross_badge_gif() {
        assert_eq!(
            bagl("icon_cross_badge.gif"),
            include!("../testdata/golden/icon_cross_badge.gif.bagl")
        );
        let (buf, bpp, is_file) = nbgl("icon_cross_badge.gif");
        assert_eq!((bpp, is_file), (1, false));
        assert_eq!(
            buf,
            include!("../testdata/golden/icon_cross_badge.gif.nbgl")
        );
    }

    #[test]
    fn badge_check_gif() {
        assert_eq!(
            bagl("badge_check.gif"),
            include!("../testdata/golden/badge_check.gif.bagl")
        );
        let (buf, bpp, is_file) = nbgl("badge_check.gif");
        assert_eq!((bpp, is_file), (1, false));
        assert_eq!(buf, include!("../testdata/golden/badge_check.gif.nbgl"));
    }
}
