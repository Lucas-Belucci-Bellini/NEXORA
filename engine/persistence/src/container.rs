//! The versioned save container.
//!
//! Implements `NEXORA SAVE FORMAT AND COMPATIBILITY.md`. Four of its
//! requirements are load-bearing here:
//!
//! * **Versioning.** Every container records the engine, save-format, world
//!   schema and content versions that produced it.
//! * **Atomicity.** "Never replace a valid save with partially written data."
//!   Writes go to a temporary file, are flushed, decoded back, and only then
//!   renamed over the original.
//! * **Corruption detection.** Each section carries a CRC; the header carries
//!   its own. A damaged section is detected before the caller sees the bytes.
//! * **Quarantine over overwrite.** A file that fails to load is moved aside
//!   with a diagnostic, not silently replaced.
//!
//! The snapshot half of "snapshot + journal" is implemented; the incremental
//! journal is deferred and tracked in the technical debt register.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use nexora_foundation::deflate::{deflate, inflate};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::crc32;
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::{
    require_readable_save_format, Compatibility, EngineVersion, SaveFormatVersion, VersionSet,
    WorldSchemaVersion,
};

use crate::codec::{Reader, Writer};

/// Magic bytes at the start of every container.
pub const MAGIC: &[u8; 4] = b"NXSV";

/// Magic bytes after the final section, proving the file is complete.
pub const TRAILER: &[u8; 4] = b"NXEN";

/// Largest number of sections a container may declare.
pub const MAX_SECTIONS: u32 = 4_096;

/// A named, checksummed blob inside a container.
///
/// Sections are addressed by [`Identifier`] so that a mod can add its own state
/// under its own namespace without colliding with the engine's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveContainer {
    versions: VersionSet,
    sections: BTreeMap<Identifier, Vec<u8>>,
}

impl SaveContainer {
    /// Create an empty container stamped with this build's versions.
    #[must_use]
    pub fn new() -> Self {
        Self {
            versions: VersionSet::current(),
            sections: BTreeMap::new(),
        }
    }

    /// The versions recorded in this container.
    #[must_use]
    pub const fn versions(&self) -> VersionSet {
        self.versions
    }

    /// Add or replace a section.
    pub fn put(&mut self, name: Identifier, data: Vec<u8>) {
        self.sections.insert(name, data);
    }

    /// Read a section, if present.
    #[must_use]
    pub fn get(&self, name: &Identifier) -> Option<&[u8]> {
        self.sections.get(name).map(Vec::as_slice)
    }

    /// Read a section, failing when it is absent.
    ///
    /// # Errors
    ///
    /// Returns an error naming the missing section.
    pub fn require(&self, name: &Identifier) -> Result<&[u8]> {
        self.get(name).ok_or_else(|| {
            Error::new(
                Domain::Save,
                "save-container",
                "the save is missing a required section",
            )
            .with_recovery(Recovery::Quarantine)
            .with_context("section", name.to_string())
        })
    }

    /// Every section name, sorted.
    #[must_use]
    pub fn section_names(&self) -> Vec<Identifier> {
        self.sections.keys().cloned().collect()
    }

    /// How many sections the container holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// Whether the container holds no sections.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    /// Serialize the container.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut header = Writer::new();
        header.raw(MAGIC);
        header.u32(self.versions.save_format.0);
        header.u16(self.versions.engine.major);
        header.u16(self.versions.engine.minor);
        header.u16(self.versions.engine.patch);
        header.u32(self.versions.world_schema.0);
        header.u32(self.versions.registry.0);
        header.u32(self.versions.content.0);
        header.u32(self.sections.len() as u32);

        let mut out = Writer::new();
        out.raw(header.as_slice());
        // The header carries its own checksum so a damaged header is caught
        // before its section count is trusted to size an allocation.
        out.u32(crc32(header.as_slice()));

