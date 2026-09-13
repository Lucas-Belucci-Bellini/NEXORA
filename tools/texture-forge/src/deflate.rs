//! A deflate compressor, because the measurement asked for one.
//!
//! The PNG encoder shipped writing **stored** blocks first — correct, and
//! compressing nothing. Then the six maps it produced were weighed:
//!
//! ```text
//! stored          123,805 bytes
//! real deflate     10,220 bytes      12.11x
//! ```
//!
//! At the scale this tool exists for — ten thousand materials, six maps each —
//! that is roughly a gigabyte against eighty-five megabytes. So this module
//! exists because of a number, not because compressors are interesting.
//!
//! # Fixed Huffman, and why that is enough here
//!
//! RFC 1951 defines a code table that both sides already know, so a fixed
//! block carries no tree. A dynamic block would beat it by some percent and
//! costs a canonical-code builder and a tree encoder. What this data actually
//! wants is the **match finder**: palette-quantised albedo is long runs of
//! identical texels, and a run is one length-distance pair however it is
//! coded.
//!
//! # Correctness is checked by somebody else's decoder
//!
//! Every table below is a place to be silently wrong. So the test suite
//! round-trips through an inflater written independently in this module's
//! tests, and the PNG files themselves are decompressed by Python's `zlib`
//! outside the test suite entirely. A compressor that only agrees with itself
//! has proved nothing.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Largest distance a match may reach back. RFC 1951 §3.2.5.
pub const WINDOW: usize = 32_768;

/// Longest match deflate can encode.
pub const MAX_MATCH: usize = 258;

/// Shortest match worth encoding: below this a literal is cheaper.
pub const MIN_MATCH: usize = 3;

/// How many candidates the match finder will consider at one position.
///
/// The chain is ordered nearest-first, and a nearer match encodes in fewer
/// bits, so the early candidates are the valuable ones.
///
/// Thirty-two used to be the value here, on the claim that the ratio stopped
/// improving past it. That was measured under **greedy** matching, and lazy
/// matching changed the answer: a deeper chain now finds the longer match one
/// byte along that greedy could not have used anyway. Re-measured over a full
/// PBR set (48 images, 995 264 bytes of filtered scanlines):
///
/// | chain | output | batch |
/// | ---: | ---: | ---: |
/// | 32 | 155 779 | 163 ms |
/// | 64 | 152 091 | 194 ms |
/// | 128 | 150 134 | 248 ms |
/// | **256** | **148 500** | **339 ms** |
/// | 1024 | 146 057 | 651 ms |
///
/// 256 is the knee: it takes 69% of what depth has left to give, and doubling
/// again buys 1.6% more for twice the time. Generation is offline and the
/// bytes are paid by everyone who clones the repository, so the trade leans
/// towards bytes - but not past the point where it stops paying.
const MAX_CHAIN: usize = 256;

/// Size of the hash table indexing three-byte sequences.
const HASH_SIZE: usize = 1 << 15;

