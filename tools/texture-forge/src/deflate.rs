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

/// Largest distance a match may reach back. RFC 1951 §3.2.5.
pub const WINDOW: usize = 32_768;

/// Longest match deflate can encode.
pub const MAX_MATCH: usize = 258;

/// Shortest match worth encoding: below this a literal is cheaper.
pub const MIN_MATCH: usize = 3;

/// How many candidates the match finder will consider at one position.
///
/// The chain is ordered nearest-first, and a nearer match encodes in fewer
/// bits, so the early candidates are the valuable ones. Thirty-two is where
/// the ratio stopped improving on this data.
const MAX_CHAIN: usize = 32;

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

/// Compress with one fixed-Huffman block.
///
/// Returns the raw deflate stream, with no zlib wrapper.
#[must_use]
pub fn fixed_block(data: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter::new(data.len() / 2 + 64);
    writer.bits(1, 1); // BFINAL
    writer.bits(1, 2); // BTYPE = 01, fixed Huffman

    let mut head = vec![usize::MAX; HASH_SIZE];
    let mut prev = vec![usize::MAX; data.len().max(1)];

    let mut at = 0usize;
    while at < data.len() {
        let (mut best_length, mut best_distance) = (0usize, 0usize);

        if at + MIN_MATCH <= data.len() {
            let slot = hash3(data, at);
            let mut candidate = head[slot];
            let limit = at.saturating_sub(WINDOW);
            let mut examined = 0;

            while candidate != usize::MAX && candidate >= limit && examined < MAX_CHAIN {
                let mut length = 0usize;
                let max = MAX_MATCH.min(data.len() - at);
                while length < max && data[candidate + length] == data[at + length] {
                    length += 1;
                }
                if length > best_length {
                    best_length = length;
                    best_distance = at - candidate;
                    if length == max {
                        break;
                    }
                }
                candidate = prev[candidate];
                examined += 1;
            }

            prev[at] = head[slot];
            head[slot] = at;
        }

        if best_length >= MIN_MATCH {
            let (symbol, extra, extra_bits) = length_symbol(best_length);
            let (code, length_bits) = fixed_literal(symbol);
            writer.code(code, length_bits);
            if extra_bits > 0 {
                writer.bits(extra, extra_bits);
            }
            let (distance_code, distance_extra, distance_bits) = distance_symbol(best_distance);
            writer.code(distance_code, 5);
            if distance_bits > 0 {
                writer.bits(distance_extra, distance_bits);
            }

            // Index every position the match covered, or the next match will
            // have nothing to chain to and long runs stop compressing.
            for skip in 1..best_length {
                let position = at + skip;
                if position + MIN_MATCH <= data.len() {
                    let slot = hash3(data, position);
                    prev[position] = head[slot];
                    head[slot] = position;
                }
            }
            at += best_length;
        } else {
            let (code, bits) = fixed_literal(u16::from(data[at]));
            writer.code(code, bits);
            at += 1;
        }
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

/// A reference inflater, for tests only.
///
/// Written from RFC 1951 independently of the encoder above, and kept in
/// the crate rather than in one test module so the PNG encoder can be
/// checked against it too. A compressor that only agrees with itself has
/// proved nothing.
#[cfg(test)]
pub(crate) mod reference {
    use super::{DISTANCE_CODES, LENGTH_CODES};

    /// A minimal inflater, written from RFC 1951 independently of the encoder
    /// above, so a shared misreading of the spec cannot pass unnoticed.
    pub(crate) struct Inflater<'a> {
        data: &'a [u8],
        at: usize,
        bit: u32,
    }

    impl<'a> Inflater<'a> {
        pub(crate) fn new(data: &'a [u8]) -> Self {
            Self {
                data,
                at: 0,
                bit: 0,
            }
        }

        fn bits(&mut self, count: u32) -> u32 {
            let mut value = 0u32;
            for index in 0..count {
                let byte = self.data[self.at];
                let bit = (byte >> self.bit) & 1;
                value |= u32::from(bit) << index;
                self.bit += 1;
                if self.bit == 8 {
                    self.bit = 0;
                    self.at += 1;
                }
            }
            value
        }

        fn align(&mut self) {
            if self.bit > 0 {
                self.bit = 0;
                self.at += 1;
            }
        }

        /// Decode one fixed literal/length symbol, MSB-first.
        fn symbol(&mut self) -> u16 {
            let mut code = 0u32;
            for length in 1..=9u32 {
                code = (code << 1) | self.bits(1);
                match length {
                    7 if code <= 0b001_0111 => return (code + 256) as u16,
                    8 if (0b0011_0000..=0b1011_1111).contains(&code) => {
                        return (code - 0b0011_0000) as u16
                    }
                    8 if (0b1100_0000..=0b1100_0111).contains(&code) => {
                        return (code - 0b1100_0000 + 280) as u16
                    }
                    9 if code >= 0b1_1001_0000 => return (code - 0b1_1001_0000 + 144) as u16,
                    _ => {}
                }
            }
            panic!("no fixed code matched");
        }

        pub(crate) fn inflate(&mut self) -> Vec<u8> {
            let mut out = Vec::new();
            loop {
                let final_block = self.bits(1);
                let kind = self.bits(2);
                match kind {
                    0 => {
                        self.align();
                        let len = u16::from_le_bytes([self.data[self.at], self.data[self.at + 1]]);
                        let nlen =
                            u16::from_le_bytes([self.data[self.at + 2], self.data[self.at + 3]]);
                        assert_eq!(len, !nlen, "stored block LEN/NLEN mismatch");
                        self.at += 4;
                        out.extend_from_slice(&self.data[self.at..self.at + len as usize]);
                        self.at += len as usize;
                    }
                    1 => loop {
                        let symbol = self.symbol();
                        if symbol == 256 {
                            break;
                        }
                        if symbol < 256 {
                            out.push(symbol as u8);
                            continue;
                        }
                        let index = (symbol - 257) as usize;
                        let (extra, base) = LENGTH_CODES[index];
                        let length = base as usize + self.bits(u32::from(extra)) as usize;

                        let mut code = 0u32;
                        for _ in 0..5 {
                            code = (code << 1) | self.bits(1);
                        }
                        let (extra, base) = DISTANCE_CODES[code as usize];
                        let distance = base as usize + self.bits(u32::from(extra)) as usize;

                        let start = out.len() - distance;
                        for offset in 0..length {
                            out.push(out[start + offset]);
                        }
                    },
                    other => panic!("unsupported block type {other}"),
                }
                if final_block == 1 {
                    return out;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::deflate::reference::Inflater;

    fn round_trip(data: &[u8]) -> Vec<u8> {
        Inflater::new(&deflate(data)).inflate()
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
        assert_eq!(Inflater::new(&compressed).inflate(), data);
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
        assert_eq!(Inflater::new(&compressed).inflate(), data);
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