        for (name, data) in &self.sections {
            let rendered = name.to_string();
            let (coding, stored) = pack(data);
            let raw_len = data.len() as u32;
            out.string(&rendered);
            // The checksum covers the name as well as the payload. Covering only
            // the payload leaves the name unprotected, and a single flipped bit
            // in a section name silently renames the section instead of being
            // reported as damage.
            //
            // It covers the bytes **as stored**, not as they were handed in:
            // the point of the checksum is to catch damage to the file, and a
            // damaged compressed payload has to be caught *before* anything
            // tries to decompress it.
            out.u32(frame_crc(&rendered, coding, raw_len, &stored));
            out.u8(coding as u8);
            // The uncompressed length, so the reader knows what it should have
            // got. A stream that inflates to a different size is damage the
            // checksum could not see — it would have to be damage that is still
            // a valid deflate stream, which is rare and not impossible.
            out.u32(raw_len);
            out.bytes(&stored);
        }
        out.raw(TRAILER);

        // A checksum over everything written so far. The per-section checksums
        // say *which* section is damaged; this one guarantees that no single
        // byte anywhere in the file - framing and trailer included - can change
        // without being noticed.
        let file_crc = crc32(out.as_slice());
        out.u32(file_crc);
        out.finish()
    }

    /// Parse a container, verifying every checksum.
    ///
    /// # Errors
    ///
    /// Returns an error when the magic, header checksum, section checksum,
    /// trailer, or version gate rejects the input. Every such error carries
    /// [`Recovery::Quarantine`] or [`Recovery::Reject`], never a silent default.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        // Verify the whole-file checksum before interpreting any field, so that
        // no damaged value is ever acted upon - not a length, not a version.
        let body_len = bytes
            .len()
            .checked_sub(4)
            .ok_or_else(|| corrupt("file is too short to be a NEXORA save"))?;
        let declared_file_crc = u32::from_le_bytes([
            bytes[body_len],
            bytes[body_len + 1],
            bytes[body_len + 2],
            bytes[body_len + 3],
        ]);
        if crc32(&bytes[..body_len]) != declared_file_crc {
            return Err(corrupt("save failed its whole-file checksum"));
        }

        let body = &bytes[..body_len];
        let mut reader = Reader::new(body);

        if reader.take(MAGIC.len())? != MAGIC {
            return Err(corrupt("file does not start with the NEXORA save magic"));
        }

        let save_format = SaveFormatVersion(reader.u32()?);
        let engine = EngineVersion::new(reader.u16()?, reader.u16()?, reader.u16()?);
        let world_schema = WorldSchemaVersion(reader.u32()?);
        let registry = nexora_foundation::version::RegistryVersion(reader.u32()?);
        let content = nexora_foundation::version::ContentVersion(reader.u32()?);
        let section_count = reader.u32()?;

        let header_end = reader.position();
        let declared_header_crc = reader.u32()?;
        if crc32(&body[..header_end]) != declared_header_crc {
            return Err(corrupt("save header failed its checksum"));
        }

        // Only now is the version gate consulted: a corrupt header could claim
        // any version at all, so integrity is established first.
        require_readable_save_format(save_format)?;

        if section_count > MAX_SECTIONS {
            return Err(
                corrupt("save declares more sections than the format allows")
                    .with_context("declared", section_count.to_string())
                    .with_context("max", MAX_SECTIONS.to_string()),
            );
        }

        let mut sections = BTreeMap::new();
        for index in 0..section_count {
            let rendered = reader.string()?;
            let name = Identifier::parse(rendered).map_err(|cause| {
                corrupt("save section name is not a valid identifier")
                    .with_context("index", index.to_string())
                    .with_source(cause)
            })?;
            let declared_crc = reader.u32()?;
            // Format 1 stored every section raw. Reading one is still supported
            // — `MIN_SUPPORTED_SAVE_FORMAT` says so — so the coding byte and the
            // length are read only from the format that writes them.
            let framed = save_format >= SAVE_FORMAT_COMPRESSED;
            let (coding_tag, raw_len) = if framed {
                (reader.u8()?, reader.u32()?)
            } else {
                (Coding::Stored as u8, 0)
            };
            let stored = reader.bytes()?;
            // Verified before the coding byte is even interpreted, so a
            // damaged frame is reported as damage rather than acted on.
            let computed = if framed {
                crc32(&{
                    let mut combined = Vec::with_capacity(rendered.len() + stored.len() + 5);
                    combined.extend_from_slice(rendered.as_bytes());
                    combined.push(coding_tag);
                    combined.extend_from_slice(&raw_len.to_le_bytes());
                    combined.extend_from_slice(stored);
                    combined
                })
            } else {
                section_crc(rendered, stored)
            };
            if computed != declared_crc {
                return Err(corrupt("save section failed its checksum")
                    .with_context("section", name.to_string()));
            }
            let coding = Coding::decode(coding_tag, &name)?;
            let expected_len = framed.then_some(raw_len as usize);
            let data = unpack(coding, stored, expected_len, &name)?;
            if sections.insert(name.clone(), data).is_some() {
                return Err(corrupt("save contains the same section twice")
                    .with_context("section", name.to_string()));
            }
        }

        if reader.take(TRAILER.len())? != TRAILER {
            return Err(corrupt(
                "save is missing its end marker; the write was incomplete",
            ));
        }
        reader.expect_exhausted()?;

        Ok(Self {
            versions: VersionSet {
                engine,
                save_format,
                world_schema,
                registry,
                content,
            },
            sections,
        })
    }

    /// How this build may treat the container's format version.
    ///
    /// # Errors
    ///
    /// Returns an error when the container was written by a newer engine.
    pub fn compatibility(&self) -> Result<Compatibility> {
        require_readable_save_format(self.versions.save_format)
    }

    /// Write the container to `path`, replacing it only once the new file is
    /// known to be complete and readable.
    ///
    /// # Errors
    ///
    /// Returns an error when any filesystem step fails, or when the freshly
    /// written bytes do not decode. The original file is left untouched unless
    /// the replacement verified successfully.
    pub fn write_atomic(&self, path: &Path) -> Result<()> {
        let encoded = self.encode();
        let temporary = temporary_path(path);

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|cause| io_error("create save directory", parent, &cause))?;
            }
        }

        {
            let mut file = fs::File::create(&temporary)
                .map_err(|cause| io_error("create temporary save", &temporary, &cause))?;
            file.write_all(&encoded)
                .map_err(|cause| io_error("write temporary save", &temporary, &cause))?;
            // Flush to disk before the rename, so a crash cannot leave the new
            // name pointing at data that never reached storage.
            file.sync_all()
                .map_err(|cause| io_error("flush temporary save", &temporary, &cause))?;
        }

        // Read the file back and decode it. Verifying before the rename is what
        // makes "never replace a valid save with bad data" true in practice
        // rather than merely intended.
        let written = fs::read(&temporary)
            .map_err(|cause| io_error("verify temporary save", &temporary, &cause))?;
        if let Err(cause) = Self::decode(&written) {
            let _ = fs::remove_file(&temporary);
            return Err(Error::new(
                Domain::Save,
                "save-container",
                "freshly written save did not verify; the previous save was left intact",
            )
            .with_recovery(Recovery::Retry)
            .with_context("path", path.display().to_string())
            .with_source(cause));
        }

        fs::rename(&temporary, path).map_err(|cause| io_error("commit save", path, &cause))?;
        Ok(())
    }

    /// Read and verify a container from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or fails verification.
    pub fn read(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).map_err(|cause| io_error("read save", path, &cause))?;
        Self::decode(&bytes).map_err(|cause| {
            Error::new(
                Domain::Save,
                "save-container",
                "save file failed verification",
            )
            .with_recovery(Recovery::Quarantine)
            .with_context("path", path.display().to_string())
            .with_source(cause)
        })
    }
}

