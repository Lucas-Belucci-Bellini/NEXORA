//! Binary reading and writing primitives.
//!
//! A save file is untrusted input (`NEXORA SECURITY THREAT MODEL.md`), so every
//! read here is bounds-checked and returns an error rather than panicking. All
//! values are little-endian so a world written on one machine loads on another.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Longest byte string this codec will accept, as a denial-of-service guard.
///
/// A corrupt or hostile file can claim any length; without a ceiling, a
/// four-byte edit turns into a multi-gigabyte allocation.
pub const MAX_BYTES_LEN: usize = 64 * 1024 * 1024;

/// Appends little-endian values to a growable buffer.
#[derive(Debug, Default)]
pub struct Writer {
    buffer: Vec<u8>,
}

impl Writer {
    /// Create an empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a `u8`.
    pub fn u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    /// Append a `u16`.
    pub fn u16(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Append a `u32`.
    pub fn u32(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Append a `u64`.
    pub fn u64(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Append an `i64`.
    pub fn i64(&mut self, value: i64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Append raw bytes with no length prefix.
    pub fn raw(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Append a length-prefixed byte string.
    pub fn bytes(&mut self, bytes: &[u8]) {
        self.u64(bytes.len() as u64);
        self.buffer.extend_from_slice(bytes);
    }

    /// Append a length-prefixed UTF-8 string.
    pub fn string(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    /// The bytes written so far.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Consume the writer and return its buffer.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

/// Reads little-endian values from a byte slice, checking every access.
#[derive(Debug)]
pub struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    /// Wrap a byte slice.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    /// Current offset.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.position
    }

    /// Bytes not yet consumed.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    /// Whether every byte has been consumed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// Read `count` raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when fewer than `count` bytes remain.
    pub fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        if count > self.remaining() {
            return Err(truncated(self.position, count, self.remaining()));
        }
        let slice = &self.bytes[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    /// Read a `u8`.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated.
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    /// Read a `u16`.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated.
    pub fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a `u32`.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated.
    pub fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a `u64`.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated.
    pub fn u64(&mut self) -> Result<u64> {
        let bytes = self.take(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(array))
    }

    /// Read an `i64`.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated.
    pub fn i64(&mut self) -> Result<i64> {
        Ok(self.u64()? as i64)
    }

    /// Read a length-prefixed byte string.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated or the declared length
    /// exceeds [`MAX_BYTES_LEN`].
    pub fn bytes(&mut self) -> Result<&'a [u8]> {
        let length = self.u64()?;
        let length = usize::try_from(length)
            .map_err(|_| oversized(self.position, length))
            .and_then(|length| {
                if length > MAX_BYTES_LEN {
                    Err(oversized(self.position, length as u64))
                } else {
                    Ok(length)
                }
            })?;
        self.take(length)
    }

    /// Read a length-prefixed UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is truncated or not valid UTF-8.
    pub fn string(&mut self) -> Result<&'a str> {
        let bytes = self.bytes()?;
        core::str::from_utf8(bytes).map_err(|cause| {
            Error::new(Domain::Save, "codec", "string field is not valid UTF-8")
                .with_recovery(Recovery::Quarantine)
                .with_context("offset", self.position.to_string())
                .with_context("cause", cause.to_string())
        })
    }

    /// Fail unless every byte has been consumed.
    ///
    /// # Errors
    ///
    /// Returns an error when trailing bytes remain. Extra data means the file
    /// is not what this decoder thinks it is, which is worth refusing.
    pub fn expect_exhausted(&self) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        Err(Error::new(
            Domain::Save,
            "codec",
            "unexpected trailing bytes after the final record",
        )
        .with_recovery(Recovery::Quarantine)
        .with_context("trailing", self.remaining().to_string()))
    }
}

fn truncated(offset: usize, wanted: usize, available: usize) -> Error {
    Error::new(
        Domain::Save,
        "codec",
        "input ended before the record was complete",
    )
    .with_recovery(Recovery::Quarantine)
    .with_context("offset", offset.to_string())
    .with_context("wanted", wanted.to_string())
    .with_context("available", available.to_string())
}

fn oversized(offset: usize, length: u64) -> Error {
    Error::new(
        Domain::Save,
        "codec",
        "declared length exceeds the maximum accepted size",
    )
    .with_recovery(Recovery::Quarantine)
    .with_context("offset", offset.to_string())
    .with_context("declared", length.to_string())
    .with_context("max", MAX_BYTES_LEN.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scalar_round_trips() {
        let mut writer = Writer::new();
        writer.u8(0xAB);
        writer.u16(0xBEEF);
        writer.u32(0xDEAD_BEEF);
        writer.u64(0x0123_4567_89AB_CDEF);
        writer.i64(-42);
        writer.string("nexora:block/stone");
        writer.bytes(&[1, 2, 3]);

        let encoded = writer.finish();
        let mut reader = Reader::new(&encoded);
        assert_eq!(reader.u8().unwrap(), 0xAB);
        assert_eq!(reader.u16().unwrap(), 0xBEEF);
        assert_eq!(reader.u32().unwrap(), 0xDEAD_BEEF);
        assert_eq!(reader.u64().unwrap(), 0x0123_4567_89AB_CDEF);
        assert_eq!(reader.i64().unwrap(), -42);
        assert_eq!(reader.string().unwrap(), "nexora:block/stone");
        assert_eq!(reader.bytes().unwrap(), &[1, 2, 3]);
        reader.expect_exhausted().unwrap();
    }

    #[test]
    fn negative_integers_survive_the_round_trip() {
        let mut writer = Writer::new();
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            writer.i64(value);
        }
        let encoded = writer.finish();
        let mut reader = Reader::new(&encoded);
        for expected in [i64::MIN, -1, 0, 1, i64::MAX] {
            assert_eq!(reader.i64().unwrap(), expected);
        }
    }

    #[test]
    fn truncated_input_errors_instead_of_panicking() {
        let mut writer = Writer::new();
        writer.u64(0xFFFF_FFFF_FFFF_FFFF);
        let encoded = writer.finish();

        // Every prefix of a valid record must be refused cleanly.
        for cut in 0..encoded.len() {
            let mut reader = Reader::new(&encoded[..cut]);
            let err = reader.u64().expect_err("a short read must error");
            assert_eq!(err.recovery(), Recovery::Quarantine);
        }
    }

    #[test]
    fn an_absurd_declared_length_is_refused() {
        // A four-byte edit to a length prefix must not become a huge allocation.
        let mut writer = Writer::new();
        writer.u64(u64::MAX);
        let encoded = writer.finish();

        let mut reader = Reader::new(&encoded);
        let err = reader
            .bytes()
            .expect_err("an oversized length must be refused");
        assert!(err.to_string().contains("exceeds the maximum"), "{err}");
    }

    #[test]
    fn invalid_utf8_is_refused() {
        let mut writer = Writer::new();
        writer.bytes(&[0xFF, 0xFE, 0xFD]);
        let encoded = writer.finish();

        let mut reader = Reader::new(&encoded);
        assert!(reader.string().is_err());
    }

    #[test]
    fn trailing_bytes_are_reported() {
        let mut writer = Writer::new();
        writer.u32(7);
        writer.raw(&[0xAA]);
        let encoded = writer.finish();

        let mut reader = Reader::new(&encoded);
        assert_eq!(reader.u32().unwrap(), 7);
        assert!(reader.expect_exhausted().is_err());
    }
}
