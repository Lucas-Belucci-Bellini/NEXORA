//! The operations the command line performs.
//!
//! The library half of the tool: generate a material and write it, read one
//! back, validate what is on disk, list what is there. `main.rs` parses
//! arguments and prints; everything it does is here, so it can be tested
//! without a process.
//!
//! # Nothing is overwritten in silence
//!
//! The brief §15 is explicit. So [`Forge::generate`] compares what it is about
//! to write with what is already there:
//!
//! ```text
//! nothing on disk            -> written
//! same appearance            -> unchanged, and nothing is rewritten
//! different, without --force -> refused, and it says which field moved
//! different, with --force    -> written at the next revision
//! ```
//!
//! The middle case matters more than it looks. Generation is deterministic, so
//! a definition that has not changed produces the same bytes; noticing that and
//! doing nothing is what makes regenerating ten thousand materials cheap.

use std::path::{Path, PathBuf};

use nexora_asset::document;
use nexora_asset::generator::{
    GeneratedMaterial, GenerationRequest, TextureGenerator, TexturePipeline,
};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::{MapRole, MapSet, TextureFormat, TextureMap};
use nexora_asset::validation::TextureValidationResult;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

use crate::layout;
use crate::pbr::PbrPipeline;
use crate::png;
use crate::procedural::ProceduralGenerator;
use crate::validator::Validator;

/// How deep [`Forge::list`] will walk looking for materials.
///
/// An identifier's path may nest, but not without limit, and a walk with no
/// bound is a walk a symlink can send anywhere.
pub const MAX_LIST_DEPTH: usize = 8;

/// What happened to a material that was asked to be generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteStatus {
    /// It was not there, and now it is.
    Written,
    /// It was there, identical, and nothing was rewritten.
    Unchanged,
    /// It was there and different, and `--force` was not given.
    Refused,
    /// It was there and different, and it was replaced at a new revision.
    Replaced,
}

impl WriteStatus {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Written => "written",
            Self::Unchanged => "unchanged",
            Self::Refused => "refused",
            Self::Replaced => "replaced",
        }
    }

    /// Whether anything reached the filesystem.
    #[must_use]
    pub const fn wrote_files(self) -> bool {
        matches!(self, Self::Written | Self::Replaced)
    }
}

/// What one generation run produced.
#[derive(Debug)]
pub struct Outcome {
    /// What happened.
    pub status: WriteStatus,
    /// The material as it now stands, whether written or already there.
    pub material: SurfaceMaterial,
    /// What the validator found.
    pub validation: TextureValidationResult,
    /// The files written, and how large each is.
    pub files: Vec<(PathBuf, usize)>,
    /// Why it was refused, when it was.
    pub refusal: Option<String>,
}

impl Outcome {
    /// Total bytes written.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.files.iter().map(|(_, size)| *size).sum()
    }
}

/// What is written under a root.
#[derive(Debug, Default)]
pub struct Listing {
    /// Every material whose definition read back, in identifier order.
    pub materials: Vec<SurfaceMaterial>,
    /// Every definition that did not, and why.
    ///
    /// Reported rather than fatal: one unreadable file should not hide the
    /// hundred beside it, and a listing that stops at the first problem is a
    /// listing nobody can use to find the problem.
    pub unreadable: Vec<(PathBuf, Error)>,
}

impl Listing {
    /// How many materials were found.
    #[must_use]
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Whether nothing at all was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty() && self.unreadable.is_empty()
    }
}

/// Generates, writes, reads and checks materials under an output root.
#[derive(Debug)]
pub struct Forge {
    root: PathBuf,
    generator: ProceduralGenerator,
    pipeline: PbrPipeline,
    validator: Validator,
}

