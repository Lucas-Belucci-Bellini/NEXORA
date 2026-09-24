//! Deflate decompression (RFC 1951): the half the runtime needs.
//!
//! Moved here from `tools/texture-forge` (ADR-0016). The forge writes PNGs and
//! the runtime reads them; one decoder serves both, because two decoders of
//! one format are two chances to disagree about what a file means. The
//! compressor stays in the forge: the runtime never writes a texture.
//!
//! # What it reads
//!
//! **Stored** and **fixed-Huffman** blocks — the two this project writes. A
//! dynamic-Huffman block is refused by name. That is a real limit on what the
//! runtime can open, and it is deliberate for now: every texture the runtime
//! loads is named in a resource manifest whose hash it has already matched
//! (ADR-0015), so what reaches this decoder was written by the forge.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// `(extra bits, base length)` for length codes 257..=285. RFC 1951 §3.2.5.
pub const LENGTH_CODES: [(u8, u16); 29] = [
    (0, 3),
    (0, 4),
    (0, 5),
    (0, 6),
    (0, 7),
    (0, 8),
    (0, 9),
    (0, 10),
    (1, 11),
    (1, 13),
    (1, 15),
    (1, 17),
    (2, 19),
    (2, 23),
    (2, 27),
    (2, 31),
    (3, 35),
    (3, 43),
    (3, 51),
    (3, 59),
    (4, 67),
    (4, 83),
    (4, 99),
    (4, 115),
    (5, 131),
    (5, 163),
    (5, 195),
    (5, 227),
    (0, 258),
];

/// `(extra bits, base distance)` for distance codes 0..=29. RFC 1951 §3.2.5.
pub const DISTANCE_CODES: [(u8, u16); 30] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 4),
    (1, 5),
    (1, 7),
    (2, 9),
    (2, 13),
    (3, 17),
    (3, 25),
    (4, 33),
    (4, 49),
    (5, 65),
    (5, 97),
    (6, 129),
    (6, 193),
    (7, 257),
    (7, 385),
    (8, 513),
    (8, 769),
    (9, 1025),
    (9, 1537),
    (10, 2049),
    (10, 3073),
    (11, 4097),
    (11, 6145),
    (12, 8193),
    (12, 12_289),
    (13, 16_385),
    (13, 24_577),
];

/// Largest output `inflate` will produce before refusing to continue.
///
/// `RESOURCE AND ASSET SYSTEM.md` lists **decompression ratios** among the
/// things to validate, and it is right to: a few hundred bytes of deflate can
/// name gigabytes of output. Sixty-four mebibytes is far past any texture this
/// project writes and far under anything that would hurt.
pub const MAX_INFLATED_BYTES: usize = 64 * 1024 * 1024;

/// Decompress a raw deflate stream.
///
/// Reads **stored** and **fixed-Huffman** blocks, which are the two this
/// project writes. A dynamic-Huffman block is refused by name rather than
/// treated as corruption, because the difference matters to whoever is
/// holding the file: one means "this came from somewhere else", the other
/// means "this is damaged".
///
/// # Errors
///
/// Returns an error when the stream ends early, names an unsupported or
/// reserved block type, carries a stored block whose length and complement
/// disagree, refers to a distance further back than the output so far, or
/// would expand past [`MAX_INFLATED_BYTES`].
pub fn inflate(data: &[u8]) -> Result<Vec<u8>> {
    inflate_bounded(data, MAX_INFLATED_BYTES)
}

/// Decompress a raw deflate stream, refusing to produce more than `limit`
/// bytes.
///
/// The bound a caller should pass is the size the container *declares*: a PNG
/// header states its dimensions, so the scanlines it can legitimately hold
/// are known before a single bit is inflated. That turns the "decompression
/// ratio" check of `RESOURCE AND ASSET SYSTEM.md` from a guess into an exact
/// limit — a stream that expands one byte past what its header promised stops
/// there, instead of at a global ceiling sixty-four mebibytes away.
///
/// # Errors
///
/// As [`inflate`], with `limit` in place of [`MAX_INFLATED_BYTES`]. A limit
/// above [`MAX_INFLATED_BYTES`] is clamped to it.
pub fn inflate_bounded(data: &[u8], limit: usize) -> Result<Vec<u8>> {
    let mut reader = BitReader {
        data,
        at: 0,
        bit: 0,
        limit: limit.min(MAX_INFLATED_BYTES),
    };
    let mut out = Vec::new();

    loop {
        let final_block = reader.bits(1)?;
        match reader.bits(2)? {
            0 => reader.stored_block(&mut out)?,
            1 => reader.fixed_block(&mut out)?,
            2 => {
                return Err(
                    damaged("dynamic Huffman blocks are not read by this decoder")
                        .with_context("offset", reader.at.to_string()),
                )
            }
            _ => {
                return Err(damaged("block type 3 is reserved and never valid")
                    .with_context("offset", reader.at.to_string()))
            }
        }
        if final_block == 1 {
            return Ok(out);
        }
    }
}

