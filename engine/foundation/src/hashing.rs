//! Deterministic hashing primitives.
//!
//! The standard library's `DefaultHasher` is explicitly *not* stable across
//! releases, so it cannot be used for anything that is written to disk, sent
//! over the network, or compared between a client and a server. Registry
//! fingerprints (`Registry System.md` §38) and save integrity checks
//! (`NEXORA SAVE FORMAT AND COMPATIBILITY.md`) both need values that are
//! identical everywhere, forever. These two functions provide that.

/// FNV-1a offset basis for the 64-bit variant.
const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a prime for the 64-bit variant.
const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;

/// A stable 64-bit content hash.
///
/// Used for identity fingerprints, not for integrity: it detects accidental
/// divergence, not deliberate tampering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fnv1a64(u64);

impl Fnv1a64 {
    /// Start a new hash.
    #[must_use]
    pub const fn new() -> Self {
        Self(FNV_OFFSET_BASIS_64)
    }

    /// Absorb raw bytes.
    pub fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(FNV_PRIME_64);
        }
    }

    /// Absorb a `u64` in little-endian order.
    pub fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    /// Absorb a length-prefixed string.
    ///
    /// The length prefix is what stops `"ab" + "c"` from colliding with
    /// `"a" + "bc"` when several strings are folded into one hash.
    pub fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write(value.as_bytes());
    }

    /// Finish and return the digest.
    #[must_use]
    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a byte slice with FNV-1a 64.
#[must_use]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hasher = Fnv1a64::new();
    hasher.write(bytes);
    hasher.finish()
}

/// CRC-32 (IEEE 802.3, reflected, polynomial `0xEDB88320`).
///
/// Used for save-section integrity. CRC catches the truncation and bit-rot that
/// a partially written file produces, which is exactly the failure the atomic
/// write path is defending against.
#[must_use]
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            // Branchless reflected CRC step.
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_published_vectors() {
        // Reference vectors from the FNV specification.
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn crc32_matches_published_vectors() {
        // Standard CRC-32/ISO-HDLC check values.
        assert_eq!(crc32(b""), 0x0000_0000);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(
            crc32(b"The quick brown fox jumps over the lazy dog"),
            0x414F_A339
        );
    }

    #[test]
    fn length_prefixing_prevents_concatenation_collisions() {
        let mut a = Fnv1a64::new();
        a.write_str("ab");
        a.write_str("c");

        let mut b = Fnv1a64::new();
        b.write_str("a");
        b.write_str("bc");

        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn crc32_detects_single_bit_flips() {
        let data = b"nexora chunk section payload".to_vec();
        let baseline = crc32(&data);
        for bit in 0..data.len() * 8 {
            let mut mutated = data.clone();
            mutated[bit / 8] ^= 1 << (bit % 8);
            assert_ne!(crc32(&mutated), baseline, "bit {bit} went undetected");
        }
    }
}
