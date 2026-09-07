//! The incremental half of `Snapshot + Journal → Recovery`.
//!
//! `NEXORA SAVE FORMAT AND COMPATIBILITY.md` defines recovery as a snapshot
//! plus the changes that happened after it. Phase 0 built the snapshot —
//! atomic, versioned, checksummed, quarantinable — and `DEBT-0001` recorded
//! what was missing: *"mudanças entre checkpoints são perdidas se o processo
//! cair."*
//!
//! # The failure this exists for
//!
//! A journal is only worth having if a crash **while appending to it** is
//! survivable. That is the ordinary case, not the exotic one: the process dies
//! with a half-written record at the end of the file.
//!
//! So every record is framed and checksummed independently:
//!
//! ```text
//! header:  magic | format | world id | snapshot bytes | snapshot crc
//! record:  length u32 | crc32 u32 | payload
//! record:  length u32 | crc32 u32 | payload
//! ...
//! ```
//!
//! # A torn tail and corruption are different answers
//!
//! Both leave a record that will not parse, and treating them the same is how a
//! recovery system either throws away good data or accepts bad data.
//!
//! | what the reader finds | what it means | what it does |
//! | --- | --- | --- |
//! | fewer bytes than the record claims | the process died mid-append | recover the prefix; **expected** |
//! | full payload, wrong checksum | storage returned different bytes than were written | recover the prefix; **loud** |
//!
//! The rule that separates them: a torn write **truncates**. It cannot produce
//! a complete record whose contents are wrong. So a full-length record failing
//! its checksum is not a crash — it is `NEXORA FAILURE AND RECOVERY
//! ARCHITECTURE.md`'s *"never hide a data-integrity failure"*, and it is
//! reported as [`Damage::Corruption`] with a recommendation to quarantine.
//!
//! Replay **stops at the first bad record either way** and never tries to
//! resynchronise past it. Skipping ahead to salvage later records would mean
//! guessing where the next frame starts, and a wrong guess feeds attacker- or
//! garbage-controlled bytes into a world as if they were edits. The same
//! document is explicit: *"prefer failing a bounded operation over corrupting
//! global state."*
//!
//! # A journal is bound to one snapshot
//!
//! Replaying a journal onto the wrong snapshot corrupts a world silently, which
//! is worse than losing the journal. The header names the snapshot it continues
//! from, and [`replay`] refuses anything else.
//!
//! # What this crate does not know
//!
//! What a record *means*. Payloads are opaque bytes, exactly as sections are in
//! [`SaveContainer`](crate::SaveContainer) — the systems that own the state
//! decide what goes inside.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::crc32;

/// Identifies the file as a NEXORA journal. `"NXJR"`.
const MAGIC: [u8; 4] = *b"NXJR";

/// Layout version of the journal framing itself.
const FORMAT_VERSION: u32 = 1;

/// Bytes of fixed header: magic, format, world id, snapshot length, snapshot crc.
const HEADER_BYTES: usize = 4 + 4 + 8 + 8 + 4;

/// Bytes of per-record frame: length and checksum.
const FRAME_BYTES: usize = 4 + 4;

/// Largest accepted single record.
///
/// A length field is the first thing a corrupt or hostile file gets to choose,
/// and an unbounded one is an allocation an attacker picks. 16 MiB is far above
/// any real edit batch and far below anything that matters as a memory budget.
pub const MAX_RECORD_BYTES: u32 = 16 * 1024 * 1024;

/// Which snapshot a journal continues from.
///
/// Replaying onto a different snapshot would apply edits to a world that never
/// had the state they assume, so this is checked before a single record is
/// read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotId {
    /// The world the snapshot belongs to.
    pub world_id: u64,
    /// Length of the encoded snapshot, in bytes.
    pub bytes: u64,
    /// CRC-32 of the encoded snapshot.
    pub crc: u32,
}

impl SnapshotId {
    /// Derive the id from a world id and the snapshot's encoded bytes.
    #[must_use]
    pub fn of(world_id: u64, snapshot: &[u8]) -> Self {
        Self {
            world_id,
            bytes: snapshot.len() as u64,
            crc: crc32(snapshot),
        }
    }
}