/// Reads bits in the two orders deflate uses.
struct BitReader<'a> {
    data: &'a [u8],
    at: usize,
    bit: u32,
    limit: usize,
}

impl BitReader<'_> {
    /// `count` bits, least-significant first.
    fn bits(&mut self, count: u32) -> Result<u32> {
        let mut value = 0u32;
        for index in 0..count {
            let byte = *self
                .data
                .get(self.at)
                .ok_or_else(|| damaged("the stream ends in the middle of a code"))?;
            value |= u32::from((byte >> self.bit) & 1) << index;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.at += 1;
            }
        }
        Ok(value)
    }

    /// Discard the rest of the current byte, as a stored block requires.
    fn align(&mut self) {
        if self.bit > 0 {
            self.bit = 0;
            self.at += 1;
        }
    }

    fn stored_block(&mut self, out: &mut Vec<u8>) -> Result<()> {
        self.align();
        let header = self
            .data
            .get(self.at..self.at + 4)
            .ok_or_else(|| damaged("a stored block's header is truncated"))?;
        let len = u16::from_le_bytes([header[0], header[1]]);
        let nlen = u16::from_le_bytes([header[2], header[3]]);
        if len != !nlen {
            return Err(
                damaged("a stored block's length and its complement disagree")
                    .with_context("len", len.to_string())
                    .with_context("nlen", nlen.to_string()),
            );
        }
        self.at += 4;
        let body = self
            .data
            .get(self.at..self.at + len as usize)
            .ok_or_else(|| damaged("a stored block is shorter than it claims"))?;
        grow(out, len as usize, self.limit)?;
        out.extend_from_slice(body);
        self.at += len as usize;
        Ok(())
    }

    fn fixed_block(&mut self, out: &mut Vec<u8>) -> Result<()> {
        loop {
            let symbol = self.fixed_symbol()?;
            if symbol == 256 {
                return Ok(());
            }
            if symbol < 256 {
                grow(out, 1, self.limit)?;
                out.push(symbol as u8);
                continue;
            }

            let index = usize::from(symbol - 257);
            let (extra, base) = *LENGTH_CODES
                .get(index)
                .ok_or_else(|| damaged("a length code outside the table"))?;
            let length = usize::from(base) + self.bits(u32::from(extra))? as usize;

            // Distance codes are five plain bits, most-significant first.
            let mut code = 0u32;
            for _ in 0..5 {
                code = (code << 1) | self.bits(1)?;
            }
            let (extra, base) = *DISTANCE_CODES
                .get(code as usize)
                .ok_or_else(|| damaged("a distance code outside the table"))?;
            let distance = usize::from(base) + self.bits(u32::from(extra))? as usize;

            if distance == 0 || distance > out.len() {
                return Err(damaged("a match reaches back further than the output")
                    .with_context("distance", distance.to_string())
                    .with_context("available", out.len().to_string()));
            }

            grow(out, length, self.limit)?;
            let start = out.len() - distance;
            for offset in 0..length {
                // Copied one byte at a time on purpose: a match may overlap
                // itself, which is how deflate encodes a run.
                out.push(out[start + offset]);
            }
        }
    }

    /// Decode one fixed literal/length symbol, most-significant bit first.
    fn fixed_symbol(&mut self) -> Result<u16> {
        let mut code = 0u32;
        for length in 1..=9u32 {
            code = (code << 1) | self.bits(1)?;
            match length {
                7 if code <= 0b001_0111 => return Ok((code + 256) as u16),
                8 if (0b0011_0000..=0b1011_1111).contains(&code) => {
                    return Ok((code - 0b0011_0000) as u16)
                }
                8 if (0b1100_0000..=0b1100_0111).contains(&code) => {
                    return Ok((code - 0b1100_0000 + 280) as u16)
                }
                9 if code >= 0b1_1001_0000 => return Ok((code - 0b1_1001_0000 + 144) as u16),
                _ => {}
            }
        }
        Err(damaged("no fixed Huffman code matches these bits"))
    }
}

