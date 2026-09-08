//! A PNG encoder, written rather than depended on.
//!
//! ADR-0002 forbids external crates in this workspace, and the audit found
//! that the format needs almost nothing this project does not already have:
//! every PNG chunk is checksummed with **CRC-32/ISO-HDLC**, which
//! `nexora_foundation::hashing::crc32` already implements and already tests
//! against the published `0xCBF43926` vector. What was missing was Adler-32,
//! which is twelve lines.
//!
//! # Stored deflate, on purpose and for now
//!
//! `IDAT` carries a zlib stream, and a zlib stream may legally consist of
//! **stored** deflate blocks — uncompressed, framed by a length and its
//! complement. Every decoder in existence reads them, because they are the
//! fallback every compressor emits for incompressible data.
//!
//! So the encoder is correct today and compresses nothing. That is a real
//! cost, it is measured rather than estimated (see the size report the tool
//! prints and DEBT-0031), and the fix — a fixed-Huffman deflate — is an
//! addition inside this module rather than a change to anything that calls it.
//! Writing a compressor before knowing what the files actually weigh would
//! have been guessing.

use nexora_asset::texture::{ChannelLayout, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::crc32;

/// The eight bytes that begin every PNG file.
pub const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Largest payload a single stored deflate block can carry.
const MAX_STORED_BLOCK: usize = 0xFFFF;

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

    // Scanlines, each prefixed with its filter byte. Filter 0 is "none":
    // filtering only pays for itself in front of a compressor, and there is
    // not one here yet. When there is, this is where the choice belongs.
    let mut raw = Vec::with_capacity(expected + height as usize);
    for row in pixels.chunks_exact(stride) {
        raw.push(0);
        raw.extend_from_slice(row);
    }

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

    write_chunk(&mut out, b"IDAT", &zlib_stored(&raw));
    write_chunk(&mut out, b"IEND", &[]);
    Ok(out)
}

/// Wrap bytes in a zlib stream made of stored deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / MAX_STORED_BLOCK * 5 + 16);
    out.extend_from_slice(&ZLIB_HEADER);

    if data.is_empty() {
        // A stream still needs one block, and it still needs to say it is last.
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    } else {
        let mut blocks = data.chunks(MAX_STORED_BLOCK).peekable();
        while let Some(block) = blocks.next() {
            let final_block = u8::from(blocks.peek().is_none());
            out.push(final_block); // BFINAL in bit 0, BTYPE 00 (stored)
            let len = block.len() as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(block);
        }
    }

    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
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
    use nexora_asset::texture::{MapRole, Resolution, TextureFormat};

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
    fn the_zlib_stream_is_well_formed_and_carries_the_scanlines() {
        let source = map(ChannelLayout::Grey, 8);
        let png = encode(&source).unwrap();
        let idat = &chunks(&png)[1].1;

        assert_eq!(&idat[..2], &ZLIB_HEADER);
        // The two header bytes together must be divisible by 31.
        assert_eq!((u32::from(idat[0]) * 256 + u32::from(idat[1])) % 31, 0);

        // Walk the stored blocks and rebuild the raw scanlines.
        let mut at = 2usize;
        let mut raw = Vec::new();
        loop {
            let header = idat[at];
            assert_eq!(header & 0b110, 0, "BTYPE must be stored");
            let len = u16::from_le_bytes(idat[at + 1..at + 3].try_into().unwrap());
            let nlen = u16::from_le_bytes(idat[at + 3..at + 5].try_into().unwrap());
            assert_eq!(len, !nlen, "LEN and NLEN must be complements");
            raw.extend_from_slice(&idat[at + 5..at + 5 + len as usize]);
            at += 5 + len as usize;
            if header & 1 == 1 {
                break;
            }
        }
        assert_eq!(
            u32::from_be_bytes(idat[at..at + 4].try_into().unwrap()),
            adler32(&raw),
            "the trailing Adler-32 must cover the uncompressed data"
        );
        assert_eq!(at + 4, idat.len(), "trailing bytes in the zlib stream");

        // Nine bytes per row: one filter byte plus eight grey samples.
        assert_eq!(raw.len(), 8 * 9);
        for (row, chunk) in raw.chunks_exact(9).enumerate() {
            assert_eq!(chunk[0], 0, "row {row} must use filter None");
            assert_eq!(&chunk[1..], &source.pixels()[row * 8..row * 8 + 8]);
        }
    }

    #[test]
    fn an_image_larger_than_one_stored_block_is_split_and_still_reassembles() {
        // 160x160 RGBA is 102 400 bytes of pixels plus 160 filter bytes: two
        // blocks. A single-block encoder passes every smaller test there is.
        let png = encode(&map(ChannelLayout::Rgba, 128)).unwrap();
        let idat = &chunks(&png)[1].1;

        let mut at = 2usize;
        let mut blocks = 0;
        let mut total = 0usize;
        loop {
            let header = idat[at];
            let len = u16::from_le_bytes(idat[at + 1..at + 3].try_into().unwrap()) as usize;
            blocks += 1;
            total += len;
            at += 5 + len;
            if header & 1 == 1 {
                break;
            }
        }
        assert!(blocks > 1, "expected more than one block, got {blocks}");
        assert_eq!(total, 128 * 128 * 4 + 128);
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
