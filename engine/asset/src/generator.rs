//! The boundary a generator crosses, and what it must hand back.
//!
//! The Texture Forge brief §4 asks that an image-generation API, a local model,
//! Stable Diffusion, Flux, another backend or a procedural generator can all be
//! added *"without rewriting the core"*. That is a request for a trait, and
//! [`TextureGenerator`] is it.
//!
//! # Provenance is enforced here, not reviewed later
//!
//! §17 says an asset must never pass as NEXORA's own when it is not. A rule
//! like that, left to review, is a rule that holds until the week somebody is
//! busy. So it is structural instead:
//!
//! * a [`crate::SurfaceMaterial`] cannot be built without a valid
//!   [`crate::Provenance`];
//! * a [`GeneratedMaterial`] cannot be built unless that record names **the
//!   generator that produced it**, at the version that produced it.
//!
//! A new backend therefore cannot return pixels attributed to nothing, or
//! attributed to somebody else. The check is one function, and it runs on every
//! generated material regardless of which backend made it.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::{ContentGeneratorVersion, ContentPipelineVersion};

use crate::material::SurfaceMaterial;
use crate::texture::{MapRole, MapSet, MapStatus, Preview};
use crate::validation::{Check, Finding, TextureValidationResult};

/// What a generation run is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationMode {
    /// Produce a material from its definition.
    Generate,
    /// Produce another material like an existing one, differing by seed.
    Variant {
        /// The material being varied.
        of: Identifier,
        /// Which variant this is, counted from one.
        index: u32,
    },
    /// Reproduce the maps of an existing material that are missing or invalid.
    ///
    /// Deliberately not the same as regenerating: a repair keeps the maps that
    /// are fine, so a material does not silently change appearance because one
    /// of its six files was corrupt.
    Repair {
        /// The material being repaired.
        of: Identifier,
    },
}

impl GenerationMode {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Generate => "generate",
            Self::Variant { .. } => "variant",
            Self::Repair { .. } => "repair",
        }
    }
}

/// How the pixels are produced.
///
/// The three from the brief §4. This is recorded rather than inferred: an
/// asset's origin must be answerable without reading the generator's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Backend {
    /// An algorithm, from parameters. Reproducible from the trace alone.
    Procedural,
    /// A model. Reproducible only as far as the model is.
    Ai,
    /// A model's output shaped by an algorithm, or the reverse.
    Hybrid,
}

impl Backend {
    /// Every backend, in a stable order.
    pub const ALL: [Self; 3] = [Self::Procedural, Self::Ai, Self::Hybrid];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Procedural => "procedural",
            Self::Ai => "ai",
            Self::Hybrid => "hybrid",
        }
    }

    /// Parse the stable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not a known backend.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|backend| backend.as_str() == raw)
            .ok_or_else(|| {
                invalid("backend is not recognised").with_context("value", raw.to_owned())
            })
    }

    /// Whether output from this backend is reproducible from its trace alone.
    ///
    /// Only the procedural one is. Recording that difference is what stops a
    /// model's output being treated as though re-running the tool would bring
    /// it back.
    #[must_use]
    pub const fn is_reproducible_from_trace(self) -> bool {
        matches!(self, Self::Procedural)
    }
}

/// One request to generate.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationRequest {
    /// The definition to realise.
    pub definition: SurfaceMaterial,
    /// What the run is for.
    pub mode: GenerationMode,
    /// Which backend the caller asked for.
    pub backend: Backend,
    /// The seed. The same seed and the same definition give the same pixels.
    pub seed: u64,
    /// Which maps to produce. Empty means "what the definition asks for".
    pub roles: Vec<MapRole>,
    /// A natural-language request, for backends that take one.
    pub prompt: Option<String>,
    /// Anything else the backend understands, recorded in the trace.
    pub parameters: BTreeMap<String, String>,
}

