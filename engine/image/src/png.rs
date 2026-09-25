//! PNG decoding: the half the runtime needs.
//!
//! Moved here from `tools/texture-forge::png` (ADR-0022), which keeps the
//! encoder and re-exports this. It reads what this project writes — eight
//! bits per channel, the four non-indexed colour types, no interlacing — and
//! refuses everything else by name.
//!
//! # Bounded before it is inflated
//!
//! A PNG header states its dimensions before its pixel data begins, so the
//! size the zlib stream may inflate to is known in advance and passed to
//! [`crate::inflate::inflate_bounded`]. A file whose stream expands past its
//! own header's promise stops at that byte.

use nexora_asset::texture::{ChannelLayout, Resolution};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::crc32;

use crate::inflate::inflate_bounded;

/// The eight bytes that begin every PNG file.
pub const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

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
/// The forge's validator uses it to answer *"does this file decode?"* by
/// decoding it; the runtime uses it to turn a verified resource into pixels.
/// Every failure names what was wrong, because "invalid PNG" is a message
/// that costs an hour.
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
    // The header has already said how many bytes the scanlines hold, so that
    // is the most the stream may inflate to -- not a global ceiling.
    let declared = (width as usize * layout.count() as usize + 1) * height as usize;
    let raw = inflate_bounded(&idat[2..idat.len() - 4], declared)?;
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

/// PNG's Paeth predictor: whichever of the three neighbours is closest to
/// their linear estimate. Public because the encoder in the forge filters
/// with the same predictor this decoder unfilters with.
pub fn paeth(a: u8, b: u8, c: u8) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG assembled by hand: stored deflate blocks, no encoder involved,
    /// so the decoder is not only ever checked against the tool that it
    /// shares a crate family with.
    pub(crate) fn hand_made(width: u32, height: u32, colour: u8, scanlines: &[u8]) -> Vec<u8> {
        fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            let mut body = kind.to_vec();
            body.extend_from_slice(data);
            out.extend_from_slice(&body);
            out.extend_from_slice(&crc32(&body).to_be_bytes());
        }
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.extend_from_slice(&[8, colour, 0, 0, 0]);

        let mut zlib = vec![0x78, 0x01, 0x01];
        let len = scanlines.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(scanlines);
        zlib.extend_from_slice(&adler32(scanlines).to_be_bytes());

        let mut out = SIGNATURE.to_vec();
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &zlib);
        chunk(&mut out, b"IEND", &[]);
        out
    }

    /// Four rows of four grey texels, each row prefixed by its filter byte.
    fn rows(filters: [u8; 4], body: [[u8; 4]; 4]) -> Vec<u8> {
        let mut out = Vec::new();
        for (filter, row) in filters.iter().zip(body) {
            out.push(*filter);
            out.extend_from_slice(&row);
        }
        out
    }

    #[test]
    fn a_hand_made_file_decodes_through_every_filter() {
        // None, Up, Sub, Average.
        let png = hand_made(
            4,
            4,
            0,
            &rows(
                [0, 2, 1, 3],
                [[10, 20, 30, 40], [1, 1, 1, 1], [5, 1, 1, 1], [0, 0, 0, 0]],
            ),
        );
        let decoded = decode(&png).expect("decodes");
        assert_eq!(decoded.resolution, Resolution::new(4, 4).unwrap());
        assert_eq!(decoded.layout, ChannelLayout::Grey);
        assert_eq!(
            decoded.pixels,
            [10, 20, 30, 40, 11, 21, 31, 41, 5, 6, 7, 8, 2, 4, 5, 6]
        );

        // Paeth, which picks the upper neighbour on these values.
        let png = hand_made(
            4,
            4,
            0,
            &rows(
                [0, 4, 0, 0],
                [[10, 20, 30, 40], [1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0]],
            ),
        );
        assert_eq!(&decode(&png).unwrap().pixels[4..8], &[11, 21, 31, 41]);
    }

    #[test]
    fn a_stream_that_outgrows_its_own_header_is_stopped() {
        // The header says 4x4 grey -- twenty bytes of scanlines -- and the
        // stream carries twenty-one.
        let mut scanlines = rows([0; 4], [[1; 4]; 4]);
        scanlines.push(99);
        let err = decode(&hand_made(4, 4, 0, &scanlines)).expect_err("one byte past the header");
        assert!(err.to_string().contains("decompression limit"), "{err}");
        assert!(err.to_string().contains("20"), "{err}");
    }

    #[test]
    fn what_this_project_does_not_write_is_refused_by_name() {
        let grey = rows([0; 4], [[1; 4]; 4]);
        assert!(decode(b"not a png")
            .unwrap_err()
            .to_string()
            .contains("signature"));
        let err = decode(&hand_made(4, 4, 3, &grey)).expect_err("indexed colour");
        assert!(err.to_string().contains("colour type"), "{err}");
        let err = decode(&hand_made(6, 4, 0, &[0u8; 28])).expect_err("six wide");
        assert!(err.to_string().contains("power of two"), "{err}");

        let mut flipped = hand_made(4, 4, 0, &grey);
        let middle = flipped.len() / 2;
        flipped[middle] ^= 0x40;
        assert_eq!(
            decode(&flipped).unwrap_err().recovery(),
            Recovery::Quarantine
        );
    }

    #[test]
    fn adler32_matches_the_published_vectors() {
        assert_eq!(adler32(b""), 0x0000_0001);
        assert_eq!(adler32(b"a"), 0x0062_0062);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }
}