impl Default for SaveContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Move a damaged save aside instead of deleting or overwriting it.
///
/// `NEXORA SAVE FORMAT AND COMPATIBILITY.md` requires detect, quarantine,
/// recover, report. A quarantined file is still the player's world and may be
/// recoverable by a later engine version or by hand.
///
/// # Errors
///
/// Returns an error when the file cannot be moved.
pub fn quarantine(path: &Path) -> Result<PathBuf> {
    let mut target = path.to_path_buf();
    let name = path
        .file_name()
        .map_or_else(|| "save".to_owned(), |n| n.to_string_lossy().into_owned());

    // Never overwrite an earlier quarantine: each incident is its own evidence.
    for attempt in 0..1_000u32 {
        let candidate = path.with_file_name(format!("{name}.quarantine.{attempt}"));
        if !candidate.exists() {
            target = candidate;
            break;
        }
    }

    fs::rename(path, &target).map_err(|cause| io_error("quarantine save", path, &cause))?;
    Ok(target)
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "save".to_owned(), |n| n.to_string_lossy().into_owned());
    path.with_file_name(format!("{name}.tmp"))
}

/// Checksum of one section, covering its name and its payload together.
/// The first save format that frames each section with how it is coded.
///
/// Below this, every section is stored raw and there is no coding byte to read.
const SAVE_FORMAT_COMPRESSED: SaveFormatVersion = SaveFormatVersion(2);