/// What was wrong with the journal, when something was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Damage {
    /// The file ended part-way through a record.
    ///
    /// The expected outcome of a crash during an append. Everything before it
    /// is intact and has been recovered.
    TornTail {
        /// Bytes after the last complete record.
        trailing_bytes: u64,
    },
    /// A complete record failed its checksum.
    ///
    /// Not a crash: a torn write truncates rather than rewriting contents. The
    /// bytes on disk are not the bytes that were written, so the file should be
    /// quarantined as evidence.
    Corruption {
        /// Byte offset of the record that failed.
        offset: u64,
        /// How many records were recovered before it.
        recovered: u64,
    },
}

impl Damage {
    /// Whether this is the benign, expected outcome of a crash.
    #[must_use]
    pub const fn is_expected_after_a_crash(self) -> bool {
        matches!(self, Self::TornTail { .. })
    }
}

/// The result of reading a journal back.
#[derive(Debug, Clone)]
pub struct Replay {
    /// Records recovered, in the order they were appended.
    pub records: Vec<Vec<u8>>,
    /// What stopped the read, if anything did.
    pub damage: Option<Damage>,
}

impl Replay {
    /// How many records were recovered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether nothing was recovered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Whether the journal was read to the end with nothing wrong.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.damage.is_none()
    }
}

/// An append-only log of changes made since a snapshot.
#[derive(Debug)]
pub struct Journal {
    file: File,
    path: PathBuf,
    base: SnapshotId,
    appended: u64,
    unsynced: u64,
}

impl Journal {
    /// Start a new journal for `base`, replacing any file already at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be created or the header cannot be
    /// written and flushed.
    pub fn create(path: &Path, base: SnapshotId) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|cause| io_error("create journal directory", parent, &cause))?;
            }
        }

        let mut file =
            File::create(path).map_err(|cause| io_error("create journal", path, &cause))?;
        file.write_all(&encode_header(base))
            .map_err(|cause| io_error("write journal header", path, &cause))?;
        // The header must be durable before any record claims to continue from
        // it; a journal whose header never reached disk is unreadable, and its
        // records would be unattributable to any snapshot.
        file.sync_all()
            .map_err(|cause| io_error("flush journal header", path, &cause))?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
            base,
            appended: 0,
            unsynced: 0,
        })
    }

    /// Reopen an existing journal to append to it.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is missing, its header is unreadable, or
    /// its header names a different snapshot than `base`.
    pub fn open_append(path: &Path, base: SnapshotId) -> Result<Self> {
        let mut header = [0u8; HEADER_BYTES];
        {
            let mut file =
                File::open(path).map_err(|cause| io_error("open journal", path, &cause))?;
            file.read_exact(&mut header)
                .map_err(|cause| io_error("read journal header", path, &cause))?;
        }
        let found = decode_header(&header)?;
        if found != base {
            return Err(mismatched_snapshot(path, base, found));
        }

        let file = OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|cause| io_error("reopen journal for append", path, &cause))?;

        Ok(Self {
            file,
            path: path.to_path_buf(),
            base,
            appended: 0,
            unsynced: 0,
        })
    }

    /// Append one record.
    ///
    /// The record is framed and checksummed, but **not** flushed — call
    /// [`sync`](Self::sync) to make it durable. Appending without syncing is
    /// the fast path; a crash then loses the unsynced tail, which the framing
    /// makes recoverable rather than fatal.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload exceeds [`MAX_RECORD_BYTES`] or the
    /// write fails.
    pub fn append(&mut self, payload: &[u8]) -> Result<()> {
        let length = u32::try_from(payload.len())
            .ok()
            .filter(|len| *len <= MAX_RECORD_BYTES);
        let Some(length) = length else {
            return Err(Error::new(
                Domain::Save,
                "journal",
                "journal record is larger than the maximum record size",
            )
            .with_recovery(Recovery::Reject)
            .with_context("bytes", payload.len().to_string())
            .with_context("maximum", MAX_RECORD_BYTES.to_string()));
        };

        let mut framed = Vec::with_capacity(FRAME_BYTES + payload.len());
        framed.extend_from_slice(&length.to_le_bytes());
        framed.extend_from_slice(&crc32(payload).to_le_bytes());
        framed.extend_from_slice(payload);

        // One write call for the whole frame. Splitting it would widen the
        // window in which a crash leaves a length with no payload behind it --
        // still recoverable, but needlessly more often.
        self.file
            .write_all(&framed)
            .map_err(|cause| io_error("append journal record", &self.path, &cause))?;
        self.appended += 1;
        self.unsynced += 1;
        Ok(())
    }

    /// Flush appended records to storage.
    ///
    /// # Errors
    ///
    /// Returns an error when the flush fails, which means the records are not
    /// durable and the caller must not report the operation as committed.
    pub fn sync(&mut self) -> Result<()> {
        // `sync_data` rather than `sync_all`: the file's length and contents
        // must be durable, its access time need not be.
        self.file
            .sync_data()
            .map_err(|cause| io_error("flush journal", &self.path, &cause))?;
        self.unsynced = 0;
        Ok(())
    }

    /// Append a record and make it durable in one step.
    ///
    /// # Errors
    ///
    /// Returns an error when either half fails.
    pub fn append_durable(&mut self, payload: &[u8]) -> Result<()> {
        self.append(payload)?;
        self.sync()
    }

    /// How many records this handle has appended.
    #[must_use]
    pub const fn appended(&self) -> u64 {
        self.appended
    }

    /// How many appended records have not been flushed yet.
    #[must_use]
    pub const fn unsynced(&self) -> u64 {
        self.unsynced
    }

    /// The snapshot this journal continues from.
    #[must_use]
    pub const fn base(&self) -> SnapshotId {
        self.base
    }

    /// Where the journal lives.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Read a journal back, recovering everything that is intact.
