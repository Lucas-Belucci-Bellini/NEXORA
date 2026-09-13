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
//! # Three block types, and the measurement that ordered them
//!
//! This module used to say fixed Huffman was enough, on the reasoning that
//! what texture data wants is the **match finder** - palette-quantised albedo
//! is long runs, and a run is one length-distance pair however it is coded.
//! That reasoning was wrong, and the way to find out was to hold one variable
//! still at a time. `zlib` accepts `Z_FIXED`, which keeps its own matcher and
//! swaps the dynamic table for the fixed one, so one comparison becomes two:
//!
//! ```text
//! this module, fixed table      148,500        <- matcher within 2.2% of zlib
//! zlib -9 Z_FIXED               145,278           |
//! zlib -9, dynamic table        114,187        <- 91% of the gap was the table
//!
//! this module, dynamic table    116,842        <- 1.023x, from 1.388x
//! ```
//!
//! Runs are indeed cheap either way. What is not cheap is everything that is
//! *not* a run: the fixed table spends nine bits on every byte above 143, and
//! filtered scanlines of procedural noise are mostly such bytes. So all three
//! block types are built here, and [`deflate`] emits whichever is smallest for
//! the data in hand - which is also why adding dynamic blocks cannot make any
//! file larger.
//!
//! # Correctness is checked by somebody else's decoder
//!
//! Every table below is a place to be silently wrong. So the test suite
//! round-trips through an inflater written independently in this module's
//! tests, and the PNG files themselves are decompressed by Python's `zlib`
//! outside the test suite entirely. A compressor that only agrees with itself
//! has proved nothing.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

/// One step of the encoding: a byte to copy out, or a span to copy back.
#[derive(Debug, Clone, Copy)]
enum Token {
    Literal(u8),
    Copy(Match),
}

/// Turn bytes into the tokens deflate encodes, independently of how they are
/// coded afterwards.
///
/// Splitting this out is what lets the fixed and dynamic encoders be compared
/// honestly: they are handed the *same* tokens, so the only thing the
/// comparison measures is the code table.
///
/// Matching is **lazy**: a match found at one position is held rather than
/// emitted, and dropped in favour of a strictly longer one starting a byte
/// later. Greedy matching cannot do this - it commits to the first long match
/// it sees and steps past the better one that started one byte in. The cost is
/// the literal left behind; the gain is everything the longer match covers.
fn tokenize(data: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
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
                None => tokens.push(Token::Literal(data[at])),
            }
            at += 1;
            continue;
        };

        let start = at - 1;
        if here.is_some_and(|found| found.length > previous.length) {
            // One byte along does better. Spend `start` as a literal and hold
            // the new one, which may itself lose to the position after it.
            tokens.push(Token::Literal(data[start]));
            held = here;
            at += 1;
        } else {
            tokens.push(Token::Copy(previous));
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
        tokens.push(Token::Copy(previous));
    }

    tokens
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

/// Compress with one fixed-Huffman block: the table both sides already know,
/// so nothing describing it travels in the stream.
///
/// Returns the raw deflate stream, with no zlib wrapper.
#[must_use]
pub fn fixed_block(data: &[u8]) -> Vec<u8> {
    encode_fixed(&tokenize(data))
}

fn encode_fixed(tokens: &[Token]) -> Vec<u8> {
    let mut writer = BitWriter::new(tokens.len() + 64);
    writer.bits(1, 1); // BFINAL
    writer.bits(1, 2); // BTYPE = 01, fixed Huffman

    for token in tokens {
        match *token {
            Token::Literal(byte) => emit_literal(&mut writer, byte),
            Token::Copy(found) => emit_match(&mut writer, found),
        }
    }

    let (code, bits) = fixed_literal(256); // end of block
    writer.code(code, bits);
    writer.finish()
}

/// Longest code deflate allows for a literal, length or distance. RFC 1951 §3.2.7.
const MAX_CODE_BITS: usize = 15;

/// Longest code allowed for the alphabet that describes the other two.
const MAX_CODE_LENGTH_BITS: usize = 7;

/// How many literal/length symbols exist: 0..=255 literals, 256 end-of-block,
/// 257..=285 lengths.
const LITERAL_SYMBOLS: usize = 286;

/// How many distance symbols exist.
const DISTANCE_SYMBOLS: usize = 30;