/// How one section's bytes are laid down on disk.
///
/// `DEBT-0003` asked for per-section compression "with the algorithm recorded
/// in the header, to allow a versioned swap". This is that record. It sits in
/// the container rather than in `nexora_world::persist`, where the entry
/// expected it, because the seam is the same for every section: chunks today,
/// entities and whatever else tomorrow, all framed once instead of each
/// producer deciding for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Coding {
    /// The bytes are the payload.
    Stored = 0,
    /// RFC 1951, as produced by [`nexora_foundation::deflate`].
    Deflate = 1,
}

impl Coding {
    fn decode(tag: u8, section: &Identifier) -> Result<Self> {
        match tag {
            0 => Ok(Self::Stored),
            1 => Ok(Self::Deflate),
            // A coding this build does not know is not a guess to make. The
            // save is readable in the sense that its checksums pass, and
            // unreadable in the sense that matters.
            other => Err(
                corrupt("save section uses a coding this build does not know")
                    .with_context("section", section.to_string())
                    .with_context("coding", other.to_string()),
            ),
        }
    }
}

/// Choose how to lay a section down, and lay it down.
///
/// Compression is attempted and then **kept only if it won**. A section of
/// high-entropy bytes deflates to slightly more than it started with, and
/// writing that would make the format worse at exactly the inputs it is
/// already worst at. `Coding::Stored` is not a fallback for failure — there is
/// no failure path — it is the answer when compressing did not pay.
fn pack(data: &[u8]) -> (Coding, Vec<u8>) {
    if data.is_empty() {
        return (Coding::Stored, Vec::new());
    }
    let deflated = deflate(data);
    if deflated.len() < data.len() {
        (Coding::Deflate, deflated)
    } else {
        (Coding::Stored, data.to_vec())
    }
}

/// Recover a section's bytes from how they were stored.
fn unpack(
    coding: Coding,
    stored: &[u8],
    expected_len: Option<usize>,
    section: &Identifier,
) -> Result<Vec<u8>> {
    let data = match coding {
        Coding::Stored => stored.to_vec(),
        Coding::Deflate => inflate(stored).map_err(|cause| {
            corrupt("save section did not decompress")
                .with_context("section", section.to_string())
                .with_source(cause)
        })?,
    };
    if let Some(expected) = expected_len {
        if data.len() != expected {
            return Err(corrupt("save section decompressed to the wrong size")
                .with_context("section", section.to_string())
                .with_context("expected", expected.to_string())
                .with_context("found", data.len().to_string()));
        }
    }
    Ok(data)
}

fn section_crc(name: &str, data: &[u8]) -> u32 {
    let mut combined = Vec::with_capacity(name.len() + data.len());
    combined.extend_from_slice(name.as_bytes());
    combined.extend_from_slice(data);
    crc32(&combined)
}

/// The checksum for one format-2 section frame.
///
/// Covers the coding and the uncompressed length as well as the name and the
/// stored bytes, for the reason the name is covered: a flipped bit in a field
/// that is not checksummed does not look like damage, it looks like a
/// different — and wrong — instruction. A coding byte that flips turns a
/// deflate stream into a raw payload; a length that flips turns a good section
/// into a refusal that blames the wrong thing. Both are now damage, reported
/// against the section they belong to.
fn frame_crc(name: &str, coding: Coding, raw_len: u32, stored: &[u8]) -> u32 {
    let mut combined = Vec::with_capacity(name.len() + stored.len() + 5);
    combined.extend_from_slice(name.as_bytes());
    combined.push(coding as u8);
    combined.extend_from_slice(&raw_len.to_le_bytes());
    combined.extend_from_slice(stored);
    crc32(&combined)
}

