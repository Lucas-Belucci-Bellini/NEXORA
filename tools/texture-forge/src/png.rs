//! A PNG encoder, written rather than depended on.
//!
//! ADR-0002 forbids external crates in this workspace, and the audit found
//! that the format needs almost nothing this project does not already have:
//! every PNG chunk is checksummed with **CRC-32/ISO-HDLC**, which
//! `nexora_foundation::hashing::crc32` already implements and already tests
//! against the published `0xCBF43926` vector. What was missing was Adler-32,
//! which is twelve lines.
//!
//! # Compression, and why it arrived second
//!
//! The first version wrote **stored** deflate blocks: legal, read by every
//! decoder, and compressing nothing. That was deliberate — correctness first,
//! then a measurement to decide whether a compressor was worth writing. The
//! measurement said 12.11x, so [`crate::deflate`] exists and this module now
//! uses it. Nothing that calls the encoder changed, which was the point of
//! putting the seam here.
//!
//! # Filtering pays only in front of a compressor
//!
//! PNG lets each scanline choose one of five filters, and the choice is made
//! here by the standard minimum-sum-of-absolute-differences heuristic. That is
//! worth doing now and was not worth doing before: measured against stored
//! blocks, filtering changed nothing at all; measured against deflate, it took
//! a 128-square height map from 6,405 bytes to 4,228.

