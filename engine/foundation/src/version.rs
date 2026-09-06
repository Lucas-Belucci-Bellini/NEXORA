//! Version identifiers and compatibility policy.
//!
//! Implements `CORE.md` §15 (CORE-12) and the version list required by
//! `NEXORA SAVE FORMAT AND COMPATIBILITY.md`. Every persisted artefact records
//! these so that a world written years ago can state what wrote it.

use core::fmt;

use crate::error::{Domain, Error, Recovery, Result};

/// A three-component engine version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EngineVersion {
    /// Incompatible engine changes.
    pub major: u16,
    /// Backwards-compatible additions.
    pub minor: u16,
    /// Fixes with no contract change.
    pub patch: u16,
}

impl EngineVersion {
    /// Construct an engine version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for EngineVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Declares a newtype over `u32` that names one versioned contract.
macro_rules! contract_version {
    ($(#[$meta:meta])* $name:ident, $label:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($label, "-v{}"), self.0)
            }
        }
    };
}

contract_version!(
    /// Layout version of the save container itself (envelope, framing, checksums).
    SaveFormatVersion,
    "save-format"
);
contract_version!(
    /// Version of the logical world data schema stored inside the container.
    WorldSchemaVersion,
    "world-schema"
);
contract_version!(
    /// Version of the registry ID assignment rules.
    RegistryVersion,
    "registry"
);
contract_version!(
    /// Version of the wire protocol between client and server.
    ProtocolVersion,
    "protocol"
);
contract_version!(
    /// Version of the shipped content set.
    ContentVersion,
    "content"
);
contract_version!(
    /// Version of the world generation algorithm.
    ///
    /// `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md` rule 1: any change
    /// to the generation algorithm must change this number, because the same
    /// seed is only guaranteed to reproduce a world under the same generator.
    GeneratorVersion,
    "generator"
);

/// What the engine is allowed to do with a given persisted version.
///
/// Mirrors the policy list in `NEXORA SAVE FORMAT AND COMPATIBILITY.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compatibility {
    /// Written by this exact version; read directly.
    ReadCurrent,
    /// Older but still readable without transformation.
    ReadLegacy,
    /// Older and readable only after an explicit migration.
    Migrate,
    /// Written by a newer engine; this build must refuse it.
    Breaking,
}

/// The full set of versions that identify how an artefact was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionSet {
    /// Engine build that wrote the artefact.
    pub engine: EngineVersion,
    /// Container layout version.
    pub save_format: SaveFormatVersion,
    /// Logical world schema version.
    pub world_schema: WorldSchemaVersion,
    /// Registry ID rules version.
    pub registry: RegistryVersion,
    /// Content set version.
    pub content: ContentVersion,
}

/// Engine version of this build.
pub const ENGINE_VERSION: EngineVersion = EngineVersion::new(0, 0, 1);

/// Save container layout version this build writes.
pub const SAVE_FORMAT_VERSION: SaveFormatVersion = SaveFormatVersion(1);

/// Oldest save container layout this build can still read.
pub const MIN_SUPPORTED_SAVE_FORMAT: SaveFormatVersion = SaveFormatVersion(1);

/// World schema version this build writes.
pub const WORLD_SCHEMA_VERSION: WorldSchemaVersion = WorldSchemaVersion(1);

/// Registry ID rules version this build implements.
pub const REGISTRY_VERSION: RegistryVersion = RegistryVersion(1);

/// Content set version shipped with this build.
pub const CONTENT_VERSION: ContentVersion = ContentVersion(1);

impl VersionSet {
    /// The version set this build produces.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            engine: ENGINE_VERSION,
            save_format: SAVE_FORMAT_VERSION,
            world_schema: WORLD_SCHEMA_VERSION,
            registry: REGISTRY_VERSION,
            content: CONTENT_VERSION,
        }
    }
}

/// Classify a save-format version found on disk against what this build supports.
///
/// A newer-than-current version is [`Compatibility::Breaking`] rather than a
/// best-effort read: guessing at a future layout is how saves get corrupted.
#[must_use]
pub fn classify_save_format(found: SaveFormatVersion) -> Compatibility {
    if found == SAVE_FORMAT_VERSION {
        Compatibility::ReadCurrent
    } else if found > SAVE_FORMAT_VERSION {
        Compatibility::Breaking
    } else if found >= MIN_SUPPORTED_SAVE_FORMAT {
        Compatibility::ReadLegacy
    } else {
        Compatibility::Migrate
    }
}

/// Reject a save-format version this build must not interpret.
///
/// # Errors
///
/// Returns an error when the version is newer than this build understands.
pub fn require_readable_save_format(found: SaveFormatVersion) -> Result<Compatibility> {
    let compatibility = classify_save_format(found);
    if compatibility == Compatibility::Breaking {
        return Err(Error::new(
            Domain::Save,
            "version-gate",
            "save was written by a newer engine and cannot be read safely",
        )
        .with_recovery(Recovery::Reject)
        .with_context("found", found.to_string())
        .with_context("supported", SAVE_FORMAT_VERSION.to_string()));
    }
    Ok(compatibility)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_reads_as_current() {
        assert_eq!(
            classify_save_format(SAVE_FORMAT_VERSION),
            Compatibility::ReadCurrent
        );
    }

    #[test]
    fn newer_save_is_breaking_and_rejected() {
        let newer = SaveFormatVersion(SAVE_FORMAT_VERSION.0 + 1);
        assert_eq!(classify_save_format(newer), Compatibility::Breaking);

        let err = require_readable_save_format(newer).expect_err("must refuse a future format");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert_eq!(err.domain(), Domain::Save);
    }

    #[test]
    fn older_than_minimum_requires_migration() {
        // Only meaningful once the minimum rises above 1; asserted structurally
        // so the classification stays correct when it does.
        let ancient = SaveFormatVersion(0);
        let expected = if MIN_SUPPORTED_SAVE_FORMAT.0 > 0 {
            Compatibility::Migrate
        } else {
            Compatibility::ReadLegacy
        };
        assert_eq!(classify_save_format(ancient), expected);
    }

    #[test]
    fn versions_render_with_their_contract_name() {
        assert_eq!(SaveFormatVersion(3).to_string(), "save-format-v3");
        assert_eq!(GeneratorVersion(7).to_string(), "generator-v7");
        assert_eq!(EngineVersion::new(1, 2, 3).to_string(), "1.2.3");
    }
}