impl Forge {
    /// A forge writing under a root directory.
    ///
    /// # Errors
    ///
    /// Returns an error when the generator or pipeline cannot be constructed,
    /// which would be a build-time mistake rather than a runtime condition.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self {
            root: root.into(),
            generator: ProceduralGenerator::new()?,
            pipeline: PbrPipeline::new()?,
            validator: Validator::new(),
        })
    }

    /// The output root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Realise a definition and write it, unless it is already there.
    ///
    /// # Errors
    ///
    /// Returns an error when generation or the pipeline fails, when validation
    /// fails, or when the filesystem refuses. A material that fails validation
    /// is **not written**: the brief §7 forbids accepting an invalid texture,
    /// and writing one and reporting it afterwards is accepting it.
    pub fn generate(
        &self,
        definition: &SurfaceMaterial,
        seed: u64,
        force: bool,
    ) -> Result<Outcome> {
        layout::check_naming(definition)?;

        let existing = self.read_definition(definition.id()).ok();
        if let Some(existing) = &existing {
            if existing.appearance_hash() == definition.appearance_hash()
                && self.all_maps_present(existing)
            {
                return Ok(Outcome {
                    status: WriteStatus::Unchanged,
                    material: existing.clone(),
                    validation: TextureValidationResult::new(),
                    files: Vec::new(),
                    refusal: None,
                });
            }
            if !force {
                return Ok(Outcome {
                    status: WriteStatus::Refused,
                    refusal: Some(format!(
                        "{} is already written at revision {} and describes a different surface; \
                         pass --force to replace it",
                        existing.id(),
                        existing.revision()
                    )),
                    material: existing.clone(),
                    validation: TextureValidationResult::new(),
                    files: Vec::new(),
                });
            }
        }

        // A replacement is numbered above what it replaced, so which came
        // first stays on the record. Generation and the pipeline each add one
        // more, so the result is strictly newer than the file it overwrites.
        let incoming = existing.as_ref().map_or_else(
            || definition.clone(),
            |existing| definition.at_revision(existing.revision()),
        );
        let produced = self
            .generator
            .generate(&GenerationRequest::generate(incoming, seed))?;
        let produced = self.pipeline.apply(produced)?;

        let validation = self.validator.material(&produced);
        validation.ok(&definition.id().to_string())?;

        let files = self.write(&produced)?;
        Ok(Outcome {
            status: if existing.is_some() {
                WriteStatus::Replaced
            } else {
                WriteStatus::Written
            },
            material: produced.definition().clone(),
            validation,
            files,
            refusal: None,
        })
    }

    /// Read a material's definition from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the file is absent or does not parse.
    pub fn read_definition(&self, id: &Identifier) -> Result<SurfaceMaterial> {
        let path = layout::definition_file(&self.root, id);
        let text = std::fs::read_to_string(&path).map_err(|cause| {
            missing("the material's definition could not be read")
                .with_context("path", path.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        document::from_text(&text)
    }

    /// Read the maps a material has on disk.
    ///
    /// Absent files are absent, not an error: which maps a material should
    /// have is the definition's business, and reporting the gap is the
    /// validator's.
    ///
    /// # Errors
    ///
    /// Returns an error when a file exists but does not decode into a map of
    /// the size and layout the definition declares.
    pub fn read_maps(&self, definition: &SurfaceMaterial) -> Result<MapSet> {
        let mut maps = MapSet::new();
        for role in MapRole::ALL {
            let path = layout::map_file(&self.root, definition.id(), role);
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let decoded = png::decode(&bytes)?;
            maps.insert(TextureMap::new(
                role,
                TextureFormat::eight_bit(decoded.layout),
                decoded.resolution,
                decoded.pixels,
            )?)?;
        }
        Ok(maps)
    }

    /// Check what is on disk for a material.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition cannot be read or a map file does
    /// not decode. A decoded map that is *wrong* is a finding, not an error —
    /// the point of the call is to collect those.
    pub fn validate(&self, id: &Identifier) -> Result<(SurfaceMaterial, TextureValidationResult)> {
        let definition = self.read_definition(id)?;
        let maps = self.read_maps(&definition)?;
        let mut result = self.validator.maps(&definition, &maps);

        for role in MapRole::ALL {
            let wanted = role.is_required() || definition.wanted_maps().contains(&role);
            if wanted && !maps.contains(role) {
                let path = layout::map_file(&self.root, id, role);
                result.push(
                    nexora_asset::validation::Finding::failure(
                        nexora_asset::validation::Check::FileExists,
                        format!(
                            "the material wants this map and {} is not there",
                            path.display()
                        ),
                    )
                    .about(role),
                );
            }
        }
        Ok((definition, result))
    }

    /// Every material written under the root, in identifier order.
    ///
    /// # Errors
    ///
    /// Returns an error when the root cannot be read. A directory holding a
    /// `material.json` that does not parse is reported and skipped rather than
    /// stopping the walk, because one bad file should not hide the rest.
    pub fn list(&self) -> Result<Listing> {
        let mut listing = Listing::default();
        if self.root.is_dir() {
            self.walk(&self.root, 0, &mut listing);
        }
        listing.materials.sort_by(|a, b| a.id().cmp(b.id()));
        Ok(listing)
    }

    fn walk(&self, directory: &Path, depth: usize, listing: &mut Listing) {
        if depth > MAX_LIST_DEPTH {
            return;
        }
        let candidate = directory.join(layout::DEFINITION_FILE);
        if candidate.is_file() {
            match std::fs::read_to_string(&candidate)
                .map_err(|cause| {
                    missing("a definition could not be read")
                        .with_context("cause", cause.to_string())
                })
                .and_then(|text| document::from_text(&text))
            {
                Ok(material) => listing.materials.push(material),
                Err(error) => listing.unreadable.push((candidate, error)),
            }
        }
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // Symlinks are not followed: a walk that follows them is a walk
            // that can leave the output root and never come back.
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                self.walk(&path, depth + 1, listing);
            }
        }
    }

    fn all_maps_present(&self, definition: &SurfaceMaterial) -> bool {
        MapRole::ALL
            .into_iter()
            .filter(|role| role.is_required() || definition.wanted_maps().contains(role))
            .all(|role| layout::map_file(&self.root, definition.id(), role).is_file())
    }

    fn write(&self, produced: &GeneratedMaterial) -> Result<Vec<(PathBuf, usize)>> {
        let definition = produced.definition();
        let directory = layout::material_dir(&self.root, definition.id());
        std::fs::create_dir_all(&directory).map_err(|cause| {
            unwritable("the material's directory could not be created")
                .with_context("path", directory.display().to_string())
                .with_context("cause", cause.to_string())
        })?;

        let mut files = Vec::new();
        for map in produced.maps().iter() {
            let path = layout::map_file(&self.root, definition.id(), map.role());
            let bytes = png::encode(map)?;
            write_file(&path, &bytes)?;
            files.push((path, bytes.len()));
        }
        if let Some(preview) = produced.preview() {
            let path = layout::preview_file(&self.root, definition.id());
            let bytes = png::encode(preview)?;
            write_file(&path, &bytes)?;
            files.push((path, bytes.len()));
        }

        // The definition goes last. A directory holding a definition and no
        // textures reads as a finished material; the reverse reads as a
        // half-written one, which is the truth if this fails partway.
        let path = layout::definition_file(&self.root, definition.id());
        let text = document::to_text(definition);
        write_file(&path, text.as_bytes())?;
        files.push((path, text.len()));
        Ok(files)
    }
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes).map_err(|cause| {
        unwritable("a file could not be written")
            .with_context("path", path.display().to_string())
            .with_context("cause", cause.to_string())
    })
}

