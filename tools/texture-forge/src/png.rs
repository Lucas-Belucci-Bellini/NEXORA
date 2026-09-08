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

use nexora_asset::texture::{ChannelLayout, Resolution, TextureMap};
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

/// What a decoded PNG turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// The image's dimensions.
    pub resolution: Resolution,
    /// The channel layout its colour type names.
    pub layout: ChannelLayout,
    /// The unfiltered pixels, row-major from the top-left.
    pub pixels: Vec<u8>,
}

/// Decode a PNG this project could have written.
///
/// Exists so the validator can answer *"does this file decode?"* by decoding
/// it, rather than by checking that it looks plausible. Every failure names
/// what was wrong, because "invalid PNG" is a message that costs an hour.
///
/// # Errors
///
/// Returns an error for a wrong signature, a truncated or mis-checksummed
/// chunk, a colour type or bit depth this project does not write, an
/// interlaced image, a zlib stream that will not inflate, or scanlines that do
/// not add up to the dimensions the header declares.
pub fn decode(data: &[u8]) -> Result<Decoded> {
    if data.len() < SIGNATURE.len() || data[..SIGNATURE.len()] != SIGNATURE {
        return Err(malformed("the file does not begin with the PNG signature"));
    }

    let mut at = SIGNATURE.len();
    let mut header: Option<(u32, u32, u8, u8)> = None;
    let mut idat = Vec::new();
    let mut ended = false;

    while at < data.len() {
        let length = u32::from_be_bytes(
            data.get(at..at + 4)
                .ok_or_else(|| malformed("a chunk length is truncated"))?
                .try_into()
                .expect("four bytes"),
        ) as usize;
        let kind: [u8; 4] = data
            .get(at + 4..at + 8)
            .ok_or_else(|| malformed("a chunk type is truncated"))?
            .try_into()
            .expect("four bytes");
        let payload = data
            .get(at + 8..at + 8 + length)
            .ok_or_else(|| malformed("a chunk is shorter than its declared length"))?;
        let stated = u32::from_be_bytes(
            data.get(at + 8 + length..at + 12 + length)
                .ok_or_else(|| malformed("a chunk's checksum is truncated"))?
                .try_into()
                .expect("four bytes"),
        );
        let actual = crc32(&data[at + 4..at + 8 + length]);
        if stated != actual {
            return Err(malformed("a chunk's checksum does not match its contents")
                .with_context("chunk", String::from_utf8_lossy(&kind).into_owned())
                .with_context("stated", format!("{stated:#010x}"))
                .with_context("actual", format!("{actual:#010x}")));
        }

        match &kind {
            b"IHDR" => {
                if payload.len() != 13 {
                    return Err(malformed("IHDR is not thirteen bytes"));
                }
                let width = u32::from_be_bytes(payload[0..4].try_into().expect("four bytes"));
                let height = u32::from_be_bytes(payload[4..8].try_into().expect("four bytes"));
                if payload[10] != 0 || payload[11] != 0 {
                    return Err(malformed(
                        "IHDR names a compression or filter method PNG does not define",
                    ));
                }
                if payload[12] != 0 {
                    return Err(malformed("interlaced images are not read by this decoder"));
                }
                header = Some((width, height, payload[8], payload[9]));
            }
            b"IDAT" => idat.extend_from_slice(payload),
            b"IEND" => ended = true,
            // Ancillary chunks are legal and carry nothing this project needs.
            _ => {}
        }
        at += 12 + length;
    }

    if !ended {
        return Err(malformed("the file has no IEND chunk"));
    }
    let (width, height, depth, color) =
        header.ok_or_else(|| malformed("the file has no IHDR chunk"))?;
    if depth != 8 {
        return Err(malformed("this decoder reads eight bits per channel")
            .with_context("bits", depth.to_string()));
    }
    let layout = match color {
        0 => ChannelLayout::Grey,
        2 => ChannelLayout::Rgb,
        4 => ChannelLayout::GreyAlpha,
        6 => ChannelLayout::Rgba,
        other => {
            return Err(
                malformed("the colour type is one this project does not write")
                    .with_context("colour_type", other.to_string()),
            )
        }
    };
    let resolution = Resolution::new(width, height)?;

    if idat.len() < 6 {
        return Err(malformed("the zlib stream is too short to be one"));
    }
    if (u32::from(idat[0]) * 256 + u32::from(idat[1])) % 31 != 0 {
        return Err(malformed("the zlib header fails its own check value"));
    }
    let raw = crate::deflate::inflate(&idat[2..idat.len() - 4])?;
    let stated = u32::from_be_bytes(idat[idat.len() - 4..].try_into().expect("four bytes"));
    if stated != adler32(&raw) {
        return Err(
            malformed("the zlib checksum does not match the decompressed data")
                .with_context("stated", format!("{stated:#010x}")),
        );
    }

    let channels = layout.count() as usize;
    let stride = width as usize * channels;
    if raw.len() != (stride + 1) * height as usize {
        return Err(
            malformed("the scanlines do not add up to the declared size")
                .with_context("expected", ((stride + 1) * height as usize).to_string())
                .with_context("found", raw.len().to_string()),
        );
    }

    let mut pixels: Vec<u8> = Vec::with_capacity(stride * height as usize);
    for row in 0..height as usize {
        let kind = raw[row * (stride + 1)];
        let line = &raw[row * (stride + 1) + 1..(row + 1) * (stride + 1)];
        for index in 0..stride {
            let a = if index >= channels {
                pixels[pixels.len() - channels]
            } else {
                0
            };
            let b = if row > 0 {
                pixels[(row - 1) * stride + index]
            } else {
                0
            };
            let c = if row > 0 && index >= channels {
                pixels[(row - 1) * stride + index - channels]
            } else {
                0
            };
            let predicted = match kind {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                4 => paeth(a, b, c),
                other => {
                    return Err(malformed("a scanline names a filter PNG does not define")
                        .with_context("filter", other.to_string())
                        .with_context("row", row.to_string()))
                }
            };
            pixels.push(line[index].wrapping_add(predicted));
        }
    }

    Ok(Decoded {
        resolution,
        layout,
        pixels,
    })
}