impl GenerationRequest {
    /// A plain generation run for a definition.
    #[must_use]
    pub fn generate(definition: SurfaceMaterial, seed: u64) -> Self {
        Self {
            definition,
            mode: GenerationMode::Generate,
            backend: Backend::Procedural,
            seed,
            roles: Vec::new(),
            prompt: None,
            parameters: BTreeMap::new(),
        }
    }

    /// A run producing another material like an existing one.
    ///
    /// `of` is the material being varied and `index` counts from one. A
    /// backend is expected to derive its seed from those two rather than from
    /// the new material's own identity, so that "variant 3 of oak" names one
    /// specific surface however the caller chooses to name the result.
    #[must_use]
    pub fn variant(definition: SurfaceMaterial, of: Identifier, index: u32, seed: u64) -> Self {
        Self {
            definition,
            mode: GenerationMode::Variant { of, index },
            backend: Backend::Procedural,
            seed,
            roles: Vec::new(),
            prompt: None,
            parameters: BTreeMap::new(),
        }
    }

    /// A run reproducing maps of an existing material that were lost.
    ///
    /// Carries no seed of its own on purpose. A repair must land beside the
    /// maps it did not touch, so the seed is the one in `definition`'s own
    /// generation record — a backend that invented a fresh one would restore a
    /// normal map that does not match the albedo next to it.
    ///
    /// It carries no role list either, and that is not an omission. A derived
    /// map cannot be rebuilt without the map it derives from, so restricting
    /// the run to "just the normal map" would leave the pipeline with no height
    /// field to build it out of. A repair therefore renders the whole set;
    /// which files are then *written* is the caller's business, and writing
    /// only the lost ones is what makes it a repair.
    #[must_use]
    pub fn repair(definition: SurfaceMaterial) -> Self {
        let of = definition.id().clone();
        Self {
            definition,
            mode: GenerationMode::Repair { of },
            backend: Backend::Procedural,
            seed: 0,
            roles: Vec::new(),
            prompt: None,
            parameters: BTreeMap::new(),
        }
    }

    /// Which maps this run should produce.
    ///
    /// An explicit list wins; otherwise it is the required maps plus whatever
    /// the definition says it wants.
    #[must_use]
    pub fn requested_roles(&self) -> Vec<MapRole> {
        if !self.roles.is_empty() {
            let mut roles = self.roles.clone();
            roles.sort_unstable();
            roles.dedup();
            return roles;
        }
        let mut roles: Vec<MapRole> = MapRole::ALL
            .into_iter()
            .filter(|role| role.is_required() || self.definition.wanted_maps().contains(role))
            .collect();
        roles.sort_unstable();
        roles
    }

    /// Set the backend.
    #[must_use]
    pub const fn with_backend(mut self, backend: Backend) -> Self {
        self.backend = backend;
        self
    }

    /// Restrict the run to specific maps.
    #[must_use]
    pub fn only(mut self, roles: Vec<MapRole>) -> Self {
        self.roles = roles;
        self
    }
}

/// A material and the pixels that realise it.
///
/// The pair the registry, the validator and the writer all consume. Constructed
/// only through [`GeneratedMaterial::assemble`], which refuses anything whose
/// origin record does not match the generator that produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedMaterial {
    definition: SurfaceMaterial,
    maps: MapSet,
    preview: Option<Preview>,
}