fn missing(message: &'static str) -> Error {
    Error::new(Domain::Content, "forge", message).with_recovery(Recovery::Quarantine)
}

fn unwritable(message: &'static str) -> Error {
    Error::new(Domain::Content, "forge", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::{MaterialCategory, PbrParameters, Revision};
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::Resolution;
    use nexora_asset::validation::{Check, Severity, Verdict};

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    /// A directory nothing else in the suite writes to.
    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nexora-forge-{}-{name}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn definition(path: &str, roughness: f64) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id(path),
            MaterialCategory::Wood,
            Resolution::square(32).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .pbr(PbrParameters {
            roughness,
            ..PbrParameters::DEFAULT
        })
        .wants(MapRole::Height)
        .wants(MapRole::Normal)
        .build()
        .expect("valid definition")
    }

    #[test]
    fn generating_writes_every_map_and_the_definition() {
        let root = scratch("write");
        let forge = Forge::new(&root).unwrap();
        let outcome = forge
            .generate(&definition("nexora:material/oak", 0.8), 7, false)
            .expect("generation succeeds");

        assert_eq!(outcome.status, WriteStatus::Written);
        assert!(outcome.status.wrote_files());
        assert!(outcome.refusal.is_none());
        assert_eq!(outcome.validation.verdict(), Verdict::Pass);

        // Albedo, height, normal and the definition.
        assert_eq!(outcome.files.len(), 4, "{:?}", outcome.files);
        for (path, size) in &outcome.files {
            assert!(
                path.is_file(),
                "{} was reported but is not there",
                path.display()
            );
            assert_eq!(
                std::fs::metadata(path).unwrap().len() as usize,
                *size,
                "{} is not the size it was reported as",
                path.display()
            );
        }
        assert!(outcome.byte_len() > 0);

        // The definition is written last, so a directory that has one is a
        // directory whose textures are already there.
        assert!(outcome
            .files
            .last()
            .unwrap()
            .0
            .ends_with(layout::DEFINITION_FILE));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn regenerating_an_unchanged_definition_writes_nothing() {
        let root = scratch("unchanged");
        let forge = Forge::new(&root).unwrap();
        let definition = definition("nexora:material/oak", 0.8);
        forge.generate(&definition, 7, false).unwrap();

        let albedo = layout::map_file(&root, definition.id(), MapRole::Albedo);
        let before = std::fs::read(&albedo).unwrap();

        let again = forge.generate(&definition, 7, false).expect("second pass");
        assert_eq!(again.status, WriteStatus::Unchanged);
        assert!(!again.status.wrote_files());
        assert!(again.files.is_empty());
        assert_eq!(std::fs::read(&albedo).unwrap(), before);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_changed_definition_is_refused_until_it_is_forced() {
        let root = scratch("refuse");
        let forge = Forge::new(&root).unwrap();
        let first = forge
            .generate(&definition("nexora:material/oak", 0.8), 7, false)
            .unwrap();
        let albedo = layout::map_file(&root, &id("nexora:material/oak"), MapRole::Albedo);
        let before = std::fs::read(&albedo).unwrap();

        let refused = forge
            .generate(&definition("nexora:material/oak", 0.2), 7, false)
            .expect("a refusal is an outcome, not an error");
        assert_eq!(refused.status, WriteStatus::Refused);
        assert!(refused.refusal.as_ref().unwrap().contains("--force"));
        assert_eq!(
            std::fs::read(&albedo).unwrap(),
            before,
            "a refusal must leave the files alone"
        );

        let forced = forge
            .generate(&definition("nexora:material/oak", 0.2), 7, true)
            .expect("forced");
        assert_eq!(forced.status, WriteStatus::Replaced);
        assert_ne!(std::fs::read(&albedo).unwrap(), before);
        // Strictly newer, so which came first stays on the record.
        assert!(
            forced.material.revision() > first.material.revision(),
            "{} is not above {}",
            forced.material.revision(),
            first.material.revision()
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn validating_reads_the_files_rather_than_the_memory_they_came_from() {
        let root = scratch("validate");
        let forge = Forge::new(&root).unwrap();
        let definition = definition("nexora:material/oak", 0.8);
        forge.generate(&definition, 7, false).unwrap();

        let (read_back, result) = forge.validate(definition.id()).expect("validates");
        assert_eq!(result.verdict(), Verdict::Pass, "{result}");
        assert_eq!(read_back.id(), definition.id());

        // Damage a file on disk. Nothing in memory changed.
        let albedo = layout::map_file(&root, definition.id(), MapRole::Albedo);
        let mut bytes = std::fs::read(&albedo).unwrap();
        let middle = bytes.len() / 2;
        bytes[middle] ^= 0xFF;
        std::fs::write(&albedo, &bytes).unwrap();

        let err = forge
            .validate(definition.id())
            .expect_err("a file that does not decode is an error, not a finding");
        assert!(
            err.to_string().contains("checksum") || err.to_string().contains("decode"),
            "{err}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_map_the_material_wants_and_does_not_have_is_reported_by_path() {
        let root = scratch("missing");
        let forge = Forge::new(&root).unwrap();
        let definition = definition("nexora:material/oak", 0.8);
        forge.generate(&definition, 7, false).unwrap();

        std::fs::remove_file(layout::map_file(&root, definition.id(), MapRole::Normal)).unwrap();

        let (_, result) = forge.validate(definition.id()).expect("still readable");
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::FileExists),
            "{result}"
        );
        assert!(result.to_string().contains("normal.png"), "{result}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn listing_finds_every_material_and_survives_one_that_is_broken() {
        let root = scratch("list");
        let forge = Forge::new(&root).unwrap();
        for path in [
            "nexora:material/oak",
            "nexora:material/pine",
            "example:material/teak",
        ] {
            forge.generate(&definition(path, 0.8), 7, false).unwrap();
        }

        // A directory that looks like a material and holds nonsense.
        let broken_dir = root.join("nexora").join("broken");
        std::fs::create_dir_all(&broken_dir).unwrap();
        std::fs::write(broken_dir.join(layout::DEFINITION_FILE), "{ not json").unwrap();

        let listing = forge.list().expect("the walk completes");
        let names: Vec<String> = listing
            .materials
            .iter()
            .map(|m| m.id().to_string())
            .collect();
        assert_eq!(
            names,
            [
                "example:material/teak",
                "nexora:material/oak",
                "nexora:material/pine"
            ],
            "sorted by identifier, and one bad file does not hide the rest"
        );
        assert_eq!(listing.len(), 3);
        assert_eq!(listing.unreadable.len(), 1);
        assert!(listing.unreadable[0].0.ends_with(layout::DEFINITION_FILE));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn listing_an_empty_root_is_not_an_error() {
        let forge = Forge::new(scratch("empty")).unwrap();
        let listing = forge.list().expect("an absent root is simply empty");
        assert!(listing.is_empty());
        assert_eq!(listing.len(), 0);
    }

    #[test]
    fn an_identifier_that_would_escape_the_root_never_reaches_the_filesystem() {
        let root = scratch("escape");
        let forge = Forge::new(&root).unwrap();
        let escaping = definition("nexora:material/../../etc", 0.8);

        let err = forge
            .generate(&escaping, 7, false)
            .expect_err("`..` must never become a directory");
        assert!(err.to_string().contains(".."), "{err}");
        assert!(!root.exists(), "nothing may be created for a refused name");
    }

    #[test]
    fn the_revision_written_is_the_one_generation_produced() {
        let root = scratch("revision");
        let forge = Forge::new(&root).unwrap();
        let authored = definition("nexora:material/oak", 0.8);
        assert_eq!(authored.revision(), Revision::FIRST);

        let outcome = forge.generate(&authored, 7, false).unwrap();
        // Authored at one, generated at two, transformed at three: each step
        // is a revision, and the file records where it ended up.
        assert_eq!(outcome.material.revision(), Revision(3));
        assert_eq!(
            forge.read_definition(authored.id()).unwrap().revision(),
            Revision(3)
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