fn malformed(message: &'static str) -> Error {
    // Quarantine, not reject: a damaged file is set aside for a person, not
    // declared meaningless.
    Error::new(Domain::Content, "png", message).with_recovery(Recovery::Quarantine)
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
    use crate::deflate::inflate;
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
        let raw = inflate(&idat[2..idat.len() - 4]).expect("the stream must inflate");
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
            let raw = inflate(&idat[2..idat.len() - 4]).expect("the stream must inflate");
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
    fn every_layout_survives_encode_then_decode() {
        for layout in [
            ChannelLayout::Grey,
            ChannelLayout::GreyAlpha,
            ChannelLayout::Rgb,
            ChannelLayout::Rgba,
        ] {
            let source = map(layout, 32);
            let decoded = decode(&encode(&source).unwrap()).expect("our own file must decode");
            assert_eq!(decoded.layout, layout);
            assert_eq!(decoded.resolution, source.resolution());
            assert_eq!(decoded.pixels, source.pixels(), "{layout:?} round trip");
        }
        // Non-square too, where a transposed stride would pass every square test.
        let png = encode_raw(64, 16, ChannelLayout::Rgb, &vec![9; 64 * 16 * 3]).unwrap();
        let decoded = decode(&png).unwrap();
        assert_eq!(decoded.resolution.width, 64);
        assert_eq!(decoded.resolution.height, 16);
    }

    #[test]
    fn a_corrupted_file_is_named_rather_than_guessed_at() {
        let good = encode(&map(ChannelLayout::Rgba, 16)).unwrap();

        // Wrong signature.
        let mut bad = good.clone();
        bad[1] = b'X';
        assert!(decode(&bad).unwrap_err().to_string().contains("signature"));

        // A flipped bit inside IDAT: the chunk CRC catches it first.
        let mut bad = good.clone();
        let middle = bad.len() / 2;
        bad[middle] ^= 0xFF;
        let err = decode(&bad).expect_err("a flipped bit must not decode silently");
        assert!(err.to_string().contains("checksum"), "{err}");
        assert_eq!(err.recovery(), Recovery::Quarantine);

        // Truncated: the last chunk is incomplete and IEND is gone.
        let err = decode(&good[..good.len() - 8]).expect_err("truncation must be caught");
        assert!(
            err.to_string().contains("truncated") || err.to_string().contains("IEND"),
            "{err}"
        );

        // Nothing at all.
        assert!(decode(&[]).is_err());
        assert!(decode(&SIGNATURE).is_err());
    }

    #[test]
    fn a_header_this_project_does_not_write_is_refused_by_name() {
        let good = encode(&map(ChannelLayout::Grey, 8)).unwrap();
        // IHDR's payload starts 8 (signature) + 8 (length and type) in.
        let ihdr = SIGNATURE.len() + 8;

        let rewrite = |offset: usize, value: u8| {
            let mut bad = good.clone();
            bad[ihdr + offset] = value;
            // Repair the chunk CRC so the header check is what fails, not the
            // checksum: otherwise this test proves nothing about IHDR.
            let crc = crc32(&bad[ihdr - 4..ihdr + 13]);
            bad[ihdr + 13..ihdr + 17].copy_from_slice(&crc.to_be_bytes());
            decode(&bad).expect_err("must be refused")
        };

        assert!(rewrite(8, 16).to_string().contains("eight bits"));
        assert!(rewrite(9, 3).to_string().contains("colour type"));
        assert!(rewrite(12, 1).to_string().contains("interlaced"));
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