/// The order the code-length code's own lengths are written in. RFC 1951 §3.2.7
/// puts the rarely-used lengths last so trailing zeros can be dropped.
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Code lengths for an optimal prefix code over `frequencies`, none longer
/// than `limit`.
///
/// Huffman's algorithm on its own can build a code deeper than deflate's
/// fifteen bits - a Fibonacci-shaped frequency distribution over 286 symbols
/// reaches 285. The repair works on the histogram of lengths rather than the
/// tree: take a leaf from just under the limit, split it in two at the limit,
/// and one overlong code comes up to meet it. That keeps the Kraft sum at
/// exactly one, which is asserted below rather than assumed - a code that does
/// not sum to one is not a code, and writing one would produce a stream no
/// decoder can read.
fn code_lengths(frequencies: &[u32], limit: usize) -> Vec<u8> {
    let mut lengths = vec![0u8; frequencies.len()];
    let used: Vec<usize> = (0..frequencies.len())
        .filter(|&symbol| frequencies[symbol] > 0)
        .collect();

    match used.len() {
        0 => return lengths,
        // A prefix code over a single symbol is incomplete, which RFC 1951
        // tolerates only for a lone distance code. Giving a second symbol the
        // same length makes it complete wherever it is used, for one bit.
        1 => {
            lengths[used[0]] = 1;
            lengths[usize::from(used[0] == 0)] = 1;
            return lengths;
        }
        _ => {}
    }

    enum Node {
        Leaf(usize),
        Branch(usize, usize),
    }

    // Huffman's algorithm over an explicit arena. The heap is keyed by
    // (weight, arena index) so ties break the same way every run: the output
    // of this tool has to be a function of its input.
    let mut arena: Vec<Node> = Vec::with_capacity(used.len() * 2);
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::with_capacity(used.len());
    for &symbol in &used {
        arena.push(Node::Leaf(symbol));
        heap.push(Reverse((u64::from(frequencies[symbol]), arena.len() - 1)));
    }
    while heap.len() > 1 {
        let Reverse((left_weight, left)) = heap.pop().expect("two or more nodes");
        let Reverse((right_weight, right)) = heap.pop().expect("two or more nodes");
        arena.push(Node::Branch(left, right));
        heap.push(Reverse((left_weight + right_weight, arena.len() - 1)));
    }
    let root = heap.pop().expect("one node remains").0 .1;

    // Depths by explicit stack, because the tree can be deep enough to
    // overflow a recursive walk on the very inputs the limit exists for.
    let mut depth = vec![0usize; arena.len()];
    let mut natural = vec![0usize; frequencies.len()];
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        match arena[index] {
            Node::Leaf(symbol) => natural[symbol] = depth[index].max(1),
            Node::Branch(left, right) => {
                depth[left] = depth[index] + 1;
                depth[right] = depth[index] + 1;
                stack.push(left);
                stack.push(right);
            }
        }
    }

    let mut count = vec![0usize; limit + 2];
    for &symbol in &used {
        count[natural[symbol].min(limit)] += 1;
    }

    // Clamping shortened codes, so the histogram now claims more of the code
    // space than exists. Each repair takes one code from the deepest level
    // that still has one, sends it a level down, and brings one of the
    // clamped codes up to be its sibling. That is exactly one unit of code
    // space returned, so the loop is driven by the Kraft sum itself and stops
    // when the sum is right - not by a count of how many codes were clamped,
    // which is the same loop run too few times.
    let target = 1u64 << limit;
    let mut kraft: u64 = (1..=limit)
        .map(|bits| count[bits] as u64 * (1u64 << (limit - bits)))
        .sum();
    while kraft > target {
        let mut bits = limit - 1;
        while bits >= 1 && count[bits] == 0 {
            bits -= 1;
        }
        assert!(bits >= 1, "no code is short enough to lengthen");
        assert!(count[limit] >= 1, "nothing sits at the limit to bring up");
        count[bits] -= 1;
        count[bits + 1] += 2;
        count[limit] -= 1;
        kraft -= 1;
    }
    assert_eq!(
        kraft, target,
        "the length histogram does not describe a complete prefix code"
    );

    // Longest codes to the least frequent symbols; ties by symbol, again so
    // the same input gives the same bytes.
    let mut order = used;
    order.sort_by_key(|&symbol| (frequencies[symbol], symbol));
    let mut next = 0usize;
    for bits in (1..=limit).rev() {
        for _ in 0..count[bits] {
            lengths[order[next]] = bits as u8;
            next += 1;
        }
    }
    assert_eq!(
        next,
        order.len(),
        "not every used symbol was given a length"
    );

    lengths
}

/// The canonical code for each length. RFC 1951 §3.2.2: codes of one length
/// are consecutive, and each length starts where the previous one left off,
/// shifted up a bit.
fn canonical_codes(lengths: &[u8]) -> Vec<u32> {
    let longest = lengths.iter().copied().max().unwrap_or(0) as usize;
    let mut count = vec![0u32; longest + 2];
    for &length in lengths {
        if length > 0 {
            count[usize::from(length)] += 1;
        }
    }
    let mut next = vec![0u32; longest + 2];
    let mut code = 0u32;
    for bits in 1..=longest {
        code = (code + count[bits - 1]) << 1;
        next[bits] = code;
    }
    lengths
        .iter()
        .map(|&length| {
            if length == 0 {
                return 0;
            }
            let assigned = next[usize::from(length)];
            next[usize::from(length)] += 1;
            assigned
        })
        .collect()
}

/// One entry of the run-length-encoded code-length sequence:
/// `(symbol, extra value, extra bit count)`.
type LengthCode = (u8, u32, u32);