impl GeneratedMaterial {
    /// Assemble a result, checking that it is attributable and self-consistent.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition carries no generation trace, when
    /// the trace names a different generator or version than the one that
    /// produced it, or when a map's resolution disagrees with the definition.
    /// Each of those is a wrong asset that would otherwise be indistinguishable
    /// from a right one.
    pub fn assemble(
        generator: &dyn TextureGenerator,
        definition: SurfaceMaterial,
        maps: MapSet,
    ) -> Result<Self> {
        let Some(trace) = definition.provenance().generation.as_ref() else {
            return Err(
                unattributable("generated output carries no generation trace")
                    .with_context("material", definition.id().to_string()),
            );
        };
        if trace.generator != *generator.id() {
            return Err(unattributable(
                "the trace names a different generator than the one that ran",
            )
            .with_context("material", definition.id().to_string())
            .with_context("recorded", trace.generator.to_string())
            .with_context("actual", generator.id().to_string()));
        }
        if trace.generator_version != generator.version() {
            return Err(
                unattributable("the trace names a different version of this generator")
                    .with_context("material", definition.id().to_string())
                    .with_context("recorded", trace.generator_version.to_string())
                    .with_context("actual", generator.version().to_string()),
            );
        }

        let expected = definition.resolution();
        for map in maps.iter() {
            if map.resolution() != expected {
                return Err(invalid("a map does not match the material's resolution")
                    .with_context("material", definition.id().to_string())
                    .with_context("map", map.role().as_str())
                    .with_context(
                        "expected",
                        format!("{}x{}", expected.width, expected.height),
                    )
                    .with_context(
                        "found",
                        format!("{}x{}", map.resolution().width, map.resolution().height),
                    ));
            }
        }

        Ok(Self {
            definition,
            maps,
            preview: None,
        })
    }

    /// Reassemble after a pipeline transformed the material.
    ///
    /// The counterpart of [`Self::assemble`], and it enforces the same rule
    /// from the other side: a pipeline cannot hand back output whose record
    /// names a different pipeline, or a different version of itself. Between
    /// the two, every map in the system is traceable to the generator that
    /// drew it and the pipeline that derived it.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition carries no generation trace, when
    /// the trace does not name this pipeline at this version, or when a map's
    /// resolution disagrees with the definition.
    pub fn transformed(
        pipeline: &dyn TexturePipeline,
        definition: SurfaceMaterial,
        maps: MapSet,
        preview: Option<Preview>,
    ) -> Result<Self> {
        let Some(trace) = definition.provenance().generation.as_ref() else {
            return Err(
                unattributable("transformed output carries no generation trace")
                    .with_context("material", definition.id().to_string()),
            );
        };
        if trace.pipeline.as_ref() != Some(pipeline.id()) {
            return Err(
                unattributable("the trace does not name the pipeline that ran")
                    .with_context("material", definition.id().to_string())
                    .with_context(
                        "recorded",
                        trace
                            .pipeline
                            .as_ref()
                            .map_or_else(|| "none".to_owned(), ToString::to_string),
                    )
                    .with_context("actual", pipeline.id().to_string()),
            );
        }
        if trace.pipeline_version != Some(pipeline.version()) {
            return Err(
                unattributable("the trace names a different version of this pipeline")
                    .with_context("material", definition.id().to_string())
                    .with_context("actual", pipeline.version().to_string()),
            );
        }

        let expected = definition.resolution();
        for map in maps.iter() {
            if map.resolution() != expected {
                return Err(invalid("a map does not match the material's resolution")
                    .with_context("material", definition.id().to_string())
                    .with_context("map", map.role().as_str()));
            }
        }

        Ok(Self {
            definition,
            maps,
            preview,
        })
    }

    /// Take the material apart, so a pipeline can add to it.
    ///
    /// Consuming rather than borrowing: a pipeline produces a new material
    /// from an old one, and handing out the pieces by value is what stops the
    /// old one being used as though it were still current.
    #[must_use]
    pub fn into_parts(self) -> (SurfaceMaterial, MapSet, Option<Preview>) {
        (self.definition, self.maps, self.preview)
    }

    /// The definition this realises.
    #[must_use]
    pub const fn definition(&self) -> &SurfaceMaterial {
        &self.definition
    }

    /// The maps produced.
    #[must_use]
    pub const fn maps(&self) -> &MapSet {
        &self.maps
    }

    /// The preview image, when one has been rendered.
    #[must_use]
    pub const fn preview(&self) -> Option<&Preview> {
        self.preview.as_ref()
    }