fn corrupt(message: &'static str) -> Error {
    Error::new(Domain::Save, "save-container", message).with_recovery(Recovery::Quarantine)
}

fn io_error(action: &'static str, path: &Path, cause: &std::io::Error) -> Error {
    Error::new(
        Domain::Save,
        "save-container",
        "a save filesystem operation failed",
    )
    .with_recovery(Recovery::Retry)
    .with_context("action", action)
    .with_context("path", path.display().to_string())
    .with_context("cause", cause.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("valid identifier")
    }

    fn sample() -> SaveContainer {
        let mut container = SaveContainer::new();
        container.put(id("nexora:save/world_header"), b"header bytes".to_vec());
        container.put(id("nexora:save/chunks"), vec![7u8; 1024]);
        container.put(id("example:save/mod_state"), b"mod bytes".to_vec());
        container
    }

    /// Build a container frame by hand at a chosen format version, so the
    /// reader can be tested against a layout this build no longer writes.
    fn framed(save_format: u32, sections: &[(&str, &[u8])]) -> Vec<u8> {
        let mut header = Writer::new();
        header.raw(MAGIC);
        header.u32(save_format);
        header.u16(0);
        header.u16(0);
        header.u16(0);
        header.u32(1);
        header.u32(1);
        header.u32(1);
        header.u32(sections.len() as u32);

        let mut out = Writer::new();
        out.raw(header.as_slice());
        out.u32(crc32(header.as_slice()));
        for (name, data) in sections {
            out.string(name);
            out.u32(section_crc(name, data));
            if save_format >= SAVE_FORMAT_COMPRESSED.0 {
                out.u8(Coding::Stored as u8);
                out.u32(data.len() as u32);
            }
            out.bytes(data);
        }
        out.raw(TRAILER);
        let body = out.as_slice().to_vec();
        out.u32(crc32(&body));
        out.finish()
    }

    /// `DEBT-0003`. Voxel words are the most compressible thing in the file and
    /// were being written raw.
    #[test]
    fn a_compressible_section_round_trips_and_the_file_gets_smaller() {
        // Not `vec![7; n]`: a single repeated byte is the easiest input there
        // is, and would flatter the result. This is a pattern with structure,
        // which is what a chunk actually looks like.
        let payload: Vec<u8> = (0..8_192u32).map(|i| ((i / 37) % 11) as u8).collect();
        let mut container = SaveContainer::new();
        container.put(id("nexora:save/chunks"), payload.clone());

        let encoded = container.encode();
        assert!(
            encoded.len() < payload.len() / 2,
            "8 KiB of structured bytes framed into {} — that is not compression",
            encoded.len()
        );

        let back = SaveContainer::decode(&encoded).expect("decodes");
        assert_eq!(
            back.get(&id("nexora:save/chunks")),
            Some(payload.as_slice()),
            "what comes back has to be exactly what went in"
        );
        assert_eq!(back.versions().save_format, SAVE_FORMAT_COMPRESSED);
    }

    /// Compression is kept only when it wins. Deflating high-entropy bytes
    /// costs a few more than it started with, and writing that would make the
    /// format worse at exactly the inputs it is already worst at.
    #[test]
    fn a_section_that_does_not_shrink_is_stored_rather_than_grown() {
        let mut rng = nexora_foundation::rng::Rng::from_seed(0x9E37_79B9_7F4A_7C15);
        let noise: Vec<u8> = (0..4_096).map(|_| (rng.next_u64() >> 24) as u8).collect();

        let (coding, stored) = pack(&noise);
        assert_eq!(coding, Coding::Stored, "random bytes must not be deflated");
        assert_eq!(stored, noise);

        let mut container = SaveContainer::new();
        container.put(id("nexora:save/chunks"), noise.clone());
        let back = SaveContainer::decode(&container.encode()).expect("decodes");
        assert_eq!(back.get(&id("nexora:save/chunks")), Some(noise.as_slice()));
    }

    /// An empty section has nothing to compress and must not acquire a deflate
    /// stream's fixed overhead on the way to disk.
    #[test]
    fn an_empty_section_stays_empty() {
        let (coding, stored) = pack(&[]);
        assert_eq!((coding, stored.len()), (Coding::Stored, 0));
    }

    /// `MIN_SUPPORTED_SAVE_FORMAT` says format 1 is still readable, and this is
    /// what that costs: the coding byte and the length are read only from the
    /// format that writes them. A real format-1 world was decoded by this build
    /// while the change was made; this keeps that true without a fixture file.
    #[test]
    fn a_format_one_save_written_before_coding_existed_still_decodes() {
        let bytes = framed(1, &[("nexora:save/chunks", b"raw payload, no coding byte")]);
        let container = SaveContainer::decode(&bytes).expect("format 1 must still decode");
        assert_eq!(container.versions().save_format, SaveFormatVersion(1));
        assert_eq!(
            container.get(&id("nexora:save/chunks")),
            Some(b"raw payload, no coding byte".as_slice())
        );
    }

    /// The checksum covers the bytes **as stored**, and is verified before
    /// anything tries to inflate them. Handing damaged bytes to a decompressor
    /// is how a corrupt file becomes a crash instead of a diagnosis.
    #[test]
    fn damage_to_a_compressed_section_is_caught_before_it_is_inflated() {
        let payload: Vec<u8> = (0..4_096u32).map(|i| ((i / 13) % 7) as u8).collect();
        let mut container = SaveContainer::new();
        container.put(id("nexora:save/chunks"), payload.clone());
        let mut bytes = container.encode();

        // Find the deflate stream rather than guessing an offset. A first
        // draft flipped "the middle byte" and hit the coding byte instead —
        // caught, but by the wrong guard, and the test would have passed while
        // proving something else.
        let (_, stream) = pack(&payload);
        let at = bytes
            .windows(stream.len())
            .position(|window| window == stream.as_slice())
            .expect("the stored stream is in the file")
            + stream.len() / 2;
        bytes[at] ^= 0b0010_0000;
        // Repair the whole-file checksum so the *section* checksum is the one
        // doing the catching, which is the one that names which section failed.
        let body_len = bytes.len() - 4;
        let repaired = crc32(&bytes[..body_len]).to_le_bytes();
        bytes[body_len..].copy_from_slice(&repaired);

        let error = SaveContainer::decode(&bytes).expect_err("damage must be reported");
        assert!(
            error.to_string().contains("checksum"),
            "expected a checksum failure, got: {error}"
        );
    }

    /// The coding byte is inside the section checksum, so flipping it is damage
    /// rather than a different instruction. Without that, a flipped bit turns a
    /// deflate stream into "this is raw" and the section decodes to garbage.
    #[test]
    fn damage_to_the_coding_byte_is_damage_and_not_a_new_instruction() {
        let payload: Vec<u8> = (0..4_096u32).map(|i| ((i / 13) % 7) as u8).collect();
        let mut container = SaveContainer::new();
        container.put(id("nexora:save/chunks"), payload.clone());
        let mut bytes = container.encode();

        let (_, stream) = pack(&payload);
        let stream_at = bytes
            .windows(stream.len())
            .position(|window| window == stream.as_slice())
            .expect("the stored stream is in the file");
        // The frame is name, crc, coding, raw length, then the payload behind
        // its own `u64` length prefix — so the coding byte sits eight bytes for
        // that prefix, four for the raw length, and one for itself ahead of the
        // stream.
        let coding_at = stream_at - 8 - 4 - 1;
        assert_eq!(
            bytes[coding_at],
            Coding::Deflate as u8,
            "aimed at the wrong byte"
        );
        bytes[coding_at] = Coding::Stored as u8;

        let body_len = bytes.len() - 4;
        let repaired = crc32(&bytes[..body_len]).to_le_bytes();
        bytes[body_len..].copy_from_slice(&repaired);

        let error = SaveContainer::decode(&bytes).expect_err("must be reported");
        assert!(
            error.to_string().contains("checksum"),
            "a flipped coding byte must read as damage, got: {error}"
        );
    }

    /// A coding this build does not know is not a guess to make.
    #[test]
    fn an_unknown_coding_is_refused_rather_than_assumed_to_be_raw() {
        let error = Coding::decode(200, &id("nexora:save/chunks"))
            .expect_err("an unknown coding must be refused");
        assert!(error.to_string().contains("coding"), "got: {error}");
    }

    /// The stored length is the check the checksum cannot make: damage that
    /// happens to still be a valid deflate stream passes the CRC and lands on
    /// the wrong number of bytes.
    #[test]
    fn a_section_that_inflates_to_the_wrong_length_is_refused() {
        let payload = b"twelve bytes";
        let (coding, stored) = pack(&[0u8; 4_096]);
        assert_eq!(coding, Coding::Deflate);
        let error = unpack(
            coding,
            &stored,
            Some(payload.len()),
            &id("nexora:save/chunks"),
        )
        .expect_err("a length mismatch must be reported");
        assert!(error.to_string().contains("size"), "got: {error}");
    }

    /// A scratch directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let base = std::env::temp_dir().join(format!(
                "nexora-test-{label}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).expect("create scratch directory");
            Self(base)
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn containers_round_trip_through_bytes() {
        let original = sample();
        let decoded = SaveContainer::decode(&original.encode()).expect("valid container");
        assert_eq!(decoded, original);
        assert_eq!(decoded.versions(), VersionSet::current());
        assert_eq!(
            decoded
                .section_names()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            [
                "example:save/mod_state",
                "nexora:save/chunks",
                "nexora:save/world_header"
            ]
        );
        assert_eq!(
            decoded.require(&id("nexora:save/world_header")).unwrap(),
            b"header bytes"
        );
    }

    #[test]
    fn an_empty_container_still_round_trips() {
        let container = SaveContainer::new();
        assert!(container.is_empty());
        let decoded = SaveContainer::decode(&container.encode()).unwrap();
        assert_eq!(decoded.len(), 0);
    }

    #[test]
    fn a_missing_section_is_named() {
        let container = sample();
        let err = container
            .require(&id("nexora:save/absent"))
            .expect_err("must not resolve");
        assert!(err.to_string().contains("nexora:save/absent"), "{err}");
        assert_eq!(err.recovery(), Recovery::Quarantine);
    }

    #[test]
    fn every_single_byte_corruption_is_detected() {
        let encoded = sample().encode();
        // Flip one bit in each byte in turn. Nothing may decode successfully:
        // this is the property that makes "detect, then quarantine" possible.
        for index in 0..encoded.len() {
            let mut damaged = encoded.clone();
            damaged[index] ^= 0b0000_0001;
            assert!(
                SaveContainer::decode(&damaged).is_err(),
                "corruption at byte {index} was not detected"
            );
        }
    }

    #[test]
    fn a_truncated_save_is_detected_rather_than_partially_loaded() {
        let encoded = sample().encode();
        for cut in 0..encoded.len() {
            assert!(
                SaveContainer::decode(&encoded[..cut]).is_err(),
                "a save truncated to {cut} bytes decoded anyway"
            );
        }
    }

    #[test]
    fn trailing_garbage_is_refused() {
        let mut encoded = sample().encode();
        encoded.extend_from_slice(b"extra");
        assert!(SaveContainer::decode(&encoded).is_err());
    }

    #[test]
    fn foreign_files_are_refused_by_the_magic_check() {
        assert!(SaveContainer::decode(b"not a nexora save at all").is_err());
        assert!(SaveContainer::decode(&[]).is_err());
    }

    #[test]
    fn write_and_read_round_trip_on_disk() {
        let dir = TempDir::new("roundtrip");
        let path = dir.path("world.nxsv");

        sample().write_atomic(&path).expect("write");
        assert!(path.exists());
        assert!(
            !dir.path("world.nxsv.tmp").exists(),
            "the temporary file must be cleaned up"
        );

        let loaded = SaveContainer::read(&path).expect("read");
        assert_eq!(loaded, sample());
        assert_eq!(loaded.compatibility().unwrap(), Compatibility::ReadCurrent);
    }

    #[test]
    fn writing_creates_missing_directories() {
        let dir = TempDir::new("nested");
        let path = dir.path("saves").join("alpha").join("world.nxsv");
        sample()
            .write_atomic(&path)
            .expect("write into a new directory tree");
        assert!(SaveContainer::read(&path).is_ok());
    }

    #[test]
    fn an_existing_save_survives_being_rewritten() {
        let dir = TempDir::new("rewrite");
        let path = dir.path("world.nxsv");

        let mut first = SaveContainer::new();
        first.put(id("nexora:save/world_header"), b"generation one".to_vec());
        first.write_atomic(&path).unwrap();

        let mut second = SaveContainer::new();
        second.put(id("nexora:save/world_header"), b"generation two".to_vec());
        second.write_atomic(&path).unwrap();

        let loaded = SaveContainer::read(&path).unwrap();
        assert_eq!(
            loaded.require(&id("nexora:save/world_header")).unwrap(),
            b"generation two"
        );
    }

    #[test]
    fn a_corrupt_file_on_disk_reports_quarantine_and_can_be_moved_aside() {
        let dir = TempDir::new("corrupt");
        let path = dir.path("world.nxsv");
        sample().write_atomic(&path).unwrap();

        // Damage the file the way a partial write or bit-rot would.
        let mut bytes = fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        fs::write(&path, &bytes).unwrap();

        let err = SaveContainer::read(&path).expect_err("a damaged save must not load");
        assert_eq!(err.recovery(), Recovery::Quarantine);

        let moved = quarantine(&path).expect("quarantine");
        assert!(moved.exists(), "the damaged save is preserved as evidence");
        assert!(
            !path.exists(),
            "the damaged save no longer occupies the live path"
        );

        // A second incident does not overwrite the first.
        sample().write_atomic(&path).unwrap();
        let second = quarantine(&path).expect("quarantine again");
        assert_ne!(second, moved);
    }

    #[test]
    fn reading_a_missing_file_reports_the_path() {
        let dir = TempDir::new("absent");
        let err = SaveContainer::read(&dir.path("nothing-here.nxsv")).expect_err("must fail");
        assert!(err.to_string().contains("nothing-here.nxsv"), "{err}");
    }

    /// Rewrite a field in an encoded container and repair both checksums, so the
    /// result is a *well-formed* file that merely says something different -
    /// which is what lets a test exercise the version gate rather than the
    /// corruption path.
    fn repair_checksums(encoded: &mut [u8]) {
        let header_end = MAGIC.len() + 4 + 6 + 4 + 4 + 4 + 4;
        let header_crc = crc32(&encoded[..header_end]);
        encoded[header_end..header_end + 4].copy_from_slice(&header_crc.to_le_bytes());

        let body_len = encoded.len() - 4;
        let file_crc = crc32(&encoded[..body_len]);
        encoded[body_len..].copy_from_slice(&file_crc.to_le_bytes());
    }

    #[test]
    fn a_save_from_a_newer_engine_is_refused() {
        let mut encoded = sample().encode();
        let future = SaveFormatVersion(nexora_foundation::version::SAVE_FORMAT_VERSION.0 + 1);
        encoded[4..8].copy_from_slice(&future.0.to_le_bytes());
        repair_checksums(&mut encoded);

        let err = SaveContainer::decode(&encoded).expect_err("a future save must be refused");
        assert_eq!(err.recovery(), Recovery::Reject, "{err}");
        assert!(err.to_string().contains("newer engine"), "{err}");
    }

    #[test]
    fn a_corrupted_section_name_is_detected() {
        // The regression this pins: with the checksum covering only the payload,
        // flipping a bit in a section name produced a different, still-valid
        // identifier and the file decoded as if nothing had happened.
        let encoded = sample().encode();
        let name_offset = encoded
            .windows(b"example:save/mod_state".len())
            .position(|window| window == b"example:save/mod_state")
            .expect("the section name appears in the encoded form");

        let mut damaged = encoded;
        damaged[name_offset] ^= 0b0000_0001;
        assert!(
            SaveContainer::decode(&damaged).is_err(),
            "a renamed section must be detected"
        );
    }
}