/// Refuse before growing the output past the cap.
fn grow(out: &[u8], by: usize, limit: usize) -> Result<()> {
    if out.len() + by > limit {
        return Err(damaged("the stream expands past the decompression limit")
            .with_context("limit", limit.to_string()));
    }
    Ok(())
}

fn damaged(message: &'static str) -> Error {
    // Quarantine rather than reject: the file exists and may be recoverable by
    // hand, so it is set aside rather than declared meaningless.
    Error::new(Domain::Content, "deflate", message).with_recovery(Recovery::Quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Streams produced by zlib, not by the encoder above. Without these the
    // decoder and the encoder could share a misreading of RFC 1951 and every
    // round-trip test would still pass. zlib's `Z_FIXED` strategy was used so
    // the blocks are ones this decoder claims to read.
    const ZLIB_EMPTY: &[u8] = &[3, 0];
    const ZLIB_SHORT: &[u8] = &[203, 72, 205, 201, 201, 7, 0];
    const ZLIB_RUNS: &[u8] = &[75, 76, 74, 164, 42, 4, 0];
    const ZLIB_LONG_RUN: &[u8] = &[115, 114, 26, 5, 163, 96, 104, 2, 0];
    const ZLIB_HIGH_BYTES: &[u8] = &[251, 255, 239, 239, 127, 218, 32, 0];
    const ZLIB_STORED: &[u8] = &[
        1, 31, 0, 224, 255, 115, 116, 111, 114, 101, 100, 32, 98, 108, 111, 99, 107, 115, 32, 97,
        114, 101, 32, 108, 101, 103, 97, 108, 32, 100, 101, 102, 108, 97, 116, 101,
    ];

    #[test]
    fn streams_written_by_zlib_inflate_to_what_zlib_was_given() {
        assert_eq!(inflate(ZLIB_EMPTY).unwrap(), b"");
        assert_eq!(inflate(ZLIB_SHORT).unwrap(), b"hello");
        assert_eq!(inflate(ZLIB_RUNS).unwrap(), b"ab".repeat(40));
        assert_eq!(inflate(ZLIB_LONG_RUN).unwrap(), vec![0x42u8; 700]);
        assert_eq!(
            inflate(ZLIB_HIGH_BYTES).unwrap(),
            [0xFFu8, 0xFE, 0xFD].repeat(30)
        );
        assert_eq!(
            inflate(ZLIB_STORED).unwrap(),
            b"stored blocks are legal deflate"
        );
    }

    #[test]
    fn a_damaged_or_unsupported_stream_is_refused_rather_than_guessed_at() {
        // Truncated mid-code.
        assert!(inflate(&ZLIB_LONG_RUN[..4]).is_err());
        // Empty: there is not even a block header.
        assert!(inflate(&[]).is_err());
        // BTYPE 10 is dynamic Huffman: unsupported, and it says so by name.
        let err = inflate(&[0b101]).expect_err("dynamic Huffman is not read");
        assert!(err.to_string().contains("dynamic"), "{err}");
        // BTYPE 11 is reserved.
        let err = inflate(&[0b111]).expect_err("block type 3 is never valid");
        assert!(err.to_string().contains("reserved"), "{err}");
        // A stored block whose complement is wrong.
        let err = inflate(&[0x01, 0x05, 0x00, 0x00, 0x00, 1, 2, 3, 4, 5])
            .expect_err("LEN and NLEN must agree");
        assert!(err.to_string().contains("complement"), "{err}");
        // Every refusal quarantines rather than rejects: the bytes exist.
        assert_eq!(
            inflate(&[0b101]).unwrap_err().recovery(),
            Recovery::Quarantine
        );
    }

    #[test]
    fn a_stream_is_stopped_at_its_declared_size_not_at_the_global_ceiling() {
        // 700 bytes of 0x42, from zlib.
        assert_eq!(inflate_bounded(ZLIB_LONG_RUN, 700).unwrap().len(), 700);
        let err = inflate_bounded(ZLIB_LONG_RUN, 699).expect_err("one byte past what was declared");
        assert!(err.to_string().contains("decompression limit"), "{err}");
        assert!(err.to_string().contains("699"), "{err}");
        assert_eq!(err.recovery(), Recovery::Quarantine);
    }
}