    /// Attach a preview image.
    ///
    /// # Errors
    ///
    /// Returns an error when a preview is already attached.
    pub fn attach_preview(&mut self, preview: Preview) -> Result<()> {
        if self.preview.is_some() {
            return Err(invalid("this material already has a preview"));
        }
        self.preview = Some(preview);
        Ok(())
    }

    /// Total bytes of pixel data held, preview included.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.maps.byte_len() + self.preview.as_ref().map_or(0, Preview::byte_len)
    }

    /// Where every map stands.
    #[must_use]
    pub fn map_status(&self) -> BTreeMap<MapRole, MapStatus> {
        self.maps.status(self.definition.wanted_maps())
    }

    /// Check the set against the definition.
    ///
    /// Covers only what this crate can see: whether the maps the material needs
    /// are present. Whether the pixels are *good* — seamless, unit-length
    /// normals, in range — is the forge's job, and its findings merge into this
    /// result.
    pub fn validate_completeness(&self) -> TextureValidationResult {
        let mut result = TextureValidationResult::new();
        for (role, status) in self.map_status() {
            match status {
                MapStatus::Required => result.push(
                    Finding::failure(
                        Check::MissingMap,
                        format!("the material needs a {} map and has none", role.as_str()),
                    )
                    .about(role),
                ),
                MapStatus::Optional => result.push(
                    Finding::warning(
                        Check::MissingMap,
                        format!(
                            "the material asked for a {} map and has none",
                            role.as_str()
                        ),
                    )
                    .about(role),
                ),
                MapStatus::Generated | MapStatus::Missing | MapStatus::Invalid => {}
            }
        }
        result
    }
}

/// Something that turns a definition into pixels.
///
/// The extension point. A procedural algorithm, a hosted image API, a local
/// model and a hybrid of two of those all implement this and nothing else
/// changes — which is the whole point of declaring it.
pub trait TextureGenerator {
    /// This generator's stable identifier, e.g. `nexora:generator/procedural`.
    fn id(&self) -> &Identifier;

    /// This generator's algorithm version, recorded in every trace it produces.
    fn version(&self) -> ContentGeneratorVersion;

    /// How it produces pixels.
    fn backend(&self) -> Backend;

    /// Whether it can serve a mode.
    ///
    /// Defaults to plain generation only. A backend that cannot vary or repair
    /// says so here rather than silently returning a fresh material when asked
    /// to repair one.
    fn supports(&self, mode: &GenerationMode) -> bool {
        matches!(mode, GenerationMode::Generate)
    }

    /// Produce the maps.
    ///
    /// # Errors
    ///
    /// Returns an error when the request asks for something this generator
    /// cannot do, or when generation fails. Failure is never a blank texture:
    /// the brief §23 forbids hiding a generation failure, and a caller that
    /// receives grey pixels instead of an error ships grey pixels.
    fn generate(&self, request: &GenerationRequest) -> Result<GeneratedMaterial>;
}

/// Something that transforms generated pixels.
///
/// Seamless correction, PBR derivation, terrain blending, decal cutting: each
/// is a step that takes a material and returns a material, so they compose in
/// any order the caller needs and none of them knows what produced its input.
pub trait TexturePipeline {
    /// This pipeline's stable identifier, e.g. `nexora:pipeline/pbr`.
    fn id(&self) -> &Identifier;

    /// This pipeline's version, recorded alongside the generator's.
    fn version(&self) -> ContentPipelineVersion;

    /// Transform a material.
    ///
    /// # Errors
    ///
    /// Returns an error when the input is not something this pipeline can
    /// transform.
    fn apply(&self, material: GeneratedMaterial) -> Result<GeneratedMaterial>;
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "generation", message).with_recovery(Recovery::Reject)
}