/// Run-length encode a code-length sequence with the 19-symbol alphabet of
/// RFC 1951 §3.2.7.
///
/// Symbol 16 repeats the previous length 3..=6 times, 17 writes 3..=10 zeros
/// and 18 writes 11..=138. Unused symbols are the common case - most of the
/// 286 literal codes never appear in one block - so the runs of zero are where
/// the header's size actually goes.
fn run_length_encode(lengths: &[u8]) -> Vec<LengthCode> {
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < lengths.len() {
        let value = lengths[at];
        let mut run = 1usize;
        while at + run < lengths.len() && lengths[at + run] == value {
            run += 1;
        }

        if value == 0 {
            while run >= 11 {
                let take = run.min(138);
                out.push((18u8, (take - 11) as u32, 7));
                run -= take;
                at += take;
            }
            while run >= 3 {
                let take = run.min(10);
                out.push((17u8, (take - 3) as u32, 3));
                run -= take;
                at += take;
            }
        } else {
            out.push((value, 0, 0));
            at += 1;
            run -= 1;
            while run >= 3 {
                let take = run.min(6);
                out.push((16u8, (take - 3) as u32, 2));
                run -= take;
                at += take;
            }
        }

        for _ in 0..run {
            out.push((value, 0, 0));
            at += 1;
        }
    }
    out
}

/// Compress with one dynamic-Huffman block: a code built for this data, and
/// the description of that code carried in the stream ahead of it.
///
/// Worth its header exactly when the data's symbol distribution is far enough
/// from the fixed table's assumptions to pay for describing a better one.
/// [`deflate`] decides that by building both and weighing them, so the header
/// can never cost anything.
///
/// Returns the raw deflate stream, with no zlib wrapper.
#[must_use]
pub fn dynamic_block(data: &[u8]) -> Vec<u8> {
    encode_dynamic(&tokenize(data))
}

