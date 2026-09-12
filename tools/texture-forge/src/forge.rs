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
use crate::manifest::Manifest;
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
    /// Maps that were lost or corrupt were rebuilt; nothing else changed.
    ///
    /// Distinct from `Replaced` on purpose. A replacement is a different
    /// surface at a new revision; a restoration is the same surface, the same
    /// revision, and the same pixels that were there before — the only thing
    /// that changed is that the files exist again.
    Restored,
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
            Self::Restored => "restored",
        }
    }

    /// Whether anything reached the filesystem.
    #[must_use]
    pub const fn wrote_files(self) -> bool {
        matches!(self, Self::Written | Self::Replaced | Self::Restored)
    }
}

/// What is on disk for one of a material's maps.
///
/// Coarser than [`nexora_asset::validation`] on purpose: this answers "can the
/// file be read back at all", which is the question a repair asks. Whether a
/// readable map is a *good* map is the validator's question, and its answer
/// does not make the file repairable — see [`Forge::repair`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapHealth {
    /// The file is there and decodes at the declared resolution.
    Present,
    /// No file.
    Absent,
    /// A file that is not a PNG this build can read.
    Unreadable,
    /// A PNG, but not the size the definition declares.
    WrongSize,
}

impl MapHealth {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Absent => "absent",
            Self::Unreadable => "unreadable",
            Self::WrongSize => "wrong_size",
        }
    }

    /// Whether the file can be used as it stands.
    #[must_use]
    pub const fn is_intact(self) -> bool {
        matches!(self, Self::Present)
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

/// What a whole manifest produced.
///
/// # One bad entry does not stop five hundred good ones
///
/// [`Forge::batch`] carries on past a failure and collects it. A run that
/// stopped at the first problem would make the operator fix one material,
/// re-run, wait, and find the next — which is the slow loop the batch exists
/// to replace. Every failure is reported, and [`Self::is_clean`] is what the
/// caller checks before claiming the set was built.
#[derive(Debug, Default)]
pub struct BatchReport {
    /// What happened to each material that was attempted, in manifest order.
    pub outcomes: Vec<Outcome>,
    /// Every entry that could not be produced, and why.
    pub failures: Vec<(Identifier, Error)>,
}

impl BatchReport {
    /// How many entries reached the given status.
    #[must_use]
    pub fn counted(&self, status: WriteStatus) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.status == status)
            .count()
    }

    /// Total bytes written across the run.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.outcomes.iter().map(Outcome::byte_len).sum()
    }

    /// Whether every entry was produced and none was refused.
    ///
    /// A refusal is not clean: the manifest asked for a material that is
    /// already there and different, and pretending otherwise would let a batch
    /// report success over a set it did not write.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.failures.is_empty() && self.counted(WriteStatus::Refused) == 0
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
        self.produce(
            definition,
            GenerationRequest::generate(definition.clone(), seed),
            force,
        )
    }

    /// Everything `generate` and `variant` share: the write policy.
    ///
    /// They differ only in the request they build, so the rules about what is
    /// already on disk live here once. Two copies of "nothing is overwritten
    /// in silence" would be one copy too many.
    fn produce(
        &self,
        definition: &SurfaceMaterial,
        request: GenerationRequest,
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
        let mut request = request;
        request.definition = incoming;
        let produced = self.generator.generate(&request)?;
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

    /// Realise every material a manifest declares.
    ///
    /// Does not return `Result`: nothing about the run as a whole can fail,
    /// only individual entries, and each of those lands in
    /// [`BatchReport::failures`] beside the id that could not be produced.
    /// The caller decides what to do about them — [`BatchReport::is_clean`]
    /// says whether there is anything to decide.
    pub fn batch(&self, manifest: &Manifest, force: bool) -> BatchReport {
        let mut report = BatchReport::default();
        for definition in &manifest.materials {
            match self.generate(definition, manifest.seed, force) {
                Ok(outcome) => report.outcomes.push(outcome),
                Err(cause) => report.failures.push((definition.id().clone(), cause)),
            }
        }
        report
    }

    /// Produce another material like one already written.
    ///
    /// The source is read from disk only to establish that it is there — the
    /// variant renders from `definition`, and the source's *identifier* is what
    /// seeds it. Writing, refusal and revision behave exactly as
    /// [`Self::generate`], because a variant is a material like any other once
    /// it exists.
    ///
    /// # Errors
    ///
    /// Returns an error when the source is not written, or for any reason
    /// [`Self::generate`] would.
    pub fn variant(
        &self,
        definition: &SurfaceMaterial,
        of: &Identifier,
        index: u32,
        seed: u64,
        force: bool,
    ) -> Result<Outcome> {
        if index == 0 {
            return Err(
                missing("variants are counted from one").with_context("index", index.to_string())
            );
        }
        // Refused rather than allowed: "variant of X" is a claim about X, and
        // a claim about a material nobody has written is not checkable later.
        self.read_definition(of).map_err(|cause| {
            missing("the material being varied is not written")
                .with_context("source", of.to_string())
                .with_context("cause", cause.to_string())
        })?;
        if definition.id() == of {
            return Err(missing("a material cannot be a variant of itself")
                .with_context("material", of.to_string()));
        }
        self.produce(
            definition,
            GenerationRequest::variant(definition.clone(), of.clone(), index, seed),
            force,
        )
    }

    /// Which of a material's maps are on disk and readable.
    ///
    /// Unlike [`Self::read_maps`], a file that does not decode is reported
    /// rather than fatal — finding out *which* file is broken is the entire
    /// point of the call, and a survey that stops at the first one cannot say.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition itself cannot be read.
    pub fn survey(&self, definition: &SurfaceMaterial) -> Vec<(MapRole, MapHealth)> {
        let mut health = Vec::new();
        for role in MapRole::ALL {
            if !(role.is_required() || definition.wanted_maps().contains(&role)) {
                continue;
            }
            let path = layout::map_file(&self.root, definition.id(), role);
            let state = match std::fs::read(&path) {
                Err(_) => MapHealth::Absent,
                Ok(bytes) => match png::decode(&bytes) {
                    Err(_) => MapHealth::Unreadable,
                    Ok(decoded) if decoded.resolution != definition.resolution() => {
                        MapHealth::WrongSize
                    }
                    Ok(_) => MapHealth::Present,
                },
            };
            health.push((role, state));
        }
        health
    }

    /// Restore a material's maps that are lost or corrupt.
    ///
    /// # What a repair is not
    ///
    /// It is not a regeneration. It renders with the seed the material's own
    /// record names, so a restored map is byte-identical to the one that was
    /// lost, and the maps that were fine are left alone.
    ///
    /// It is also not a fix for a map that is *present, readable and failing a
    /// check*. Regenerating that map produces the same failing map, because
    /// the recipe is what is wrong. Those are reported by
    /// [`Self::validate`] and left where they are.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition cannot be read, when the material
    /// carries no generation record to reproduce from, or when generation or
    /// validation fails.
    pub fn repair(&self, id: &Identifier) -> Result<Outcome> {
        let definition = self.read_definition(id)?;
        let lost: Vec<MapRole> = self
            .survey(&definition)
            .into_iter()
            .filter(|(_, health)| !health.is_intact())
            .map(|(role, _)| role)
            .collect();

        if lost.is_empty() {
            return Ok(Outcome {
                status: WriteStatus::Unchanged,
                material: definition,
                validation: TextureValidationResult::new(),
                files: Vec::new(),
                refusal: None,
            });
        }

        // The whole set is rendered, and only the lost files are written:
        // the normal map is derived from height, so a run restricted to the
        // normal map would have nothing to derive it from.
        let produced = self
            .generator
            .generate(&GenerationRequest::repair(definition.clone()))?;
        let produced = self.pipeline.apply(produced)?;

        let validation = self.validator.material(&produced);
        validation.ok(&id.to_string())?;

        // Only the lost ones. The pipeline produced every map the definition
        // wants, and rewriting the intact ones would be a regeneration wearing
        // a repair's name — and would touch files a person may have hand-edited.
        let files = self.write_maps(&produced, &lost)?;
        Ok(Outcome {
            status: WriteStatus::Restored,
            material: definition,
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

    /// Write only the named maps, leaving the definition and the rest alone.
    ///
    /// What a repair needs: the definition on disk is still correct — the same
    /// material, the same revision, the same record — and rewriting it would
    /// claim a change that did not happen.
    fn write_maps(
        &self,
        produced: &GeneratedMaterial,
        roles: &[MapRole],
    ) -> Result<Vec<(PathBuf, usize)>> {
        let id = produced.definition().id();
        let directory = layout::material_dir(&self.root, id);
        std::fs::create_dir_all(&directory).map_err(|cause| {
            unwritable("the material's directory could not be created")
                .with_context("path", directory.display().to_string())
                .with_context("cause", cause.to_string())
        })?;

        let mut files = Vec::new();
        for role in roles {
            let Some(map) = produced.maps().get(*role) else {
                return Err(
                    unwritable("the pipeline did not produce a map that was lost")
                        .with_context("material", id.to_string())
                        .with_context("map", role.as_str()),
                );
            };
            let path = layout::map_file(&self.root, id, *role);
            let bytes = png::encode(map)?;
            write_file(&path, &bytes)?;
            files.push((path, bytes.len()));
        }
        Ok(files)
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

    // ---- batch -------------------------------------------------------

    fn manifest(text: &str) -> Manifest {
        crate::manifest::from_text(text).expect("the test manifest must read")
    }

    const SET: &str = r#"{
        "schema": 1,
        "name": "temperate forest",
        "prefix": "temperate_forest",
        "seed": "0x5eed",
        "defaults": { "resolution": { "width": 16, "height": 16 },
                      "maps": ["normal", "height"] },
        "materials": [ { "name": "oak_bark", "category": "wood" },
                       { "name": "forest_soil", "category": "soil" },
                       { "name": "granite", "category": "stone" } ]
    }"#;

    #[test]
    fn a_batch_writes_every_material_the_manifest_declares() {
        let root = scratch("batch-write");
        let forge = Forge::new(&root).unwrap();
        let report = forge.batch(&manifest(SET), false);

        assert!(report.is_clean(), "{:?}", report.failures);
        assert_eq!(report.counted(WriteStatus::Written), 3);
        assert!(report.byte_len() > 0);

        // And they are where the grouping says they are.
        for short in ["oak_bark", "forest_soil", "granite"] {
            // `material_dir` strips the `material/` prefix, so the grouping
            // segment is what shows up on disk.
            let directory = root.join("nexora/temperate_forest").join(short);
            assert!(
                directory.join(layout::DEFINITION_FILE).is_file(),
                "{} has no definition",
                directory.display()
            );
        }
        assert_eq!(forge.list().unwrap().len(), 3);
    }

    #[test]
    fn re_running_a_batch_rewrites_nothing() {
        // The property that makes regenerating ten thousand materials cheap,
        // measured across a whole set rather than one material at a time.
        let root = scratch("batch-idempotent");
        let forge = Forge::new(&root).unwrap();
        assert!(forge.batch(&manifest(SET), false).is_clean());

        let again = forge.batch(&manifest(SET), false);
        assert!(again.is_clean());
        assert_eq!(again.counted(WriteStatus::Unchanged), 3);
        assert_eq!(again.byte_len(), 0, "nothing should have been rewritten");
    }

    #[test]
    fn one_failing_entry_does_not_stop_the_others() {
        // The whole point of the policy. `..` is a legal identifier segment
        // and an illegal path segment, so this entry fails at the naming
        // guard — after the first material and before the last.
        let root = scratch("batch-partial");
        let forge = Forge::new(&root).unwrap();
        let report = forge.batch(
            &manifest(
                r#"{
                    "schema": 1, "name": "mixed",
                    "defaults": { "resolution": { "width": 16, "height": 16 } },
                    "materials": [ { "name": "good_one", "category": "stone" },
                                   { "name": "../escape", "category": "stone" },
                                   { "name": "good_two", "category": "stone" } ]
                }"#,
            ),
            false,
        );

        assert!(!report.is_clean(), "a failure must not read as success");
        assert_eq!(report.failures.len(), 1);
        assert_eq!(
            report.failures[0].0.to_string(),
            "nexora:material/../escape"
        );
        assert_eq!(
            report.counted(WriteStatus::Written),
            2,
            "the entries either side of the bad one must still be written"
        );
    }

    #[test]
    fn a_batch_over_something_already_written_and_different_is_refused_not_clean() {
        let root = scratch("batch-refuse");
        let forge = Forge::new(&root).unwrap();
        assert!(forge.batch(&manifest(SET), false).is_clean());

        // Same names, one different surface.
        let changed = manifest(&SET.replace(
            r#"{ "name": "granite", "category": "stone" }"#,
            r#"{ "name": "granite", "category": "metal" }"#,
        ));
        let report = forge.batch(&changed, false);
        assert!(!report.is_clean(), "a refusal is not a clean run");
        assert_eq!(report.counted(WriteStatus::Refused), 1);
        assert_eq!(report.counted(WriteStatus::Unchanged), 2);
        assert!(report.failures.is_empty(), "a refusal is not an error");

        // And with --force it goes through, at a newer revision.
        let forced = forge.batch(&changed, true);
        assert!(forced.is_clean());
        assert_eq!(forced.counted(WriteStatus::Replaced), 1);
        let granite = forge
            .read_definition(&id("nexora:material/temperate_forest/granite"))
            .unwrap();
        assert!(granite.revision() > Revision(1), "{}", granite.revision());
        assert_eq!(granite.category(), MaterialCategory::Metal);
    }

    #[test]
    fn a_manifest_declaring_nothing_writes_nothing_and_is_clean() {
        let root = scratch("batch-empty");
        let forge = Forge::new(&root).unwrap();
        let report = forge.batch(
            &manifest(r#"{ "schema": 1, "name": "none", "materials": [] }"#),
            false,
        );
        assert!(report.is_clean());
        assert!(report.outcomes.is_empty());
        assert_eq!(report.byte_len(), 0);
    }

    #[test]
    fn every_material_in_a_set_gets_its_own_noise() {
        // One manifest carries one seed, so the generator must be mixing the
        // identifier in — otherwise a set is one texture repeated.
        let root = scratch("batch-distinct");
        let forge = Forge::new(&root).unwrap();
        let report = forge.batch(&manifest(SET), false);
        assert!(report.is_clean());

        let albedos: Vec<Vec<u8>> = report
            .outcomes
            .iter()
            .map(|outcome| {
                let path = layout::map_file(&root, outcome.material.id(), MapRole::Albedo);
                std::fs::read(path).expect("every material has an albedo")
            })
            .collect();
        assert_eq!(albedos.len(), 3);
        assert_ne!(albedos[0], albedos[1]);
        assert_ne!(albedos[1], albedos[2]);
        assert_ne!(albedos[0], albedos[2]);
    }

    // ---- variant and repair ------------------------------------------

    #[test]
    fn a_variant_is_a_material_like_any_other_once_it_exists() {
        let root = scratch("variant");
        let forge = Forge::new(&root).unwrap();
        let source = definition("nexora:material/oak", 0.8);
        forge.generate(&source, 3, false).unwrap();

        let derived = definition("nexora:material/oak_weathered", 0.8);
        let outcome = forge
            .variant(&derived, source.id(), 1, 0, false)
            .expect("the variant is produced");

        assert_eq!(outcome.status, WriteStatus::Written);
        assert_eq!(outcome.validation.verdict(), Verdict::Pass);
        assert_eq!(forge.list().unwrap().len(), 2);

        // Same recipe, different pixels.
        let a = std::fs::read(layout::map_file(&root, source.id(), MapRole::Albedo)).unwrap();
        let b = std::fs::read(layout::map_file(&root, derived.id(), MapRole::Albedo)).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn a_variant_of_a_material_that_is_not_written_is_refused() {
        // "variant of X" is a claim about X. A claim about nothing cannot be
        // checked later, so it is refused now.
        let root = scratch("variant-orphan");
        let forge = Forge::new(&root).unwrap();
        let error = forge
            .variant(
                &definition("nexora:material/oak_2", 0.8),
                &id("nexora:material/oak"),
                1,
                0,
                false,
            )
            .expect_err("there is no source");
        assert!(error.to_string().contains("not written"), "{error}");
    }

    #[test]
    fn a_variant_of_itself_and_a_variant_numbered_zero_are_refused() {
        let root = scratch("variant-degenerate");
        let forge = Forge::new(&root).unwrap();
        let source = definition("nexora:material/oak", 0.8);
        forge.generate(&source, 3, false).unwrap();

        let itself = forge
            .variant(&source, source.id(), 1, 0, false)
            .expect_err("a material cannot be its own variant");
        assert!(itself.to_string().contains("variant of itself"), "{itself}");

        let zero = forge
            .variant(
                &definition("nexora:material/oak_2", 0.8),
                source.id(),
                0,
                0,
                false,
            )
            .expect_err("variants count from one");
        assert!(zero.to_string().contains("counted from one"), "{zero}");
    }

    #[test]
    fn a_survey_reports_every_map_rather_than_stopping_at_the_first_bad_one() {
        let root = scratch("survey");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 5, false).unwrap();

        std::fs::remove_file(layout::map_file(&root, material.id(), MapRole::Height)).unwrap();
        std::fs::write(
            layout::map_file(&root, material.id(), MapRole::Normal),
            b"this is not a png",
        )
        .unwrap();

        let survey = forge.survey(&material);
        let state = |role: MapRole| {
            survey
                .iter()
                .find(|(each, _)| *each == role)
                .map(|(_, health)| *health)
        };
        assert_eq!(state(MapRole::Albedo), Some(MapHealth::Present));
        assert_eq!(state(MapRole::Height), Some(MapHealth::Absent));
        assert_eq!(state(MapRole::Normal), Some(MapHealth::Unreadable));
        assert_eq!(
            survey.len(),
            3,
            "only the maps this material wants: {survey:?}"
        );
    }

    #[test]
    fn a_survey_notices_a_png_of_the_wrong_size() {
        let root = scratch("survey-size");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 5, false).unwrap();

        // A real PNG, decodable, and not this material's.
        let smaller = definition("nexora:material/other", 0.8);
        let produced = ProceduralGenerator::new()
            .unwrap()
            .generate(&GenerationRequest::generate(
                SurfaceMaterial::builder(
                    id("nexora:material/other"),
                    MaterialCategory::Wood,
                    Resolution::square(16).unwrap(),
                    Provenance::authored("operator", "test"),
                )
                .build()
                .unwrap(),
                1,
            ))
            .unwrap();
        let _ = smaller;
        std::fs::write(
            layout::map_file(&root, material.id(), MapRole::Albedo),
            png::encode(produced.maps().get(MapRole::Albedo).unwrap()).unwrap(),
        )
        .unwrap();

        let survey = forge.survey(&material);
        assert_eq!(
            survey
                .iter()
                .find(|(role, _)| *role == MapRole::Albedo)
                .map(|(_, health)| *health),
            Some(MapHealth::WrongSize)
        );
    }

    #[test]
    fn a_repair_restores_a_lost_map_byte_for_byte() {
        // The property the mode exists for. A restored map has to be the one
        // that was lost, not a new one that happens to be the same shape —
        // otherwise it does not match the maps beside it.
        let root = scratch("repair");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 0xfeed, false).unwrap();

        let before: Vec<(MapRole, Vec<u8>)> = [MapRole::Albedo, MapRole::Height, MapRole::Normal]
            .into_iter()
            .map(|role| {
                (
                    role,
                    std::fs::read(layout::map_file(&root, material.id(), role)).unwrap(),
                )
            })
            .collect();

        std::fs::remove_file(layout::map_file(&root, material.id(), MapRole::Normal)).unwrap();
        let outcome = forge.repair(material.id()).expect("the repair runs");
        assert_eq!(outcome.status, WriteStatus::Restored);
        assert_eq!(
            outcome.files.len(),
            1,
            "only the lost map: {:?}",
            outcome.files
        );

        for (role, original) in &before {
            let now = std::fs::read(layout::map_file(&root, material.id(), *role)).unwrap();
            assert_eq!(&now, original, "{} changed", role.as_str());
        }
    }

    #[test]
    fn a_repair_leaves_the_definition_and_the_intact_maps_untouched() {
        let root = scratch("repair-untouched");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 11, false).unwrap();

        let definition_path = layout::definition_file(&root, material.id());
        let written = forge.read_definition(material.id()).unwrap();
        let text = std::fs::read(&definition_path).unwrap();

        std::fs::remove_file(layout::map_file(&root, material.id(), MapRole::Albedo)).unwrap();
        forge.repair(material.id()).unwrap();

        assert_eq!(
            std::fs::read(&definition_path).unwrap(),
            text,
            "a repair that rewrote the definition would claim a change that did not happen"
        );
        assert_eq!(
            forge.read_definition(material.id()).unwrap().revision(),
            written.revision()
        );
    }

    #[test]
    fn repairing_something_intact_does_nothing() {
        let root = scratch("repair-noop");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 2, false).unwrap();

        let outcome = forge.repair(material.id()).unwrap();
        assert_eq!(outcome.status, WriteStatus::Unchanged);
        assert!(outcome.files.is_empty());
        assert_eq!(outcome.byte_len(), 0);
    }

    #[test]
    fn a_repair_restores_several_lost_maps_at_once() {
        let root = scratch("repair-many");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 4, false).unwrap();

        let before: Vec<Vec<u8>> = [MapRole::Albedo, MapRole::Height, MapRole::Normal]
            .into_iter()
            .map(|role| std::fs::read(layout::map_file(&root, material.id(), role)).unwrap())
            .collect();

        std::fs::remove_file(layout::map_file(&root, material.id(), MapRole::Albedo)).unwrap();
        std::fs::write(
            layout::map_file(&root, material.id(), MapRole::Height),
            b"corrupt",
        )
        .unwrap();

        let outcome = forge.repair(material.id()).unwrap();
        assert_eq!(outcome.files.len(), 2);
        for (index, role) in [MapRole::Albedo, MapRole::Height, MapRole::Normal]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                std::fs::read(layout::map_file(&root, material.id(), role)).unwrap(),
                before[index],
                "{} did not come back",
                role.as_str()
            );
        }
    }

    #[test]
    fn a_repaired_material_validates() {
        let root = scratch("repair-validates");
        let forge = Forge::new(&root).unwrap();
        let material = definition("nexora:material/oak", 0.8);
        forge.generate(&material, 9, false).unwrap();
        std::fs::remove_file(layout::map_file(&root, material.id(), MapRole::Normal)).unwrap();

        // Broken first, so the check is known capable of failing.
        let (_, broken) = forge.validate(material.id()).unwrap();
        assert_eq!(broken.verdict(), Verdict::Fail);
        assert!(broken
            .findings()
            .iter()
            .any(|finding| finding.check == Check::FileExists
                && finding.severity == Severity::Failure));

        forge.repair(material.id()).unwrap();
        let (_, healed) = forge.validate(material.id()).unwrap();
        assert_eq!(healed.verdict(), Verdict::Pass);
    }
}