///
/// # Errors
///
/// Returns an error when the file cannot be read, its header is not a journal
/// header, or its header names a snapshot other than `base`. A damaged *body*
/// is not an error — it is reported in [`Replay::damage`] alongside the records
/// that survived.
pub fn replay(path: &Path, base: SnapshotId) -> Result<Replay> {
    let bytes = fs::read(path).map_err(|cause| io_error("read journal", path, &cause))?;
    replay_bytes(&bytes, base, Some(path))
}

/// Read a journal back from memory. Used by [`replay`] and by tests.
///
/// # Errors
///
/// As [`replay`].
pub fn replay_bytes(bytes: &[u8], base: SnapshotId, path: Option<&Path>) -> Result<Replay> {
    if bytes.len() < HEADER_BYTES {
        return Err(Error::new(
            Domain::Save,
            "journal",
            "journal is too short to contain a header",
        )
        .with_recovery(Recovery::Quarantine)
        .with_context("bytes", bytes.len().to_string()));
    }

    let found = decode_header(&bytes[..HEADER_BYTES])?;
    if found != base {
        return Err(mismatched_snapshot(
            path.unwrap_or_else(|| Path::new("<memory>")),
            base,
            found,
        ));
    }

    let mut records = Vec::new();
    let mut offset = HEADER_BYTES;

    loop {
        let remaining = bytes.len() - offset;
        if remaining == 0 {
            return Ok(Replay {
                records,
                damage: None,
            });
        }
        if remaining < FRAME_BYTES {
            // Not even a full frame header: the process died between records.
            return Ok(Replay {
                records,
                damage: Some(Damage::TornTail {
                    trailing_bytes: remaining as u64,
                }),
            });
        }

        let length = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("a four byte window is four bytes"),
        );
        let expected_crc = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .expect("a four byte window is four bytes"),
        );

        // A length beyond the cap cannot have been written by `append`, so the
        // frame header itself is not trustworthy. Treated as a torn tail rather
        // than as an allocation instruction.
        if length > MAX_RECORD_BYTES {
            return Ok(Replay {
                records,
                damage: Some(Damage::TornTail {
                    trailing_bytes: remaining as u64,
                }),
            });
        }

        let payload_start = offset + FRAME_BYTES;
        let payload_end = payload_start + length as usize;
        if payload_end > bytes.len() {
            // The record claims more bytes than the file holds: a truncated
            // write, which is what a crash looks like.
            return Ok(Replay {
                records,
                damage: Some(Damage::TornTail {
                    trailing_bytes: remaining as u64,
                }),
            });
        }

        let payload = &bytes[payload_start..payload_end];
        if crc32(payload) != expected_crc {
            // Full payload, wrong contents. A torn write truncates; it does not
            // rewrite. So this is storage returning different bytes than were
            // written, and it is never silent.
            let recovered = records.len() as u64;
            return Ok(Replay {
                records,
                damage: Some(Damage::Corruption {
                    offset: offset as u64,
                    recovered,
                }),
            });
        }

        records.push(payload.to_vec());
        offset = payload_end;
    }
}