fn encode_dynamic(tokens: &[Token]) -> Vec<u8> {
    let mut literal_frequency = vec![0u32; LITERAL_SYMBOLS];
    let mut distance_frequency = vec![0u32; DISTANCE_SYMBOLS];
    literal_frequency[256] = 1; // the end-of-block symbol is always written
    for token in tokens {
        match *token {
            Token::Literal(byte) => literal_frequency[usize::from(byte)] += 1,
            Token::Copy(found) => {
                let (symbol, _, _) = length_symbol(found.length);
                literal_frequency[usize::from(symbol)] += 1;
                let (code, _, _) = distance_symbol(found.distance);
                distance_frequency[code as usize] += 1;
            }
        }
    }

    let literal_lengths = code_lengths(&literal_frequency, MAX_CODE_BITS);
    let mut distance_lengths = code_lengths(&distance_frequency, MAX_CODE_BITS);
    if distance_lengths.iter().all(|&length| length == 0) {
        // No matches at all. A block with no distance code is legal but reads
        // differently in different decoders; two one-bit codes cost two bits
        // and are unambiguous.
        distance_lengths[0] = 1;
        distance_lengths[1] = 1;
    }
    let literal_codes = canonical_codes(&literal_lengths);
    let distance_codes = canonical_codes(&distance_lengths);

    // Trailing unused symbols do not have to be described.
    let mut hlit = LITERAL_SYMBOLS;
    while hlit > 257 && literal_lengths[hlit - 1] == 0 {
        hlit -= 1;
    }
    let mut hdist = DISTANCE_SYMBOLS;
    while hdist > 1 && distance_lengths[hdist - 1] == 0 {
        hdist -= 1;
    }

    let mut described = Vec::with_capacity(hlit + hdist);
    described.extend_from_slice(&literal_lengths[..hlit]);
    described.extend_from_slice(&distance_lengths[..hdist]);
    let sequence = run_length_encode(&described);

    let mut header_frequency = vec![0u32; 19];
    for &(symbol, _, _) in &sequence {
        header_frequency[usize::from(symbol)] += 1;
    }
    let header_lengths = code_lengths(&header_frequency, MAX_CODE_LENGTH_BITS);
    let header_codes = canonical_codes(&header_lengths);

    let mut hclen = CODE_LENGTH_ORDER.len();
    while hclen > 4 && header_lengths[CODE_LENGTH_ORDER[hclen - 1]] == 0 {
        hclen -= 1;
    }

    let mut writer = BitWriter::new(tokens.len() + 64);
    writer.bits(1, 1); // BFINAL
    writer.bits(2, 2); // BTYPE = 10, dynamic Huffman
    writer.bits((hlit - 257) as u32, 5);
    writer.bits((hdist - 1) as u32, 5);
    writer.bits((hclen - 4) as u32, 4);
    for &slot in &CODE_LENGTH_ORDER[..hclen] {
        writer.bits(u32::from(header_lengths[slot]), 3);
    }
    for &(symbol, extra, extra_bits) in &sequence {
        let slot = usize::from(symbol);
        writer.code(header_codes[slot], u32::from(header_lengths[slot]));
        if extra_bits > 0 {
            writer.bits(extra, extra_bits);
        }
    }

    let write_symbol = |writer: &mut BitWriter, symbol: usize| {
        writer.code(literal_codes[symbol], u32::from(literal_lengths[symbol]));
    };
    for token in tokens {
        match *token {
            Token::Literal(byte) => write_symbol(&mut writer, usize::from(byte)),
            Token::Copy(found) => {
                let (symbol, extra, extra_bits) = length_symbol(found.length);
                write_symbol(&mut writer, usize::from(symbol));
                if extra_bits > 0 {
                    writer.bits(extra, extra_bits);
                }
                let (code, distance_extra, distance_bits) = distance_symbol(found.distance);
                let slot = code as usize;
                writer.code(distance_codes[slot], u32::from(distance_lengths[slot]));
                if distance_bits > 0 {
                    writer.bits(distance_extra, distance_bits);
                }
            }
        }
    }
    write_symbol(&mut writer, 256);

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

/// Compress with whichever of the three block types comes out smallest.
///
/// Each one wins somewhere. **Dynamic** almost always wins on real texture
/// data, which is what the measurement in this module's header is about.
/// **Fixed** wins on short inputs, where a table costs more to describe than
/// it saves. **Stored** wins on data with no structure at all, where fixed
/// Huffman's nine bits for any byte above 143 make the output larger than the
/// input.
///
/// They are built and weighed rather than predicted. Matching runs once and
/// both Huffman encoders are handed the same tokens, so the extra work is two
/// passes over a token stream and the choice cannot be wrong.
#[must_use]
pub fn deflate(data: &[u8]) -> Vec<u8> {
    let tokens = tokenize(data);
    let mut best = encode_fixed(&tokens);
    let dynamic = encode_dynamic(&tokens);
    if dynamic.len() < best.len() {
        best = dynamic;
    }
    let stored = stored_blocks(data);
    if stored.len() <= best.len() {
        return stored;
    }
    best
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
/// Reads all three block types RFC 1951 defines, which is also all three this
/// project writes. Block type 3 is reserved by the format and never valid, so
/// it is named as damage rather than as something unsupported.
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
            2 => reader.dynamic_block(&mut out)?,
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

    /// Decode a block coded with the table both sides already know.
    ///
    /// Built as a [`Huffman`] like any other rather than decoded by hand, so
    /// the two block types share one symbol loop and one set of bounds checks.
    fn fixed_block(&mut self, out: &mut Vec<u8>) -> Result<()> {
        let mut literal_lengths = [0u8; 288];
        for (symbol, length) in literal_lengths.iter_mut().enumerate() {
            *length = match symbol {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            };
        }
        // Thirty-two five-bit distance codes, of which 30 and 31 never appear
        // in a valid stream; hitting one lands in the table check below.
        let distance_lengths = [5u8; 32];

        let literal = Huffman::new(&literal_lengths)?;
        let distance = Huffman::new(&distance_lengths)?;
        self.huffman_block(out, &literal, &distance)
    }

    /// Decode a block that carries its own code, described ahead of the data.
    fn dynamic_block(&mut self, out: &mut Vec<u8>) -> Result<()> {
        let hlit = self.bits(5)? as usize + 257;
        let hdist = self.bits(5)? as usize + 1;
        let hclen = self.bits(4)? as usize + 4;
        if hlit > LITERAL_SYMBOLS || hdist > DISTANCE_SYMBOLS {
            return Err(damaged("a dynamic block claims more codes than exist")
                .with_context("hlit", hlit.to_string())
                .with_context("hdist", hdist.to_string()));
        }

        let mut header_lengths = [0u8; 19];
        for &slot in &CODE_LENGTH_ORDER[..hclen] {
            header_lengths[slot] = self.bits(3)? as u8;
        }
        let header = Huffman::new(&header_lengths)?;

        let mut lengths = vec![0u8; hlit + hdist];
        let mut at = 0usize;
        while at < lengths.len() {
            let symbol = header.decode(self)?;
            let (repeat, value) = match symbol {
                0..=15 => {
                    lengths[at] = symbol as u8;
                    at += 1;
                    continue;
                }
                16 => {
                    let previous = *lengths
                        .get(at.wrapping_sub(1))
                        .ok_or_else(|| damaged("a code-length repeat with no length before it"))?;
                    (3 + self.bits(2)? as usize, previous)
                }
                17 => (3 + self.bits(3)? as usize, 0),
                18 => (11 + self.bits(7)? as usize, 0),
                _ => return Err(damaged("a code-length symbol outside the alphabet")),
            };
            if at + repeat > lengths.len() {
                return Err(damaged("a code-length repeat runs past the table")
                    .with_context("at", at.to_string())
                    .with_context("repeat", repeat.to_string()));
            }
            lengths[at..at + repeat].fill(value);
            at += repeat;
        }

        let literal = Huffman::new(&lengths[..hlit])?;
        let distance = Huffman::new(&lengths[hlit..])?;
        self.huffman_block(out, &literal, &distance)
    }

    /// The part both Huffman block types share: symbols until end-of-block,
    /// literals copied out and matches copied back.
    fn huffman_block(
        &mut self,
        out: &mut Vec<u8>,
        literal: &Huffman,
        distance: &Huffman,
    ) -> Result<()> {
        loop {
            let symbol = literal.decode(self)?;
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

            let code = distance.decode(self)?;
            let (extra, base) = *DISTANCE_CODES
                .get(usize::from(code))
                .ok_or_else(|| damaged("a distance code outside the table"))?;
            let span = usize::from(base) + self.bits(u32::from(extra))? as usize;

            if span == 0 || span > out.len() {
                return Err(damaged("a match reaches back further than the output")
                    .with_context("distance", span.to_string())
                    .with_context("available", out.len().to_string()));
            }

            grow(out, length)?;
            let start = out.len() - span;
            for offset in 0..length {
                // Copied one byte at a time on purpose: a match may overlap
                // itself, which is how deflate encodes a run.
                out.push(out[start + offset]);
            }
        }
    }
}

/// A canonical Huffman code, in the form that decodes without a tree.
///
/// `counts[bits]` is how many codes are that long and `symbols` holds them in
/// canonical order, so walking the lengths from one upwards finds the symbol
/// by arithmetic. Building an actual tree of pointers would be a node per
/// symbol per block for no gain.
struct Huffman {
    counts: [u16; MAX_CODE_BITS + 1],
    symbols: Vec<u16>,
}

impl Huffman {
    /// # Errors
    ///
    /// Refuses lengths past the fifteen bits deflate allows, and lengths that
    /// describe more codes than those bits can hold - an over-subscribed set
    /// is not a code, and decoding it would return whatever the arithmetic
    /// happened to land on.
    fn new(lengths: &[u8]) -> Result<Self> {
        let mut counts = [0u16; MAX_CODE_BITS + 1];
        for &length in lengths {
            let bits = usize::from(length);
            if bits > MAX_CODE_BITS {
                return Err(damaged("a code longer than deflate's fifteen bits")
                    .with_context("bits", bits.to_string()));
            }
            counts[bits] += 1;
        }
        counts[0] = 0;

        // Kraft: each length halves what is left. Going negative means the
        // lengths overlap, which no prefix code does.
        let mut left = 1i32;
        for &at_this_length in &counts[1..=MAX_CODE_BITS] {
            left = (left << 1) - i32::from(at_this_length);
            if left < 0 {
                return Err(damaged("the code lengths describe overlapping codes"));
            }
        }
        // `left > 0` is an *incomplete* code, which RFC 1951 permits for a
        // lone distance code. It is only wrong if the stream then uses a code
        // the lengths never defined, and `decode` catches that where it
        // happens rather than refusing the whole block here.

        let mut offsets = [0usize; MAX_CODE_BITS + 2];
        for bits in 1..=MAX_CODE_BITS {
            offsets[bits + 1] = offsets[bits] + usize::from(counts[bits]);
        }
        let mut symbols = vec![0u16; offsets[MAX_CODE_BITS + 1]];
        for (symbol, &length) in lengths.iter().enumerate() {
            if length > 0 {
                let slot = usize::from(length);
                symbols[offsets[slot]] = symbol as u16;
                offsets[slot] += 1;
            }
        }

        Ok(Self { counts, symbols })
    }

    /// Read one symbol, most-significant bit first.
    fn decode(&self, reader: &mut BitReader<'_>) -> Result<u16> {
        let mut code = 0i32;
        let mut first = 0i32;
        let mut index = 0i32;
        for bits in 1..=MAX_CODE_BITS {
            code |= reader.bits(1)? as i32;
            let count = i32::from(self.counts[bits]);
            if code - first < count {
                let slot = usize::try_from(index + (code - first))
                    .map_err(|_| damaged("a Huffman code indexes outside its own table"))?;
                return self
                    .symbols
                    .get(slot)
                    .copied()
                    .ok_or_else(|| damaged("a Huffman code names a symbol the lengths omit"));
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err(damaged("no Huffman code matches these bits"))
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

    /// Bytes with no structure of any kind: full byte entropy *and* no
    /// repeats a match finder can reach, which are two different things.
    fn unmatchable(count: usize) -> Vec<u8> {
        let mut state = 0x243F_6A88_85A3_08D3u64;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 33) as u8
            })
            .collect()
    }

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
    // Dynamic blocks, which the constants above deliberately are not: they
    // were produced with `Z_FIXED`. These come from zlib's default strategy,
    // so they carry a real code description - HLIT, HDIST, HCLEN, the
    // code-length alphabet and its run-length symbols. Nothing else in this
    // file proves the decoder reads a tree it did not write.
    const ZLIB_DYNAMIC_SKEWED: &[u8] = &[
        237, 193, 217, 14, 64, 48, 20, 5, 192, 182, 212, 78, 181, 182, 255, 255, 83, 241, 34, 33,
        17, 164, 210, 229, 58, 51, 140, 115, 177, 73, 118, 233, 153, 188, 148, 61, 145, 191, 86,
        216, 42, 63, 87, 185, 80, 123, 211, 132, 166, 141, 70, 71, 129, 34, 175, 255, 59, 13, 55,
        12, 216, 27, 192, 145, 17, 194, 50, 65, 164, 102, 160, 111, 1, 56, 88, 1,
    ];
    const ZLIB_DYNAMIC_MIXED: &[u8] = &[
        237, 210, 53, 91, 22, 0, 24, 133, 97, 131, 48, 8, 139, 48, 16, 44, 48, 177, 9, 27, 21, 9,
        149, 178, 8, 11, 27, 176, 193, 32, 44, 236, 192, 0, 177, 0, 11, 176, 8, 139, 48, 8, 11,
        177, 64, 12, 194, 2, 44, 194, 32, 44, 192, 201, 225, 123, 135, 231, 55, 120, 177, 156, 241,
        89, 206, 221, 64, 85, 67, 171, 67, 151, 158, 253, 205, 70, 142, 155, 56, 121, 250, 92, 207,
        21, 107, 55, 108, 11, 58, 24, 118, 250, 252, 165, 107, 233, 247, 179, 94, 190, 249, 80, 94,
        85, 211, 184, 89, 203, 182, 6, 70, 198, 131, 135, 141, 177, 117, 116, 158, 181, 96, 201,
        170, 128, 205, 187, 14, 28, 57, 113, 38, 46, 225, 230, 157, 135, 207, 10, 138, 74, 190,
        255, 106, 160, 200, 12, 48, 31, 101, 101, 55, 69, 50, 123, 67, 195, 35, 47, 92, 150, 204,
        151, 234, 90, 165, 230, 173, 36, 51, 222, 201, 101, 246, 194, 165, 146, 57, 27, 159, 152,
        114, 247, 145, 100, 154, 104, 106, 235, 117, 237, 37, 153, 121, 94, 43, 125, 55, 110, 151,
        204, 173, 204, 236, 220, 183, 31, 37, 211, 169, 123, 95, 147, 225, 99, 37, 179, 110, 203,
        238, 224, 163, 39, 37, 243, 170, 184, 180, 226, 119, 67, 201, 12, 177, 176, 182, 159, 58,
        67, 50, 135, 34, 162, 98, 174, 92, 151, 204, 143, 58, 101, 181, 214, 237, 36, 51, 201, 213,
        125, 209, 178, 213, 146, 185, 152, 148, 122, 239, 241, 115, 201, 180, 208, 233, 216, 173,
        247, 64, 201, 44, 246, 246, 219, 180, 99, 159, 100, 30, 60, 205, 123, 247, 233, 171, 100,
        122, 244, 51, 29, 97, 57, 65, 50, 91, 247, 132, 28, 59, 117, 78, 50, 239, 203, 42, 255, 52,
        106, 42, 153, 209, 54, 14, 211, 102, 206, 151, 204, 241, 232, 216, 171, 55, 110, 75, 230,
        175, 138, 122, 155, 246, 157, 37, 227, 54, 199, 99, 249, 154, 245, 146, 73, 78, 203, 120,
        242, 226, 181, 100, 116, 245, 13, 251, 12, 26, 42, 25, 31, 255, 192, 157, 251, 15, 75, 38,
        39, 191, 240, 243, 183, 159, 146, 81, 156, 41, 25, 197, 153, 146, 81, 156, 41, 25, 197,
        153, 146, 81, 156, 41, 25, 154, 112, 163, 137, 100, 154, 208, 165, 9, 31, 154, 200, 161,
        137, 254, 52, 17, 68, 19, 229, 52, 97, 75, 19, 103, 104, 66, 149, 38, 230, 210, 68, 58, 77,
        24, 208, 68, 0, 77, 20, 208, 132, 57, 77, 132, 210, 68, 53, 77, 56, 209, 68, 60, 77, 104,
        210, 132, 23, 77, 100, 210, 68, 119, 154, 216, 66, 19, 197, 52, 97, 65, 19, 17, 52, 81, 71,
        19, 174, 52, 145, 68, 19, 58, 52, 225, 77, 19, 79, 105, 162, 31, 77, 236, 161, 137, 50,
        154, 176, 161, 137, 104, 154, 80, 161, 137, 57, 52, 145, 70, 19, 250, 52, 225, 79, 19, 249,
        52, 97, 70, 19, 7, 105, 162, 138, 38, 28, 105, 34, 142, 38, 52, 104, 194, 147, 38, 238,
        211, 132, 17, 77, 108, 166, 137, 34, 154, 24, 69, 19, 225, 52, 81, 75, 19, 46, 52, 145, 72,
        19, 218, 52, 177, 146, 38, 178, 105, 162, 47, 77, 236, 166, 137, 82, 154, 176, 166, 137,
        40, 154, 80, 166, 9, 119, 154, 72, 165, 137, 142, 52, 225, 71, 19, 121, 52, 97, 74, 19, 33,
        52, 81, 73, 19, 14, 52, 17, 75, 19, 234, 52, 225, 65, 19, 25, 52, 97, 72, 19, 129, 52, 81,
        72, 19, 35, 105, 34, 140, 38, 106, 104, 194, 153, 38, 18, 104, 66, 139, 38, 86, 208, 68,
        22, 77, 24, 211, 196, 46, 154, 40, 161, 9, 43, 154, 136, 164, 9, 37, 154, 152, 77, 19, 41,
        52, 161, 71, 19, 190, 52, 145, 75, 19, 38, 52, 17, 76, 19, 21, 52, 97, 79, 19, 49, 52, 161,
        6, 19, 118, 150, 206, 246, 78, 22, 245, 91, 191, 255, 199, 254, 3,
    ];

    /// The plaintext of `ZLIB_DYNAMIC_SKEWED`: symbol `i`, `i * i + 1` times.
    fn skewed_plaintext() -> Vec<u8> {
        (0..24u8)
            .flat_map(|symbol| {
                std::iter::repeat_n(symbol, usize::from(symbol) * usize::from(symbol) + 1)
            })
            .collect()
    }

    /// The plaintext of `ZLIB_DYNAMIC_MIXED`: an arithmetic sequence with no
    /// short period, then a run that is nothing but matches.
    fn mixed_plaintext() -> Vec<u8> {
        let mut data: Vec<u8> = (0..2000usize)
            .map(|index| ((index * 7 + index / 13) % 251) as u8)
            .collect();
        for _ in 0..120 {
            data.extend_from_slice(b"NEXORA");
        }
        data
    }

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
    fn dynamic_blocks_written_by_zlib_inflate_to_what_zlib_was_given() {
        // The decoder and the encoder here could share a misreading of RFC
        // 1951 §3.2.7 and every round-trip in this file would still pass.
        // These two streams were built by zlib, which has not read this code.
        assert_eq!(inflate(ZLIB_DYNAMIC_SKEWED).unwrap(), skewed_plaintext());
        assert_eq!(inflate(ZLIB_DYNAMIC_MIXED).unwrap(), mixed_plaintext());
    }

    #[test]
    fn a_code_deeper_than_deflate_allows_is_brought_within_the_limit() {
        // Fibonacci weights are the worst case for Huffman: every merge is
        // with the next symbol, so the tree is a chain and its depth is the
        // symbol count. Forty-seven symbols is as far as the weights go
        // inside a u32.
        let mut frequencies = vec![1u32, 1];
        while frequencies.len() < 47 {
            let next = frequencies[frequencies.len() - 1] + frequencies[frequencies.len() - 2];
            frequencies.push(next);
        }

        // The control: without a limit the code really is too deep, so the
        // test below is exercising the repair and not passing by accident.
        let natural = code_lengths(&frequencies, 40);
        assert!(
            natural.iter().copied().max().unwrap() > MAX_CODE_BITS as u8,
            "the input does not force an overlong code; the limit is untested"
        );

        let limited = code_lengths(&frequencies, MAX_CODE_BITS);
        assert!(limited.iter().copied().max().unwrap() <= MAX_CODE_BITS as u8);
        // Every symbol still has a code, and the code is still complete -
        // `code_lengths` asserts the Kraft sum itself, so reaching here at all
        // is part of the claim.
        assert!(limited.iter().all(|&length| length > 0));
    }

    #[test]
    fn a_block_whose_natural_code_is_too_deep_still_round_trips() {
        // The same shape, but end to end: tokens straight into the dynamic
        // encoder, so the frequencies survive to it instead of being turned
        // into matches by the tokenizer.
        let mut counts = vec![1usize, 1];
        while counts.len() < 17 {
            let next = counts[counts.len() - 1] + counts[counts.len() - 2];
            counts.push(next);
        }

        let mut expected = Vec::new();
        let mut tokens = Vec::new();
        for (symbol, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                expected.push(symbol as u8);
                tokens.push(Token::Literal(symbol as u8));
            }
        }

        let stream = encode_dynamic(&tokens);
        assert_eq!((stream[0] >> 1) & 3, 2, "this must be a dynamic block");
        assert_eq!(inflate(&stream).unwrap(), expected);
    }

    #[test]
    fn each_block_type_wins_where_it_should_and_none_of_them_costs_anything() {
        // Reading the header back is how the choice is observed: bit 0 is
        // BFINAL and bits 1-2 are BTYPE.
        let block_type = |stream: &[u8]| (stream[0] >> 1) & 3;

        // Texture-like: filtered scanlines of graded noise. A code built for
        // this data pays for describing itself many times over.
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let textured: Vec<u8> = (0..40_000)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                // Graded, not uniform: a handful of values dominate, which is
                // exactly what the fixed table has no way to exploit.
                (((state >> 33) as u8) & 0x3F) + 0x90
            })
            .collect();
        assert_eq!(
            block_type(&deflate(&textured)),
            2,
            "dynamic should win here"
        );

        // Short: the code description costs more than it can save.
        assert_eq!(block_type(&deflate(b"NEXORA")), 1, "fixed should win here");

        // Structureless: nothing to code and nothing to match either, so a
        // code description is pure overhead and so is a fixed table - which
        // would in fact come out at 63 274, larger than the input.
        let noise = unmatchable(60_000);
        assert_eq!(block_type(&deflate(&noise)), 0, "stored should win here");

        // And in no case is the answer worse than it was before dynamic
        // blocks existed, because the smallest of the three is what ships.
        for case in [textured.as_slice(), b"NEXORA".as_slice(), noise.as_slice()] {
            // Nothing dynamic blocks added can make any input larger than it
            // was before they existed: the smallest of the three ships.
            assert!(deflate(case).len() <= fixed_block(case).len());
            assert!(deflate(case).len() <= stored_blocks(case).len());
            assert_eq!(inflate(&deflate(case)).unwrap(), case);
        }
    }

    #[test]
    fn the_code_length_alphabet_survives_every_run_it_has_a_symbol_for() {
        // Symbol 17 covers 3..=10 zeros, 18 covers 11..=138, and 16 repeats a
        // non-zero length 3..=6 times. The boundaries are where an off-by-one
        // in the extra-bit arithmetic hides, and a wrong header makes the
        // whole block unreadable rather than slightly wrong.
        for run in [1usize, 2, 3, 6, 7, 10, 11, 137, 138, 139, 200, 300] {
            let mut lengths = vec![0u8; run];
            lengths.extend_from_slice(&[7, 7, 7, 7, 7, 7, 7, 7]);
            lengths.extend(std::iter::repeat_n(0u8, run));
            lengths.push(3);

            let sequence = run_length_encode(&lengths);
            let mut replayed: Vec<u8> = Vec::new();
            for &(symbol, extra, _) in &sequence {
                match symbol {
                    16 => {
                        let previous = *replayed.last().expect("16 repeats something");
                        replayed.extend(std::iter::repeat_n(previous, 3 + extra as usize));
                    }
                    17 => replayed.extend(std::iter::repeat_n(0u8, 3 + extra as usize)),
                    18 => replayed.extend(std::iter::repeat_n(0u8, 11 + extra as usize)),
                    value => replayed.push(value),
                }
            }
            assert_eq!(replayed, lengths, "run of {run} did not survive");
        }
    }

    #[test]
    fn a_damaged_or_unsupported_stream_is_refused_rather_than_guessed_at() {
        // Truncated mid-code.
        assert!(inflate(&ZLIB_LONG_RUN[..4]).is_err());
        // Empty: there is not even a block header.
        assert!(inflate(&[]).is_err());
        // BTYPE 10 is dynamic Huffman, and this one stops before its own
        // code description does.
        let err = inflate(&[0b101]).expect_err("the code description is not there");
        assert!(err.to_string().contains("ends"), "{err}");
        // A dynamic block claiming 288 literal codes, which cannot exist:
        // BFINAL, BTYPE 10, then five bits of HLIT all set.
        let err = inflate(&[0xFD, 0x00, 0x00]).expect_err("286 is the ceiling");
        assert!(err.to_string().contains("more codes than exist"), "{err}");
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
        // This test used to be handed a counter sequence, described as having
        // "no repeats inside the window". It has plenty: `index * K >> 24` is
        // an arithmetic progression modulo 256, so the matcher eats it and
        // 60 000 bytes come out at 2 948. Eight bits of entropy per byte says
        // nothing about whether a *match finder* can find anything, and the
        // test was passing on an assertion that had stopped meaning anything.
        // See `a_sequence_with_full_byte_entropy_can_still_be_all_matches`.
        //
        // These bytes are unmatchable as well as high-entropy, which is what
        // the fallback is for.
        let data = unmatchable(60_000);
        let compressed = deflate(&data);
        let stored = stored_blocks(&data);
        assert!(
            compressed.len() <= stored.len(),
            "compressed {} exceeds stored {}",
            compressed.len(),
            stored.len()
        );
        // And the fallback is doing real work here: fixed Huffman alone would
        // have made this *larger* than the input it was given.
        assert!(
            fixed_block(&data).len() > data.len(),
            "the pathological case this guards against is not being reached"
        );
        assert_eq!(inflate(&compressed).unwrap(), data);
    }

    #[test]
    fn a_sequence_with_full_byte_entropy_can_still_be_all_matches() {
        // Kept as a case in its own right, because it is the mistake above
        // written down: a flat byte histogram and a compressible file are
        // unrelated properties.
        let data: Vec<u8> = (0..60_000u32)
            .map(|index| (index.wrapping_mul(2_654_435_761) >> 24) as u8)
            .collect();
        let mut seen = [0u32; 256];
        for &byte in &data {
            seen[usize::from(byte)] += 1;
        }
        assert!(
            seen.iter().all(|&count| count > 0),
            "every byte value occurs"
        );

        let compressed = deflate(&data);
        assert!(
            compressed.len() * 10 < data.len(),
            "{} bytes is not the order of ten times smaller",
            compressed.len()
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
