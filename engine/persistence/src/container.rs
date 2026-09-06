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
            out.string(&rendered);
            // The checksum covers the name as well as the payload. Covering only
            // the payload leaves the name unprotected, and a single flipped bit
            // in a section name silently renames the section instead of being
            // reported as damage.
            out.u32(section_crc(&rendered, data));
            out.bytes(data);
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
            let data = reader.bytes()?;
            if section_crc(rendered, data) != declared_crc {
                return Err(corrupt("save section failed its checksum")
                    .with_context("section", name.to_string()));
            }
            if sections.insert(name.clone(), data.to_vec()).is_some() {
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
fn section_crc(name: &str, data: &[u8]) -> u32 {
    let mut combined = Vec::with_capacity(name.len() + data.len());
    combined.extend_from_slice(name.as_bytes());
    combined.extend_from_slice(data);
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