fn encode_header(base: SnapshotId) -> Vec<u8> {
    let mut header = Vec::with_capacity(HEADER_BYTES);
    header.extend_from_slice(&MAGIC);
    header.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(&base.world_id.to_le_bytes());
    header.extend_from_slice(&base.bytes.to_le_bytes());
    header.extend_from_slice(&base.crc.to_le_bytes());
    header
}

fn decode_header(header: &[u8]) -> Result<SnapshotId> {
    if header.len() < HEADER_BYTES {
        return Err(bad_header("journal header is truncated"));
    }
    if header[..4] != MAGIC {
        return Err(bad_header("file is not a NEXORA journal"));
    }
    let format = u32::from_le_bytes(header[4..8].try_into().expect("four bytes"));
    if format != FORMAT_VERSION {
        return Err(bad_header("journal format version is not supported")
            .with_context("found", format.to_string())
            .with_context("supported", FORMAT_VERSION.to_string()));
    }
    Ok(SnapshotId {
        world_id: u64::from_le_bytes(header[8..16].try_into().expect("eight bytes")),
        bytes: u64::from_le_bytes(header[16..24].try_into().expect("eight bytes")),
        crc: u32::from_le_bytes(header[24..28].try_into().expect("four bytes")),
    })
}

fn bad_header(message: &'static str) -> Error {
    Error::new(Domain::Save, "journal", message).with_recovery(Recovery::Quarantine)
}

fn mismatched_snapshot(path: &Path, expected: SnapshotId, found: SnapshotId) -> Error {
    Error::new(
        Domain::Save,
        "journal",
        "journal continues from a different snapshot and must not be replayed",
    )
    .with_recovery(Recovery::Quarantine)
    .with_context("path", path.display().to_string())
    .with_context("expected_world", format!("{:#x}", expected.world_id))
    .with_context("found_world", format!("{:#x}", found.world_id))
    .with_context("expected_crc", format!("{:#010x}", expected.crc))
    .with_context("found_crc", format!("{:#010x}", found.crc))
}