fn unattributable(message: &'static str) -> Error {
    // Quarantine rather than reject: the pixels may be fine, but an asset whose
    // origin cannot be established must not enter the registry until a person
    // has looked at it.
    Error::new(Domain::Content, "provenance", message).with_recovery(Recovery::Quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MaterialCategory, SurfaceMaterial};
    use crate::provenance::{GenerationTrace, Provenance};
    use crate::texture::{Resolution, TextureFormat, TextureMap};

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    /// A generator that produces flat colour. Enough to exercise the boundary
    /// without pretending to be an art tool.
    struct FlatGenerator {
        id: Identifier,
        version: ContentGeneratorVersion,
    }

    impl FlatGenerator {
        fn new() -> Self {
            Self {
                id: id("nexora:generator/flat"),
                version: ContentGeneratorVersion(1),
            }
        }
    }

    impl TextureGenerator for FlatGenerator {
        fn id(&self) -> &Identifier {
            &self.id
        }
        fn version(&self) -> ContentGeneratorVersion {
            self.version
        }
        fn backend(&self) -> Backend {
            Backend::Procedural
        }
        fn generate(&self, request: &GenerationRequest) -> Result<GeneratedMaterial> {
            let definition = attributed(request.definition.clone(), self.id.clone(), self.version);
            let mut maps = MapSet::new();
            for role in request.requested_roles() {
                maps.insert(flat_map(role, definition.resolution()))?;
            }
            GeneratedMaterial::assemble(self, definition, maps)
        }
    }

    fn flat_map(role: MapRole, resolution: Resolution) -> TextureMap {
        let format = TextureFormat::eight_bit(role.natural_layout());
        let bytes = (resolution.texels() * format.bytes_per_texel().unwrap()) as usize;
        TextureMap::new(role, format, resolution, vec![128; bytes]).expect("well-formed")
    }

    fn attributed(
        definition: SurfaceMaterial,
        generator: Identifier,
        version: ContentGeneratorVersion,
    ) -> SurfaceMaterial {
        let trace = GenerationTrace::new(generator, version, 42);
        definition
            .revised(Provenance::generated("NEXORA", "test", trace))
            .expect("revision")
    }

    fn definition() -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id("nexora:material/stone"),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .wants(MapRole::Roughness)
        .build()
        .expect("valid definition")
    }

    #[test]
    fn a_generator_produces_the_maps_the_definition_asks_for() {
        let generator = FlatGenerator::new();
        let request = GenerationRequest::generate(definition(), 42);
        assert_eq!(
            request.requested_roles(),
            [MapRole::Albedo, MapRole::Roughness]
        );

        let generated = generator.generate(&request).expect("generation succeeds");
        assert_eq!(generated.maps().len(), 2);
        assert!(generated.maps().contains(MapRole::Albedo));
        assert_eq!(generated.byte_len(), 16 * 16 * 4 + 16 * 16);

        let status = generated.map_status();
        assert_eq!(status[&MapRole::Albedo], MapStatus::Generated);
        assert_eq!(status[&MapRole::Metallic], MapStatus::Missing);
        generated
            .validate_completeness()
            .ok("nexora:material/stone")
            .expect("a complete material passes");
    }

    #[test]
    fn output_attributed_to_the_wrong_generator_is_refused() {
        let generator = FlatGenerator::new();
        let impostor = attributed(
            definition(),
            id("nexora:generator/somebody_else"),
            ContentGeneratorVersion(1),
        );
        let err = GeneratedMaterial::assemble(&generator, impostor, MapSet::new())
            .expect_err("a mismatched trace must be refused");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(err.to_string().contains("different generator"), "{err}");
    }

    #[test]
    fn output_attributed_to_the_wrong_version_is_refused() {
        let generator = FlatGenerator::new();
        let stale = attributed(
            definition(),
            id("nexora:generator/flat"),
            ContentGeneratorVersion(99),
        );
        let err = GeneratedMaterial::assemble(&generator, stale, MapSet::new())
            .expect_err("a stale version must be refused");
        assert!(err.to_string().contains("different version"), "{err}");
    }

    #[test]
    fn output_with_no_trace_at_all_is_refused() {
        let generator = FlatGenerator::new();
        // The authored definition has provenance, but no generation trace: it
        // is a recipe that nothing has run yet.
        let err = GeneratedMaterial::assemble(&generator, definition(), MapSet::new())
            .expect_err("unattributed pixels must not exist");
        assert!(err.to_string().contains("no generation trace"), "{err}");
    }

    #[test]
    fn a_map_at_the_wrong_resolution_is_refused() {
        let generator = FlatGenerator::new();
        let definition = attributed(
            definition(),
            id("nexora:generator/flat"),
            ContentGeneratorVersion(1),
        );
        let mut maps = MapSet::new();
        maps.insert(flat_map(MapRole::Albedo, Resolution::square(32).unwrap()))
            .unwrap();

        let err = GeneratedMaterial::assemble(&generator, definition, maps)
            .expect_err("a mismatched map must be refused");
        assert!(err.to_string().contains("resolution"), "{err}");
        assert!(err.to_string().contains("16x16"), "{err}");
    }

    #[test]
    fn a_missing_required_map_fails_completeness_and_an_optional_one_warns() {
        let generator = FlatGenerator::new();
        let request = GenerationRequest::generate(definition(), 42).only(vec![MapRole::Roughness]);
        let generated = generator.generate(&request).unwrap();

        let result = generated.validate_completeness();
        assert_eq!(result.verdict(), crate::validation::Verdict::Fail);
        assert!(result.ok("nexora:material/stone").is_err());

        let request = GenerationRequest::generate(definition(), 42).only(vec![MapRole::Albedo]);
        let generated = generator.generate(&request).unwrap();
        let result = generated.validate_completeness();
        // Roughness was wanted and is absent: a warning, not a failure.
        assert_eq!(result.verdict(), crate::validation::Verdict::Warn);
        assert!(result.is_usable());
    }

    #[test]
    fn a_preview_attaches_once() {
        let generator = FlatGenerator::new();
        let mut generated = generator
            .generate(&GenerationRequest::generate(definition(), 42))
            .unwrap();
        assert!(generated.preview().is_none());

        let preview = Preview::new(Resolution::square(16).unwrap(), vec![0; 16 * 16 * 3]).unwrap();
        generated.attach_preview(preview.clone()).expect("attaches");
        assert!(generated.preview().is_some());
        assert!(generated.attach_preview(preview).is_err());
    }

    #[test]
    fn only_procedural_output_claims_to_be_reproducible_from_its_trace() {
        assert!(Backend::Procedural.is_reproducible_from_trace());
        assert!(!Backend::Ai.is_reproducible_from_trace());
        assert!(!Backend::Hybrid.is_reproducible_from_trace());
        for backend in Backend::ALL {
            assert_eq!(Backend::parse(backend.as_str()).unwrap(), backend);
        }
        assert!(Backend::parse("diffusion").is_err());
    }

    #[test]
    fn a_generator_says_which_modes_it_serves() {
        let generator = FlatGenerator::new();
        assert!(generator.supports(&GenerationMode::Generate));
        assert!(!generator.supports(&GenerationMode::Repair {
            of: id("nexora:material/stone")
        }));
        assert_eq!(GenerationMode::Generate.as_str(), "generate");
        assert_eq!(
            GenerationMode::Variant {
                of: id("nexora:material/stone"),
                index: 3
            }
            .as_str(),
            "variant"
        );
    }

    #[test]
    fn an_explicit_role_list_overrides_what_the_definition_wants() {
        let request = GenerationRequest::generate(definition(), 1)
            .only(vec![MapRole::Height, MapRole::Height, MapRole::Normal])
            .with_backend(Backend::Hybrid);
        assert_eq!(
            request.requested_roles(),
            [MapRole::Normal, MapRole::Height]
        );
        assert_eq!(request.backend, Backend::Hybrid);
    }
}