use nexora_asset::texture::{ChannelLayout, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::crc32;

/// The eight bytes that begin every PNG file.
pub const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// zlib header: deflate, 32 KiB window, no preset dictionary.
///
/// `0x78 0x01`. The two bytes together must be divisible by 31, which is what
/// makes `0x01` the only legal second byte for this first one.
const ZLIB_HEADER: [u8; 2] = [0x78, 0x01];

/// PNG's name for a channel layout.
///
/// Not every layout this project can hold is a PNG colour type by accident —
/// these four are exactly PNG's four non-indexed types.
const fn color_type(layout: ChannelLayout) -> u8 {
    match layout {
        ChannelLayout::Grey => 0,
        ChannelLayout::Rgb => 2,
        ChannelLayout::GreyAlpha => 4,
        ChannelLayout::Rgba => 6,
    }
}

/// Encode a texture map as a PNG file.
///
/// # Errors
///
/// Returns an error when the map's bit depth is not eight. PNG defines other
/// depths and this encoder does not write them; refusing is better than
/// writing a file whose header claims something the bytes do not say.
pub fn encode(map: &TextureMap) -> Result<Vec<u8>> {
    let format = map.format();
    if format.bits_per_channel != 8 {
        return Err(unsupported("this encoder writes eight bits per channel")
            .with_context("bits", format.bits_per_channel.to_string())
            .with_context("map", map.role().as_str()));
    }
    let resolution = map.resolution();
    encode_raw(
        resolution.width,
        resolution.height,
        format.channels,
        map.pixels(),
    )
}

/// Encode raw eight-bit pixels as a PNG file.
///
/// # Errors
///
/// Returns an error when the buffer length does not match the dimensions and
/// layout exactly.
pub fn encode_raw(
    width: u32,
    height: u32,
    layout: ChannelLayout,
    pixels: &[u8],
) -> Result<Vec<u8>> {
    let stride = width as usize * layout.count() as usize;
    let expected = stride * height as usize;
    if pixels.len() != expected {
        return Err(unsupported("pixel buffer does not match the dimensions")
            .with_context("expected", expected.to_string())
            .with_context("found", pixels.len().to_string()));
    }
    if width == 0 || height == 0 {
        return Err(unsupported("a PNG needs a non-zero extent"));
    }

    let raw = filter_scanlines(pixels, stride, layout.count() as usize);

    let mut out = Vec::with_capacity(raw.len() + 1024);
    out.extend_from_slice(&SIGNATURE);

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.push(8);
    header.push(color_type(layout));
    header.push(0); // compression: deflate, the only value PNG defines
    header.push(0); // filter method: adaptive, the only value PNG defines
    header.push(0); // interlace: none
    write_chunk(&mut out, b"IHDR", &header);

    write_chunk(&mut out, b"IDAT", &zlib(&raw));
    write_chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// Wrap a deflate stream in the zlib framing `IDAT` requires.
fn zlib(data: &[u8]) -> Vec<u8> {
    let compressed = crate::deflate::deflate(data);
    let mut out = Vec::with_capacity(compressed.len() + 6);
    out.extend_from_slice(&ZLIB_HEADER);
    out.extend_from_slice(&compressed);
    // The trailing checksum covers the *uncompressed* data, which is what
    // makes it a check on the whole round trip rather than on the framing.
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// Choose a filter for each scanline and apply it.
///
/// PNG's filters are transforms on the *bytes* of a row, taken relative to the
/// byte one pixel to the left (`a`), the byte above (`b`) and the byte above
/// and to the left (`c`). The heuristic is the one the specification suggests:
/// pick the filter whose output has the smallest sum of absolute signed
/// values, on the theory that small residuals compress well.
fn filter_scanlines(pixels: &[u8], stride: usize, bytes_per_pixel: usize) -> Vec<u8> {
    let rows = pixels.len() / stride;
    let mut out = Vec::with_capacity(pixels.len() + rows);
    let mut previous = vec![0u8; stride];
    let mut candidate = vec![0u8; stride];
    let mut best = vec![0u8; stride];

    for row in 0..rows {
        let line = &pixels[row * stride..(row + 1) * stride];
        let mut best_kind = 0u8;
        let mut best_score = u64::MAX;

        for kind in 0..5u8 {
            let mut score = 0u64;
            for index in 0..stride {
                let a = if index >= bytes_per_pixel {
                    line[index - bytes_per_pixel]
                } else {
                    0
                };
                let b = previous[index];
                let c = if index >= bytes_per_pixel {
                    previous[index - bytes_per_pixel]
                } else {
                    0
                };
                let predicted = match kind {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                    _ => paeth(a, b, c),
                };
                let value = line[index].wrapping_sub(predicted);
                candidate[index] = value;
                // Treat the byte as signed: a residual of 255 is -1, which is
                // as cheap as +1 and must not score as expensive.
                score += u64::from(value.min(value.wrapping_neg()));
            }
            if score < best_score {
                best_score = score;
                best_kind = kind;
                best.copy_from_slice(&candidate);
            }
        }

        out.push(best_kind);
        out.extend_from_slice(&best);
        previous.copy_from_slice(line);
    }
    out
}

/// PNG's Paeth predictor: whichever of the three neighbours is closest to
/// their linear estimate.
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let estimate = i32::from(a) + i32::from(b) - i32::from(c);
    let (pa, pb, pc) = (
        (estimate - i32::from(a)).abs(),
        (estimate - i32::from(b)).abs(),
        (estimate - i32::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Length, type, data, CRC — the shape of every PNG chunk.
fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    // The CRC covers the type and the data, and not the length.
    let checksum = crc32(&out[start..]);
    out.extend_from_slice(&checksum.to_be_bytes());
}

/// Adler-32, as zlib defines it.
///
/// Two running sums modulo 65521, the largest prime below 2^16. The only
/// checksum in this file that `nexora_foundation` did not already provide.
#[must_use]
pub fn adler32(data: &[u8]) -> u32 {
    const MODULUS: u32 = 65_521;
    let (mut low, mut high) = (1u32, 0u32);
    for &byte in data {
        low = (low + u32::from(byte)) % MODULUS;
        high = (high + low) % MODULUS;
    }
    (high << 16) | low
}

fn unsupported(message: &'static str) -> Error {
    Error::new(Domain::Content, "png", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deflate::reference::Inflater;
    use nexora_asset::texture::{MapRole, Resolution, TextureFormat};

    /// Undo PNG's per-scanline filtering, so a test can compare pixels.
    fn unfilter(raw: &[u8], width: usize, channels: usize) -> Vec<u8> {
        let stride = width * channels;
        let mut out: Vec<u8> = Vec::with_capacity(raw.len());
        for row in 0..raw.len() / (stride + 1) {
            let kind = raw[row * (stride + 1)];
            let line = &raw[row * (stride + 1) + 1..(row + 1) * (stride + 1)];
            for index in 0..stride {
                let a = if index >= channels {
                    out[out.len() - channels]
                } else {
                    0
                };
                let b = if row > 0 {
                    out[(row - 1) * stride + index]
                } else {
                    0
                };
                let c = if row > 0 && index >= channels {
                    out[(row - 1) * stride + index - channels]
                } else {
                    0
                };
                let predicted = match kind {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                    4 => paeth(a, b, c),
                    other => panic!("unknown filter {other}"),
                };
                out.push(line[index].wrapping_add(predicted));
            }
        }
        out
    }

    fn map(layout: ChannelLayout, edge: u32) -> TextureMap {
        let format = TextureFormat::eight_bit(layout);
        let resolution = Resolution::square(edge).unwrap();
        let bytes = (resolution.texels() * u64::from(layout.count())) as usize;
        // A gradient rather than a fill: a flat image would pass an encoder
        // that got the stride wrong.
        let pixels = (0..bytes).map(|index| (index % 251) as u8).collect();
        TextureMap::new(MapRole::Albedo, format, resolution, pixels).expect("well-formed")
    }

    /// Read the chunks back out of an encoded file.
    fn chunks(png: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut at = SIGNATURE.len();
        let mut found = Vec::new();
        while at + 8 <= png.len() {
            let len = u32::from_be_bytes(png[at..at + 4].try_into().unwrap()) as usize;
            let kind = String::from_utf8(png[at + 4..at + 8].to_vec()).unwrap();
            let data = png[at + 8..at + 8 + len].to_vec();
            let stated = u32::from_be_bytes(png[at + 8 + len..at + 12 + len].try_into().unwrap());
            assert_eq!(
                stated,
                crc32(&png[at + 4..at + 8 + len]),
                "chunk {kind} carries a wrong CRC"
            );
            found.push((kind, data));
            at += 12 + len;
        }
        assert_eq!(at, png.len(), "trailing bytes after the last chunk");
        found
    }

    #[test]
    fn adler32_matches_the_published_vectors() {
        // Taken from zlib's own `adler32`, not from memory. Two of the five
        // were wrong when written from recall, and the implementation was
        // right -- which is the whole argument for anchoring a checksum test
        // in a reference rather than in confidence.
        assert_eq!(adler32(b""), 0x0000_0001);
        assert_eq!(adler32(b"a"), 0x0062_0062);
        assert_eq!(adler32(b"abc"), 0x024D_0127);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        // Long enough that both sums wrap the modulus many times over.
        assert_eq!(adler32(&vec![0xFF; 10_000]), 0xB623_EB2B);
    }

    #[test]
    fn a_file_begins_with_the_signature_and_the_three_required_chunks() {
        let png = encode(&map(ChannelLayout::Rgba, 8)).unwrap();
        assert_eq!(&png[..8], &SIGNATURE);

        let found = chunks(&png);
        let kinds: Vec<&str> = found.iter().map(|(kind, _)| kind.as_str()).collect();
        assert_eq!(kinds, ["IHDR", "IDAT", "IEND"]);

        let header = &found[0].1;
        assert_eq!(header.len(), 13);
        assert_eq!(u32::from_be_bytes(header[0..4].try_into().unwrap()), 8);
        assert_eq!(u32::from_be_bytes(header[4..8].try_into().unwrap()), 8);
        assert_eq!(header[8], 8, "bit depth");
        assert_eq!(header[9], 6, "colour type RGBA");
        assert_eq!(&header[10..13], &[0, 0, 0]);
        assert!(found[2].1.is_empty(), "IEND carries no data");
    }

    #[test]
    fn every_layout_maps_to_its_png_colour_type() {
        for (layout, expected) in [
            (ChannelLayout::Grey, 0u8),
            (ChannelLayout::Rgb, 2),
            (ChannelLayout::GreyAlpha, 4),
            (ChannelLayout::Rgba, 6),
        ] {
            let png = encode_raw(4, 4, layout, &vec![7; 16 * layout.count() as usize]).unwrap();
            assert_eq!(chunks(&png)[0].1[9], expected, "{layout:?}");
        }
    }

    #[test]
    fn the_zlib_stream_is_well_formed_and_inflates_to_the_scanlines() {
        let source = map(ChannelLayout::Grey, 8);
        let png = encode(&source).unwrap();
        let idat = &chunks(&png)[1].1;

        assert_eq!(&idat[..2], &ZLIB_HEADER);
        // The two header bytes together must be divisible by 31.
        assert_eq!((u32::from(idat[0]) * 256 + u32::from(idat[1])) % 31, 0);

        // Inflated by the reference decoder, which was written from RFC 1951
        // separately from the encoder it is checking.
        let raw = Inflater::new(&idat[2..idat.len() - 4]).inflate();
        assert_eq!(
            u32::from_be_bytes(idat[idat.len() - 4..].try_into().unwrap()),
            adler32(&raw),
            "the trailing Adler-32 must cover the uncompressed data"
        );

        // Nine bytes per row: one filter byte plus eight grey samples.
        assert_eq!(raw.len(), 8 * 9);
        assert_eq!(
            unfilter(&raw, 8, 1),
            source.pixels(),
            "the scanlines must reconstruct"
        );
    }

    #[test]
    fn every_scanline_reconstructs_through_whichever_filter_was_chosen() {
        // Each layout exercises a different `bytes_per_pixel`, which is the
        // parameter every filter but `Up` depends on. A filter chosen with the
        // wrong stride reconstructs to garbage rather than to an error.
        for layout in [
            ChannelLayout::Grey,
            ChannelLayout::GreyAlpha,
            ChannelLayout::Rgb,
            ChannelLayout::Rgba,
        ] {
            let source = map(layout, 32);
            let png = encode(&source).unwrap();
            let idat = &chunks(&png)[1].1;
            let raw = Inflater::new(&idat[2..idat.len() - 4]).inflate();
            assert_eq!(
                unfilter(&raw, 32, layout.count() as usize),
                source.pixels(),
                "{layout:?} did not reconstruct"
            );
        }
    }

    #[test]
    fn compression_actually_compresses_a_texture_shaped_image() {
        // Not a tautology: the first version of this encoder wrote stored
        // blocks, and this assertion would have failed on it. Bands of one
        // colour are what a palette-quantised albedo looks like.
        let edge = 64u32;
        let pixels: Vec<u8> = (0..edge)
            .flat_map(|y| (0..edge).map(move |x| (x, y)))
            .flat_map(|(x, _)| {
                let band = ((x / 4) * 16) as u8;
                [band, band / 2, band / 3, 255]
            })
            .collect();
        let png = encode_raw(edge, edge, ChannelLayout::Rgba, &pixels).unwrap();
        assert!(
            png.len() * 8 < pixels.len(),
            "{} raw bytes became {} of PNG, under 8x",
            pixels.len(),
            png.len()
        );
    }

    #[test]
    fn a_bit_depth_this_encoder_cannot_write_is_refused() {
        let format = TextureFormat::sixteen_bit(ChannelLayout::Grey);
        let resolution = Resolution::square(8).unwrap();
        let map = TextureMap::new(MapRole::Height, format, resolution, vec![0; 8 * 8 * 2]).unwrap();

        let err = encode(&map).expect_err("sixteen bits is not written yet");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("eight bits"), "{err}");
    }

    #[test]
    fn a_mismatched_buffer_is_refused_before_anything_is_written() {
        assert!(encode_raw(4, 4, ChannelLayout::Rgba, &[0; 63]).is_err());
        assert!(encode_raw(4, 4, ChannelLayout::Rgba, &[0; 65]).is_err());
        assert!(encode_raw(0, 4, ChannelLayout::Grey, &[]).is_err());
        encode_raw(4, 4, ChannelLayout::Rgba, &[0; 64]).expect("the exact size is written");
    }

    #[test]
    fn encoding_is_deterministic() {
        // Two encodes of the same map must be byte-identical, or a material's
        // recorded hash stops meaning anything about the file on disk.
        let source = map(ChannelLayout::Rgba, 16);
        assert_eq!(encode(&source).unwrap(), encode(&source).unwrap());
    }
}