fn io_error(operation: &'static str, path: &Path, cause: &std::io::Error) -> Error {
    Error::new(Domain::Save, "journal", operation)
        .with_recovery(Recovery::Retry)
        .with_context("path", path.display().to_string())
        .with_context("cause", cause.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> SnapshotId {
        SnapshotId::of(0xC0FF_EE00_1234_5678, b"a snapshot's encoded bytes")
    }

    /// Build a journal in memory, exactly as [`Journal::append`] would on disk.
    fn journal_bytes(base: SnapshotId, payloads: &[&[u8]]) -> Vec<u8> {
        let mut bytes = encode_header(base);
        for payload in payloads {
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&crc32(payload).to_le_bytes());
            bytes.extend_from_slice(payload);
        }
        bytes
    }

    #[test]
    fn an_empty_journal_replays_to_nothing_and_is_complete() {
        let replay = replay_bytes(&encode_header(base()), base(), None).expect("valid");
        assert!(replay.is_empty());
        assert!(replay.is_complete());
    }

    #[test]
    fn records_come_back_in_the_order_they_were_appended() {
        // Order is the whole contract: a journal replayed out of order applies
        // a later edit before an earlier one and produces a different world.
        let bytes = journal_bytes(base(), &[b"first", b"second", b"third"]);
        let replay = replay_bytes(&bytes, base(), None).expect("valid");
        assert!(replay.is_complete());
        assert_eq!(
            replay.records,
            vec![b"first".to_vec(), b"second".to_vec(), b"third".to_vec()]
        );
    }

    #[test]
    fn an_empty_payload_is_a_real_record() {
        // Zero-length is a legitimate record and must not be mistaken for the
        // end of the file.
        let bytes = journal_bytes(base(), &[b"", b"after"]);
        let replay = replay_bytes(&bytes, base(), None).expect("valid");
        assert!(replay.is_complete());
        assert_eq!(replay.records, vec![Vec::new(), b"after".to_vec()]);
    }

    // --- the crash cases ---------------------------------------------------

    #[test]
    fn a_crash_mid_payload_recovers_every_earlier_record() {
        let bytes = journal_bytes(base(), &[b"kept", b"also kept", b"lost to the crash"]);
        // Die three bytes into the last payload.
        let truncated = &bytes[..bytes.len() - 3];

        let replay = replay_bytes(truncated, base(), None).expect("a torn tail is not an error");
        assert_eq!(
            replay.records,
            vec![b"kept".to_vec(), b"also kept".to_vec()]
        );
        assert!(matches!(replay.damage, Some(Damage::TornTail { .. })));
        assert!(replay.damage.expect("damaged").is_expected_after_a_crash());
    }

    #[test]
    fn a_crash_inside_the_frame_header_recovers_every_earlier_record() {
        // The narrower window: dead after two bytes of the length field.
        let bytes = journal_bytes(base(), &[b"kept"]);
        let mut truncated = bytes.clone();
        truncated.extend_from_slice(&[0x04, 0x00]);

        let replay = replay_bytes(&truncated, base(), None).expect("a torn tail is not an error");
        assert_eq!(replay.records, vec![b"kept".to_vec()]);
        assert_eq!(replay.damage, Some(Damage::TornTail { trailing_bytes: 2 }));
    }

    #[test]
    fn a_crash_immediately_after_a_record_leaves_a_complete_journal() {
        let bytes = journal_bytes(base(), &[b"kept"]);
        let replay = replay_bytes(&bytes, base(), None).expect("valid");
        assert!(replay.is_complete(), "a clean boundary is not damage");
        assert_eq!(replay.len(), 1);
    }

    // --- corruption is a different answer ----------------------------------

    #[test]
    fn a_flipped_bit_in_a_complete_record_is_corruption_not_a_torn_tail() {
        // A torn write truncates; it cannot produce a full record with wrong
        // contents. So this is storage lying, and it must not be filed under
        // "expected after a crash".
        let mut bytes = journal_bytes(base(), &[b"first", b"second"]);
        let last = bytes.len() - 1;
        bytes[last] ^= 0b0000_0001;

        let replay =
            replay_bytes(&bytes, base(), None).expect("body damage is reported, not thrown");
        assert_eq!(replay.records, vec![b"first".to_vec()]);
        match replay.damage.expect("damaged") {
            Damage::Corruption { recovered, .. } => assert_eq!(recovered, 1),
            other => panic!("expected corruption, found {other:?}"),
        }
        assert!(!replay.damage.expect("damaged").is_expected_after_a_crash());
    }

    #[test]
    fn replay_stops_at_damage_and_does_not_resynchronise_past_it() {
        // Salvaging later records would mean guessing where the next frame
        // starts, and a wrong guess feeds garbage into a world as if it were an
        // edit. Better to lose the tail than to invent one.
        let mut bytes = journal_bytes(base(), &[b"good", b"damaged", b"would be good"]);
        // Corrupt the middle record's payload.
        let position = HEADER_BYTES + FRAME_BYTES + b"good".len() + FRAME_BYTES + 1;
        bytes[position] ^= 0xFF;

        let replay = replay_bytes(&bytes, base(), None).expect("reported");
        assert_eq!(replay.records, vec![b"good".to_vec()]);
        assert!(matches!(replay.damage, Some(Damage::Corruption { .. })));
    }

    // --- the wrong-snapshot trap -------------------------------------------

    #[test]
    fn a_journal_from_another_snapshot_is_refused_entirely() {
        // The dangerous case: these records would apply cleanly and produce a
        // world that never existed. Losing the journal is recoverable; a
        // silently wrong world is not.
        let bytes = journal_bytes(base(), &[b"an edit"]);
        let other = SnapshotId::of(base().world_id, b"a different snapshot");
        let error = replay_bytes(&bytes, other, None).expect_err("must refuse");
        assert!(error.to_string().contains("different snapshot"));
    }

    #[test]
    fn a_journal_from_another_world_is_refused_even_at_the_same_size() {
        let mine = SnapshotId::of(1, b"identical bytes");
        let theirs = SnapshotId::of(2, b"identical bytes");
        let bytes = journal_bytes(theirs, &[b"an edit"]);
        assert!(replay_bytes(&bytes, mine, None).is_err());
    }

    #[test]
    fn a_file_that_is_not_a_journal_is_refused() {
        let mut bytes = journal_bytes(base(), &[b"x"]);
        bytes[0] = b'X';
        let error = replay_bytes(&bytes, base(), None).expect_err("must refuse");
        assert!(error.to_string().contains("not a NEXORA journal"));
    }

    #[test]
    fn a_future_format_version_is_refused_rather_than_guessed() {
        let mut bytes = journal_bytes(base(), &[b"x"]);
        bytes[4..8].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        assert!(replay_bytes(&bytes, base(), None).is_err());
    }

    #[test]
    fn a_truncated_header_is_refused_rather_than_read_as_empty() {
        let bytes = journal_bytes(base(), &[]);
        assert!(replay_bytes(&bytes[..HEADER_BYTES - 1], base(), None).is_err());
    }

    #[test]
    fn an_absurd_length_field_does_not_become_an_allocation() {
        // The length is the first thing a corrupt file gets to choose.
        let mut bytes = encode_header(base());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());

        let replay = replay_bytes(&bytes, base(), None).expect("reported, not attempted");
        assert!(replay.is_empty());
        assert!(matches!(replay.damage, Some(Damage::TornTail { .. })));
    }

    // --- on disk -----------------------------------------------------------

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(label: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("nexora-journal-{label}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("scratch");
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_journal_round_trips_through_the_filesystem() {
        let scratch = Scratch::new("roundtrip");
        let path = scratch.file("world.nxjr");

        let mut journal = Journal::create(&path, base()).expect("created");
        journal.append_durable(b"one").expect("appended");
        journal.append_durable(b"two").expect("appended");
        assert_eq!(journal.appended(), 2);
        assert_eq!(journal.unsynced(), 0);
        drop(journal);

        let replay = replay(&path, base()).expect("read back");
        assert!(replay.is_complete());
        assert_eq!(replay.records, vec![b"one".to_vec(), b"two".to_vec()]);
    }

    #[test]
    fn reopening_appends_rather_than_truncating() {
        let scratch = Scratch::new("reopen");
        let path = scratch.file("world.nxjr");

        let mut journal = Journal::create(&path, base()).expect("created");
        journal.append_durable(b"before").expect("appended");
        drop(journal);

        let mut reopened = Journal::open_append(&path, base()).expect("reopened");
        reopened.append_durable(b"after").expect("appended");
        drop(reopened);

        let replay = replay(&path, base()).expect("read back");
        assert_eq!(replay.records, vec![b"before".to_vec(), b"after".to_vec()]);
    }

    #[test]
    fn reopening_against_the_wrong_snapshot_is_refused_before_a_single_append() {
        let scratch = Scratch::new("reopen-mismatch");
        let path = scratch.file("world.nxjr");

        let journal = Journal::create(&path, base()).expect("created");
        drop(journal);

        let other = SnapshotId::of(base().world_id, b"a different snapshot");
        assert!(Journal::open_append(&path, other).is_err());
    }

    #[test]
    fn a_truncated_file_on_disk_recovers_its_intact_prefix() {
        // The real crash, end to end: append, lose the tail, read it back.
        let scratch = Scratch::new("truncated");
        let path = scratch.file("world.nxjr");

        let mut journal = Journal::create(&path, base()).expect("created");
        journal.append_durable(b"survives").expect("appended");
        journal.append_durable(b"lost").expect("appended");
        drop(journal);

        let full = fs::read(&path).expect("read");
        fs::write(&path, &full[..full.len() - 2]).expect("truncate");

        let recovered = replay(&path, base()).expect("a torn tail is recoverable");
        assert_eq!(recovered.records, vec![b"survives".to_vec()]);
        assert!(recovered
            .damage
            .expect("damaged")
            .is_expected_after_a_crash());
    }

    #[test]
    fn a_record_larger_than_the_cap_is_refused_at_append() {
        let scratch = Scratch::new("oversize");
        let path = scratch.file("world.nxjr");
        let mut journal = Journal::create(&path, base()).expect("created");

        let oversize = vec![0u8; MAX_RECORD_BYTES as usize + 1];
        assert!(journal.append(&oversize).is_err());
        assert_eq!(journal.appended(), 0, "a refused append changes nothing");
    }

    #[test]
    fn unsynced_records_are_counted_until_they_are_flushed() {
        let scratch = Scratch::new("unsynced");
        let path = scratch.file("world.nxjr");
        let mut journal = Journal::create(&path, base()).expect("created");

        journal.append(b"one").expect("appended");
        journal.append(b"two").expect("appended");
        assert_eq!(journal.unsynced(), 2, "not durable yet");
        journal.sync().expect("flushed");
        assert_eq!(journal.unsynced(), 0);
    }
}