/// `(extra bits, base length)` for length codes 257..=285. RFC 1951 §3.2.5.
const LENGTH_CODES: [(u8, u16); 29] = [
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
const DISTANCE_CODES: [(u8, u16); 30] = [
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

/// Packs bits the way deflate wants them.
///
/// Two orders live in one stream and mixing them up is the classic way to
/// produce something no decoder will read: **Huffman codes go
/// most-significant bit first**, everything else least-significant first.
struct BitWriter {
    out: Vec<u8>,
    accumulator: u32,
    bits: u32,
}

impl BitWriter {
    fn new(capacity: usize) -> Self {
        Self {
            out: Vec::with_capacity(capacity),
            accumulator: 0,
            bits: 0,
        }
    }

    /// Write `count` bits of `value`, least-significant bit first.
    fn bits(&mut self, value: u32, count: u32) {
        self.accumulator |= (value & ((1 << count) - 1)) << self.bits;
        self.bits += count;
        while self.bits >= 8 {
            self.out.push((self.accumulator & 0xFF) as u8);
            self.accumulator >>= 8;
            self.bits -= 8;
        }
    }

    /// Write a Huffman code, most-significant bit first.
    fn code(&mut self, code: u32, count: u32) {
        // Reversed, then written LSB-first, which is the same thing.
        let mut reversed = 0u32;
        for bit in 0..count {
            reversed |= ((code >> (count - 1 - bit)) & 1) << bit;
        }
        self.bits(reversed, count);
    }

    fn finish(mut self) -> Vec<u8> {
        if self.bits > 0 {
            self.out.push((self.accumulator & 0xFF) as u8);
        }
        self.out
    }
}

/// The fixed literal/length code for a symbol. RFC 1951 §3.2.6.
const fn fixed_literal(symbol: u16) -> (u32, u32) {
    match symbol {
        0..=143 => (0b0011_0000 + symbol as u32, 8),
        144..=255 => (0b1_1001_0000 + (symbol as u32 - 144), 9),
        256..=279 => (symbol as u32 - 256, 7),
        _ => (0b1100_0000 + (symbol as u32 - 280), 8),
    }
}

/// Which length code covers a match of this length, and the extra bits.
fn length_symbol(length: usize) -> (u16, u32, u32) {
    let length = length as u16;
    let mut index = LENGTH_CODES.len() - 1;
    while index > 0 && LENGTH_CODES[index].1 > length {
        index -= 1;
    }
    let (extra, base) = LENGTH_CODES[index];
    (
        257 + index as u16,
        u32::from(length - base),
        u32::from(extra),
    )
}

/// Which distance code covers this distance, and the extra bits.
fn distance_symbol(distance: usize) -> (u32, u32, u32) {
    let distance = distance as u16;
    let mut index = DISTANCE_CODES.len() - 1;
    while index > 0 && DISTANCE_CODES[index].1 > distance {
        index -= 1;
    }
    let (extra, base) = DISTANCE_CODES[index];
    (index as u32, u32::from(distance - base), u32::from(extra))
}

fn hash3(data: &[u8], at: usize) -> usize {
    let triple = u32::from(data[at]) << 16 | u32::from(data[at + 1]) << 8 | u32::from(data[at + 2]);
    // Knuth's multiplicative hash, folded into the table's index width.
    ((triple.wrapping_mul(2_654_435_761)) >> (32 - 15)) as usize % HASH_SIZE
}

/// A match: how far it runs, and how far back it reaches to find itself.
#[derive(Debug, Clone, Copy)]
struct Match {
    length: usize,
    distance: usize,
}

/// The chain of positions sharing a three-byte prefix.
///
/// `head[slot]` is the most recent position hashing to `slot`; `prev[position]`
/// is the one before it. Walking `prev` therefore visits candidates
/// nearest-first, and a nearer match encodes in fewer bits.
struct Chain {
    head: Vec<usize>,
    prev: Vec<usize>,
}

impl Chain {
    fn new(len: usize) -> Self {
        Self {
            head: vec![usize::MAX; HASH_SIZE],
            prev: vec![usize::MAX; len.max(1)],
        }
    }

    /// Index `at` so later positions can reach back to it.
    ///
    /// Every position a match covers has to enter the chain, not just the
    /// positions the encoder stops at: skip them and the next search has
    /// nothing to reach back to, and long runs stop compressing.
    fn insert(&mut self, data: &[u8], at: usize) {
        if at + MIN_MATCH > data.len() {
            return;
        }
        let slot = hash3(data, at);
        self.prev[at] = self.head[slot];
        self.head[slot] = at;
    }

    /// The longest match reaching back from `at`, if one is worth encoding.
    ///
    /// Does not index `at`: the search must not find the position it starts
    /// from, so insertion is the caller's, and happens after.
    fn longest(&self, data: &[u8], at: usize) -> Option<Match> {
        if at + MIN_MATCH > data.len() {
            return None;
        }
        let mut best: Option<Match> = None;
        let mut candidate = self.head[hash3(data, at)];
        let limit = at.saturating_sub(WINDOW);
        let ceiling = MAX_MATCH.min(data.len() - at);
        let mut examined = 0;

        while candidate != usize::MAX && candidate >= limit && examined < MAX_CHAIN {
            let mut length = 0usize;
            while length < ceiling && data[candidate + length] == data[at + length] {
                length += 1;
            }
            if length > best.map_or(0, |found: Match| found.length) {
                best = Some(Match {
                    length,
                    distance: at - candidate,
                });
                if length == ceiling {
                    break;
                }
            }
            candidate = self.prev[candidate];
            examined += 1;
        }

        best.filter(|found| found.length >= MIN_MATCH)
    }
}

fn emit_literal(writer: &mut BitWriter, byte: u8) {
    let (code, bits) = fixed_literal(u16::from(byte));
    writer.code(code, bits);
}

fn emit_match(writer: &mut BitWriter, found: Match) {
    let (symbol, extra, extra_bits) = length_symbol(found.length);
    let (code, length_bits) = fixed_literal(symbol);
    writer.code(code, length_bits);
    if extra_bits > 0 {
        writer.bits(extra, extra_bits);
    }
    let (distance_code, distance_extra, distance_bits) = distance_symbol(found.distance);
    writer.code(distance_code, 5);
    if distance_bits > 0 {
        writer.bits(distance_extra, distance_bits);
    }
}

/// Compress with one fixed-Huffman block.
///
/// Matching is **lazy**: a match found at one position is held rather than
/// emitted, and dropped in favour of a strictly longer one starting a byte
/// later. Greedy matching cannot do this - it commits to the first long match
/// it sees and skips past the better one that started one byte in. The cost is
/// the literal left behind; the gain is everything the longer match covers.
///
/// Both encodings are legal deflate and any decoder reads either. RFC 1951
/// describes the format, not the choice.
///
/// Returns the raw deflate stream, with no zlib wrapper.
#[must_use]
pub fn fixed_block(data: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter::new(data.len() / 2 + 64);
    writer.bits(1, 1); // BFINAL
    writer.bits(1, 2); // BTYPE = 01, fixed Huffman

    let mut chain = Chain::new(data.len());
    // A match found at the previous position and not yet committed.
    let mut held: Option<Match> = None;
    let mut at = 0usize;

    while at < data.len() {
        let here = chain.longest(data, at);
        chain.insert(data, at);

        let Some(previous) = held else {
            match here {
                Some(found) => held = Some(found),
                None => emit_literal(&mut writer, data[at]),
            }
            at += 1;
            continue;
        };

        let start = at - 1;
        if here.is_some_and(|found| found.length > previous.length) {
            // One byte along does better. Spend `start` as a literal and hold
            // the new one, which may itself lose to the position after it.
            emit_literal(&mut writer, data[start]);
            held = here;
            at += 1;
        } else {
            emit_match(&mut writer, previous);
            // `start` and `at` are indexed already; the rest of the span is not.
            for position in at + 1..start + previous.length {
                chain.insert(data, position);
            }
            held = None;
            at = start + previous.length;
        }
    }

    if let Some(previous) = held {
        // Unreachable while `MIN_MATCH` is 3: holding a match at `len - 1`
        // would require one to have been found there, and a match needs
        // `MIN_MATCH` bytes of room. Emitted anyway, so lowering the constant
        // cannot quietly truncate the stream.
        emit_match(&mut writer, previous);
    }

    let (code, bits) = fixed_literal(256); // end of block
    writer.code(code, bits);
    writer.finish()
}

/// Wrap data in stored deflate blocks: correct, and no smaller than the input.
#[must_use]
pub fn stored_blocks(data: &[u8]) -> Vec<u8> {
    const MAX_STORED: usize = 0xFFFF;
    let mut out = Vec::with_capacity(data.len() + data.len() / MAX_STORED * 5 + 8);
    if data.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
        return out;
    }
    let mut blocks = data.chunks(MAX_STORED).peekable();
    while let Some(block) = blocks.next() {
        out.push(u8::from(blocks.peek().is_none()));
        let len = block.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out
}

/// Compress, falling back to stored when compressing would not help.
///
/// Fixed Huffman spends nine bits on any byte above 143, so on data with no
/// structure it can come out larger than it went in. Trying both and keeping
/// the smaller costs one extra pass and removes the pathological case
/// entirely.
#[must_use]
pub fn deflate(data: &[u8]) -> Vec<u8> {
    let compressed = fixed_block(data);
    let stored = stored_blocks(data);
    if compressed.len() < stored.len() {
        compressed
    } else {
        stored
    }
}

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
    let mut reader = BitReader {
        data,
        at: 0,
        bit: 0,
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
        grow(out, len as usize)?;
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
                grow(out, 1)?;
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

            grow(out, length)?;
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
fn grow(out: &[u8], by: usize) -> Result<()> {
    if out.len() + by > MAX_INFLATED_BYTES {
        return Err(damaged("the stream expands past the decompression limit")
            .with_context("limit", MAX_INFLATED_BYTES.to_string()));
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

    fn round_trip(data: &[u8]) -> Vec<u8> {
        inflate(&deflate(data)).expect("our own output must inflate")
    }

    /// What the encoder used to do: take the first match and skip past it.
    ///
    /// Only the *decision* differs. The match finder is the same [`Chain`] the
    /// real encoder walks and the emitters are the same two functions, so a
    /// size difference between this and [`fixed_block`] is attributable to
    /// laziness and to nothing else.
    fn greedy_block(data: &[u8]) -> Vec<u8> {
        let mut writer = BitWriter::new(data.len() / 2 + 64);
        writer.bits(1, 1);
        writer.bits(1, 2);

        let mut chain = Chain::new(data.len());
        let mut at = 0usize;
        while at < data.len() {
            let here = chain.longest(data, at);
            chain.insert(data, at);
            match here {
                Some(found) => {
                    emit_match(&mut writer, found);
                    for position in at + 1..at + found.length {
                        chain.insert(data, position);
                    }
                    at += found.length;
                }
                None => {
                    emit_literal(&mut writer, data[at]);
                    at += 1;
                }
            }
        }

        let (code, bits) = fixed_literal(256);
        writer.code(code, bits);
        writer.finish()
    }

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
    fn a_match_pointing_before_the_start_of_the_output_is_refused() {
        // A fixed block whose first symbol is a length/distance pair: there is
        // nothing behind it to copy, and a decoder that trusted the distance
        // would index out of bounds.
        let mut writer = BitWriter::new(8);
        writer.bits(1, 1);
        writer.bits(1, 2);
        let (code, bits) = fixed_literal(257); // length 3
        writer.code(code, bits);
        writer.code(0, 5); // distance 1, with no output yet
        let stream = writer.finish();

        let err = inflate(&stream).expect_err("a match must have something to copy");
        assert!(err.to_string().contains("reaches back further"), "{err}");
    }

    #[test]
    fn every_shape_of_input_survives_the_round_trip() {
        let cases: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"a".to_vec(),
            b"aaaaaaaaaa".to_vec(),
            b"abcabcabcabcabcabc".to_vec(),
            b"the quick brown fox jumps over the lazy dog".repeat(20),
            vec![0u8; 100_000],
            (0..=255u8).cycle().take(50_000).collect(),
            // High bytes only: the case where fixed Huffman spends nine bits.
            vec![0xFFu8; 3],
        ];
        for case in cases {
            assert_eq!(
                round_trip(&case),
                case,
                "round trip failed on {} bytes",
                case.len()
            );
        }
    }

    #[test]
    fn deferring_a_match_by_one_byte_beats_taking_the_first_one() {
        // The shape lazy matching exists for. Each instance plants two things
        // the finder can reach back to:
        //
        //   plant  = anchor ++ [one byte that is not the run's first]
        //   plant  = anchor[1..] ++ run
        //   probe  = anchor ++ run
        //
        // At the probe's first byte only the three-byte anchor matches; one
        // byte later the whole `anchor[1..] ++ run` does. Greedy commits to
        // the three and steps over the start of the long one, then picks up
        // the remainder as a second match. Lazy spends one literal and takes
        // the long match whole.
        //
        // Arithmetic for one instance, in fixed-Huffman bits: greedy pays a
        // length-3 code and a distance (~16) plus a length-24 code and a
        // second distance (~33); lazy pays one 8-bit literal and a single
        // length-26 code and distance. That is around eight bits saved, which
        // rounds away in one instance - so there are a hundred, with unrelated
        // contents so they cannot match each other. Expect ~100 bytes; the
        // assertion asks for half of that, which is a floor and not a guess.
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 33) as u8
        };

        let mut data = Vec::new();
        for _ in 0..100 {
            let anchor: Vec<u8> = (0..3).map(|_| next()).collect();
            let run: Vec<u8> = (0..24).map(|_| next()).collect();
            let separator: Vec<u8> = (0..8).map(|_| next()).collect();

            data.extend_from_slice(&anchor);
            data.push(run[0].wrapping_add(1)); // the anchor, not followed by the run
            data.extend_from_slice(&separator);
            data.extend_from_slice(&anchor[1..]);
            data.extend_from_slice(&run);
            data.extend_from_slice(&separator);
            data.extend_from_slice(&anchor);
            data.extend_from_slice(&run); // the probe: both matches available
            data.extend_from_slice(&separator);
        }

        let lazy = fixed_block(&data);
        let greedy = greedy_block(&data);

        assert!(
            greedy.len() >= lazy.len() + 50,
            "lazy {} against greedy {}: the deferral is not paying",
            lazy.len(),
            greedy.len()
        );
        assert_eq!(inflate(&lazy).unwrap(), data);
        assert_eq!(
            inflate(&greedy).unwrap(),
            data,
            "the oracle must be legal too"
        );
    }

    #[test]
    fn a_long_run_becomes_a_handful_of_bytes() {
        // The case the measurement was about: palette-quantised albedo is runs.
        let data = vec![0x42u8; 100_000];
        let compressed = deflate(&data);

        // The bound is derived, not guessed. Fixed Huffman codes the longest
        // match (258) as symbol 285 in seven bits with no extra, plus a
        // five-bit distance code for distance 1: twelve bits per match. So
        // ceil(99_999 / 258) = 388 matches cost 4_656 bits, plus one literal,
        // the end-of-block symbol and the three-bit header: about 585 bytes.
        // Anything near that is optimal for this block type; 700 leaves room
        // for the first few positions, where the chain has nothing to find.
        assert!(
            compressed.len() < 700,
            "100 KB of one byte compressed to {} bytes, past the 585 the coding allows",
            compressed.len()
        );
        assert_eq!(inflate(&compressed).unwrap(), data);
    }

    #[test]
    fn incompressible_data_falls_back_to_stored_rather_than_growing() {
        // A counter-based sequence with no repeats inside the window.
        let data: Vec<u8> = (0..60_000u32)
            .map(|index| (index.wrapping_mul(2_654_435_761) >> 24) as u8)
            .collect();
        let compressed = deflate(&data);
        let stored = stored_blocks(&data);
        assert!(
            compressed.len() <= stored.len(),
            "compressed {} exceeds stored {}",
            compressed.len(),
            stored.len()
        );
        assert_eq!(inflate(&compressed).unwrap(), data);
    }

    #[test]
    fn matches_at_the_edges_of_every_table_entry_round_trip() {
        // One case per length code and per distance code, which is where an
        // off-by-one in either table hides.
        for (extra, base) in LENGTH_CODES {
            for length in [usize::from(base), usize::from(base) + usize::from(extra)] {
                let length = length.min(MAX_MATCH);
                let mut data = vec![0xA5u8; length];
                data.extend(std::iter::repeat_n(0xA5u8, length));
                assert_eq!(round_trip(&data), data, "length {length}");
            }
        }
        for (_, base) in DISTANCE_CODES {
            let distance = usize::from(base);
            let mut data: Vec<u8> = (0..distance).map(|index| (index % 251) as u8).collect();
            let tail: Vec<u8> = data[..MIN_MATCH.max(4).min(distance)].to_vec();
            data.extend_from_slice(&tail);
            assert_eq!(round_trip(&data), data, "distance {distance}");
        }
    }

    #[test]
    fn a_match_that_overlaps_itself_round_trips() {
        // Distance 1, length 200: legal in deflate, and the case a naive
        // copy-then-append gets wrong.
        let mut data = b"x".to_vec();
        data.extend(std::iter::repeat_n(b'x', 200));
        assert_eq!(round_trip(&data), data);

        // Distance 3, length 20: overlapping by a period.
        let mut data = b"abc".to_vec();
        for _ in 0..20 {
            let next = data[data.len() - 3];
            data.push(next);
        }
        assert_eq!(round_trip(&data), data);
    }

    #[test]
    fn bits_and_codes_are_written_in_the_two_orders_deflate_wants() {
        let mut writer = BitWriter::new(4);
        // LSB-first: 0b101 in three bits lands as 1, 0, 1 from bit zero up.
        writer.bits(0b101, 3);
        // MSB-first: the code 0b110 lands as 1, 1, 0 from bit three up.
        writer.code(0b110, 3);
        let out = writer.finish();
        assert_eq!(out, vec![0b0_011_101], "{out:?}");
    }

    #[test]
    fn the_fixed_code_table_matches_rfc_1951() {
        // The four ranges, at both ends of each.
        assert_eq!(fixed_literal(0), (0b0011_0000, 8));
        assert_eq!(fixed_literal(143), (0b1011_1111, 8));
        assert_eq!(fixed_literal(144), (0b1_1001_0000, 9));
        assert_eq!(fixed_literal(255), (0b1_1111_1111, 9));
        assert_eq!(fixed_literal(256), (0b000_0000, 7));
        assert_eq!(fixed_literal(279), (0b001_0111, 7));
        assert_eq!(fixed_literal(280), (0b1100_0000, 8));
        assert_eq!(fixed_literal(287), (0b1100_0111, 8));
    }

    #[test]
    fn the_length_and_distance_tables_cover_their_ranges_without_gaps() {
        // Every length from 3 to 258 maps to a code whose base plus extra
        // reaches it exactly.
        for length in MIN_MATCH..=MAX_MATCH {
            let (symbol, extra, bits) = length_symbol(length);
            let (declared_bits, base) = LENGTH_CODES[(symbol - 257) as usize];
            assert_eq!(u32::from(declared_bits), bits, "length {length}");
            assert_eq!(
                usize::from(base) + extra as usize,
                length,
                "length {length}"
            );
            assert!(
                extra < (1 << bits) || bits == 0,
                "length {length} overflows its extra bits"
            );
        }
        for distance in 1..=WINDOW {
            let (code, extra, bits) = distance_symbol(distance);
            let (declared_bits, base) = DISTANCE_CODES[code as usize];
            assert_eq!(u32::from(declared_bits), bits, "distance {distance}");
            assert_eq!(usize::from(base) + extra as usize, distance);
        }
    }
}
