//! The procedural generator.
//!
//! Implements [`TextureGenerator`] over [`Recipe`]. It is the first backend and
//! the only one that is fully reproducible from its own provenance: given the
//! material, the seed and the generator version, the pixels can be rebuilt
//! exactly, forever, with no model and no network.
//!
//! # What it produces, and what it deliberately leaves to the pipeline
//!
//! Albedo, and the height field the recipe actually authored. Normals,
//! roughness, metallic and ambient occlusion are *derived* from those two, and
//! deriving them is a pipeline's job — a generator that also derived them would
//! have to be reimplemented by every future backend, which is exactly the
//! coupling the trait exists to prevent.
//!
//! # The three modes differ only in where the seed comes from
//!
//! Everything downstream of the seed is one code path, because a variant that
//! rendered differently from a generation would be a second renderer to keep
//! in step.
//!
//! | mode | seed |
//! | --- | --- |
//! | generate | derived from the material's own identity and appearance |
//! | variant | derived from the **source's** identity and the variant index |
//! | repair | read verbatim from the record of the generation being repaired |
//!
//! Each row is a decision worth stating. A **variant** seeds from the source
//! so that "variant 3 of oak" names one specific surface no matter what the
//! result is called — seeding from the new material's own name would make
//! renaming it repaint it. A **repair** does not re-derive at all: it has to
//! land beside maps it did not touch, and a re-derived seed is only equal to
//! the original by coincidence of nothing having changed. Reading the recorded
//! seed makes the restored map identical by construction rather than by luck.

use std::collections::BTreeMap;

use nexora_asset::generator::{
    Backend, GeneratedMaterial, GenerationMode, GenerationRequest, TextureGenerator,
};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::provenance::{GenerationTrace, Provenance};
use nexora_asset::texture::{MapRole, MapSet};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::Fnv1a64;
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::ContentGeneratorVersion;

use crate::recipe::Recipe;

/// The procedural generator's stable identifier.
pub const PROCEDURAL_GENERATOR: &str = "nexora:generator/procedural";

/// The procedural generator's algorithm version.
///
/// Changing anything that changes a pixel must change this number: provenance
/// claims the pixels can be rebuilt, and that claim is only true while the
/// version and the algorithm agree.
pub const PROCEDURAL_VERSION: ContentGeneratorVersion = ContentGeneratorVersion(1);

/// The tool string recorded in every asset this crate produces.
pub const TOOL: &str = concat!("nexora-texture-forge@", env!("CARGO_PKG_VERSION"));

/// Who authored the content this generator produces.
pub const AUTHOR: &str = "NEXORA";

/// Turns a material definition into pixels, from numbers alone.
#[derive(Debug, Clone)]
pub struct ProceduralGenerator {
    id: Identifier,
}

impl ProceduralGenerator {
    /// Build the generator.
    ///
    /// # Errors
    ///
    /// Returns an error only if [`PROCEDURAL_GENERATOR`] stops being a valid
    /// identifier, which would be a build-time mistake.
    pub fn new() -> Result<Self> {
        Ok(Self {
            id: Identifier::parse(PROCEDURAL_GENERATOR)?,
        })
    }

    /// The seed this request actually renders with.
    ///
    /// Derived from the material's identity and appearance as well as the
    /// requested seed, so that two different materials asked for with seed `0`
    /// do not come out identical — which is what would happen if the request's
    /// seed were used directly, and it is the kind of thing nobody notices
    /// until the whole world is made of the same stone.
    #[must_use]
    pub fn effective_seed(definition: &SurfaceMaterial, requested: u64) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str(&definition.id().to_string());
        hasher.write_u64(definition.appearance_hash());
        hasher.write_u64(u64::from(PROCEDURAL_VERSION.0));
        hasher.write_u64(requested);
        hasher.finish()
    }

    /// The seed a variant renders with.
    ///
    /// Deliberately independent of the variant's own identifier: two materials
    /// both claiming to be variant 3 of `nexora:material/oak` *are* the same
    /// surface, and naming one of them differently should not change its
    /// pixels. The index is mixed in rather than added, so consecutive
    /// variants are unrelated rather than neighbouring.
    #[must_use]
    pub fn variant_seed(of: &Identifier, index: u32, requested: u64) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str("variant");
        hasher.write_str(&of.to_string());
        hasher.write_u64(u64::from(index));
        hasher.write_u64(u64::from(PROCEDURAL_VERSION.0));
        hasher.write_u64(requested);
        hasher.finish()
    }

    /// The seed a repair renders with: the one the record already names.
    ///
    /// # Errors
    ///
    /// Returns an error when the material carries no generation record, when
    /// that record names a different generator, or when it names a version
    /// this build is not. Each is a case where the pixels this build would
    /// produce are not the pixels being repaired, and producing them anyway
    /// would leave a material half from one algorithm and half from another.
    fn recorded_seed(&self, definition: &SurfaceMaterial) -> Result<u64> {
        let Some(trace) = definition.provenance().generation.as_ref() else {
            return Err(unsupported(
                "a repair needs the record that says how the material was made, and there is none",
            )
            .with_context("material", definition.id().to_string()));
        };
        if trace.generator != self.id {
            return Err(
                unsupported("this generator cannot reproduce what another one made")
                    .with_context("recorded", trace.generator.to_string())
                    .with_context("this", self.id.to_string()),
            );
        }
        if trace.generator_version != PROCEDURAL_VERSION {
            return Err(unsupported(
                "this build's algorithm is not the one that made the material",
            )
            .with_context("recorded", trace.generator_version.to_string())
            .with_context("this", PROCEDURAL_VERSION.to_string()));
        }
        Ok(trace.seed)
    }

    /// The seed this request renders with, whichever mode it is.
    ///
    /// # Errors
    ///
    /// Returns an error only in repair mode; see [`Self::recorded_seed`].
    fn seed_for(&self, request: &GenerationRequest) -> Result<u64> {
        match &request.mode {
            GenerationMode::Generate => Ok(Self::effective_seed(&request.definition, request.seed)),
            GenerationMode::Variant { of, index } => {
                Ok(Self::variant_seed(of, *index, request.seed))
            }
            GenerationMode::Repair { .. } => self.recorded_seed(&request.definition),
        }
    }

    fn trace(
        &self,
        definition: &SurfaceMaterial,
        request: &GenerationRequest,
    ) -> Result<Provenance> {
        let mut parameters: BTreeMap<String, String> = BTreeMap::new();
        parameters.insert(
            "category".to_owned(),
            definition.category().as_str().to_owned(),
        );
        parameters.insert("mode".to_owned(), request.mode.as_str().to_owned());
        parameters.insert(
            "resolution".to_owned(),
            format!(
                "{}x{}",
                definition.resolution().width,
                definition.resolution().height
            ),
        );
        parameters.insert(
            "requested_seed".to_owned(),
            format!("{:#018x}", request.seed),
        );
        parameters.insert(
            "appearance".to_owned(),
            format!("{:#018x}", definition.appearance_hash()),
        );
        for (key, value) in &request.parameters {
            parameters.insert(key.clone(), value.clone());
        }

        if let GenerationMode::Variant { index, .. } = &request.mode {
            parameters.insert("variant_index".to_owned(), index.to_string());
        }

        // The seed recorded is the one that rendered, not the one requested —
        // that is what makes the record enough to rebuild the pixels, and what
        // a later repair reads back.
        let mut trace =
            GenerationTrace::new(self.id.clone(), PROCEDURAL_VERSION, self.seed_for(request)?)
                .from_preset(Identifier::nexora(&format!(
                    "preset/{}",
                    definition.category().as_str()
                ))?);
        trace.backend = Backend::Procedural;
        trace.parameters = parameters;
        if let GenerationMode::Variant { of, .. } | GenerationMode::Repair { of } = &request.mode {
            trace.inputs.push(of.clone());
        }

        Ok(Provenance::generated(AUTHOR, TOOL, trace))
    }
}

/// Which maps this generator produces for a request.
///
/// Albedo always, because a material without colour is not a material. Height
/// when it is asked for, or when a map that is derived from it is — a request
/// for a normal map with no height field to build it from would otherwise fail
/// one stage later, with a less useful message.
fn produced_roles(request: &GenerationRequest) -> Vec<MapRole> {
    let requested = request.requested_roles();
    let needs_height = requested.iter().any(|role| {
        matches!(
            role,
            MapRole::Height | MapRole::Normal | MapRole::AmbientOcclusion
        )
    });
    let mut roles = vec![MapRole::Albedo];
    if needs_height {
        roles.push(MapRole::Height);
    }
    roles
}

impl TextureGenerator for ProceduralGenerator {
    fn id(&self) -> &Identifier {
        &self.id
    }

    fn version(&self) -> ContentGeneratorVersion {
        PROCEDURAL_VERSION
    }

    fn backend(&self) -> Backend {
        Backend::Procedural
    }

    fn supports(&self, _mode: &GenerationMode) -> bool {
        // All three. The modes differ in where the seed comes from and in
        // nothing else, so there is no mode this backend can render but not
        // serve.
        true
    }

    fn generate(&self, request: &GenerationRequest) -> Result<GeneratedMaterial> {
        let definition = request.definition.clone();
        let recipe = Recipe::for_category(definition.category())?;
        let seed = self.seed_for(request)?;
        let canvas = recipe.render(definition.resolution(), seed);

        let mut maps = MapSet::new();
        for role in produced_roles(request) {
            let map = match role {
                MapRole::Albedo => canvas.albedo_map()?,
                MapRole::Height => canvas.height_map()?,
                // `produced_roles` returns only those two. A third would be a
                // bug here rather than a caller's mistake, so it is loud.
                other => {
                    return Err(unsupported("this generator does not produce that map")
                        .with_context("map", other.as_str()))
                }
            };
            maps.insert(map)?;
        }

        let attributed = definition.revised(self.trace(&definition, request)?)?;
        GeneratedMaterial::assemble(self, attributed, maps)
    }
}

fn unsupported(message: &'static str) -> Error {
    Error::new(Domain::Content, "procedural-generator", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::MaterialCategory;
    use nexora_asset::provenance::ProvenanceClass;
    use nexora_asset::texture::Resolution;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn definition(path: &str, category: MaterialCategory, edge: u32) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id(path),
            category,
            Resolution::square(edge).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .wants(MapRole::Height)
        .build()
        .expect("valid definition")
    }

    fn generate(definition: SurfaceMaterial, seed: u64) -> GeneratedMaterial {
        ProceduralGenerator::new()
            .unwrap()
            .generate(&GenerationRequest::generate(definition, seed))
            .expect("generation succeeds")
    }

    #[test]
    fn the_same_definition_and_seed_produce_identical_pixels() {
        let first = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 32),
            7,
        );
        let second = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 32),
            7,
        );

        assert_eq!(
            first.maps().get(MapRole::Albedo).unwrap().pixels(),
            second.maps().get(MapRole::Albedo).unwrap().pixels()
        );
        assert_eq!(
            first.maps().get(MapRole::Height).unwrap().pixels(),
            second.maps().get(MapRole::Height).unwrap().pixels()
        );
        // The origin record agrees too, timestamp aside.
        assert_eq!(
            first.definition().provenance().content_hash(),
            second.definition().provenance().content_hash()
        );
    }

    #[test]
    fn a_different_seed_produces_a_different_surface() {
        let a = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 32),
            1,
        );
        let b = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 32),
            2,
        );
        assert_ne!(
            a.maps().get(MapRole::Albedo).unwrap().pixels(),
            b.maps().get(MapRole::Albedo).unwrap().pixels()
        );
    }

    #[test]
    fn two_materials_asked_for_with_the_same_seed_are_not_the_same_texture() {
        // The trap: a caller passing seed 0 for everything. If the request's
        // seed were used directly, the whole world would be one surface.
        let oak = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 32),
            0,
        );
        let pine = generate(
            definition("nexora:material/pine", MaterialCategory::Wood, 32),
            0,
        );
        assert_ne!(
            oak.maps().get(MapRole::Albedo).unwrap().pixels(),
            pine.maps().get(MapRole::Albedo).unwrap().pixels()
        );
    }

    #[test]
    fn every_category_produces_a_material_that_validates() {
        for category in MaterialCategory::ALL {
            let generated = generate(
                definition("nexora:material/sample", category, 32),
                0xC0FF_EE00,
            );
            generated
                .validate_completeness()
                .ok("nexora:material/sample")
                .unwrap_or_else(|err| panic!("{category:?} must be complete: {err}"));

            let albedo = generated.maps().get(MapRole::Albedo).unwrap();
            assert_eq!(albedo.byte_len(), 32 * 32 * 4);
            // Not a flat fill: a texture with one colour is a bug that would
            // otherwise pass every structural check there is.
            let distinct: std::collections::BTreeSet<&[u8]> =
                albedo.pixels().chunks_exact(4).collect();
            assert!(
                distinct.len() > 2,
                "{category:?} produced {} distinct colours",
                distinct.len()
            );
        }
    }

    #[test]
    fn the_output_carries_a_trace_that_names_this_generator_and_its_inputs() {
        let generated = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 16),
            5,
        );
        let provenance = generated.definition().provenance();
        provenance.validate().expect("the record must be valid");

        assert_eq!(provenance.class, ProvenanceClass::ProceduralDerivative);
        assert_eq!(provenance.source_tool, TOOL);
        assert!(provenance.source_tool.starts_with("nexora-texture-forge@"));

        let trace = provenance.generation.as_ref().unwrap();
        assert_eq!(trace.generator.to_string(), PROCEDURAL_GENERATOR);
        assert_eq!(trace.generator_version, PROCEDURAL_VERSION);
        assert_eq!(
            trace.preset.as_ref().unwrap().to_string(),
            "nexora:preset/wood"
        );
        assert_eq!(trace.parameters["category"], "wood");
        assert_eq!(trace.parameters["resolution"], "16x16");
        // A field on the trace, not an entry in the parameters map: what
        // decides whether an asset may ship is not a string any generator may
        // forget to write.
        assert_eq!(trace.backend, Backend::Procedural);
        assert!(!trace.parameters.contains_key("backend"));
        // No prompt: this backend takes numbers, and the record says so.
        assert!(trace.prompt.is_none());
    }

    #[test]
    fn generating_advances_the_revision_rather_than_overwriting_the_definition() {
        let authored = definition("nexora:material/oak", MaterialCategory::Wood, 16);
        assert_eq!(authored.revision().0, 1);
        assert!(authored.provenance().generation.is_none());

        let generated = generate(authored.clone(), 1);
        assert_eq!(generated.definition().revision().0, 2);
        // The original is untouched, which is what §15 asks for.
        assert_eq!(authored.revision().0, 1);
        assert!(authored.provenance().generation.is_none());
    }

    // ---- variant and repair -------------------------------------------

    #[test]
    fn every_mode_is_served() {
        let generator = ProceduralGenerator::new().unwrap();
        for mode in [
            GenerationMode::Generate,
            GenerationMode::Variant {
                of: id("nexora:material/oak"),
                index: 1,
            },
            GenerationMode::Repair {
                of: id("nexora:material/oak"),
            },
        ] {
            assert!(generator.supports(&mode), "{}", mode.as_str());
        }
    }

    #[test]
    fn a_variant_is_the_same_recipe_at_a_different_seed() {
        let generator = ProceduralGenerator::new().unwrap();
        let source = definition("nexora:material/oak", MaterialCategory::Wood, 32);
        let original = generator
            .generate(&GenerationRequest::generate(source.clone(), 0))
            .unwrap();

        let derived = definition("nexora:material/oak_weathered", MaterialCategory::Wood, 32);
        let variant = generator
            .generate(&GenerationRequest::variant(
                derived,
                source.id().clone(),
                1,
                0,
            ))
            .unwrap();

        let a = original.maps().get(MapRole::Albedo).unwrap().pixels();
        let b = variant.maps().get(MapRole::Albedo).unwrap().pixels();
        assert_ne!(a, b, "a variant that matches its source is not a variant");
        assert_eq!(a.len(), b.len(), "and it is still the same recipe and size");
    }

    #[test]
    fn variants_of_one_source_differ_from_each_other() {
        let generator = ProceduralGenerator::new().unwrap();
        let source = id("nexora:material/oak");
        let pixels: Vec<Vec<u8>> = (1..=4)
            .map(|index| {
                let definition = definition(
                    &format!("nexora:material/oak_{index}"),
                    MaterialCategory::Wood,
                    16,
                );
                generator
                    .generate(&GenerationRequest::variant(
                        definition,
                        source.clone(),
                        index,
                        0,
                    ))
                    .unwrap()
                    .maps()
                    .get(MapRole::Albedo)
                    .unwrap()
                    .pixels()
                    .to_vec()
            })
            .collect();

        for (left, one) in pixels.iter().enumerate() {
            for (right, other) in pixels.iter().enumerate().skip(left + 1) {
                assert_ne!(one, other, "variant {} and variant {}", left + 1, right + 1);
            }
        }
    }

    #[test]
    fn a_variant_is_named_by_its_source_and_index_not_by_its_own_name() {
        // "variant 3 of oak" is one specific surface. Calling the result
        // something else must not repaint it — otherwise renaming a material
        // silently changes the world.
        let generator = ProceduralGenerator::new().unwrap();
        let source = id("nexora:material/oak");
        let under_one_name = generator
            .generate(&GenerationRequest::variant(
                definition("nexora:material/oak_a", MaterialCategory::Wood, 16),
                source.clone(),
                3,
                0,
            ))
            .unwrap();
        let under_another = generator
            .generate(&GenerationRequest::variant(
                definition(
                    "nexora:material/completely_different",
                    MaterialCategory::Wood,
                    16,
                ),
                source,
                3,
                0,
            ))
            .unwrap();

        assert_eq!(
            under_one_name.maps().get(MapRole::Albedo).unwrap().pixels(),
            under_another.maps().get(MapRole::Albedo).unwrap().pixels()
        );
    }

    #[test]
    fn a_variant_records_what_it_was_varied_from() {
        let generator = ProceduralGenerator::new().unwrap();
        let source = id("nexora:material/oak");
        let produced = generator
            .generate(&GenerationRequest::variant(
                definition("nexora:material/oak_2", MaterialCategory::Wood, 16),
                source.clone(),
                2,
                0,
            ))
            .unwrap();

        let trace = produced
            .definition()
            .provenance()
            .generation
            .as_ref()
            .expect("a generated material carries its record");
        assert_eq!(trace.inputs, vec![source]);
        assert_eq!(
            trace.parameters.get("mode").map(String::as_str),
            Some("variant")
        );
        assert_eq!(
            trace.parameters.get("variant_index").map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn a_repair_reproduces_the_original_pixels_exactly() {
        // The property the whole mode exists for: a restored map has to sit
        // beside the ones that were never lost.
        let generator = ProceduralGenerator::new().unwrap();
        let original = generator
            .generate(&GenerationRequest::generate(
                definition("nexora:material/oak", MaterialCategory::Wood, 32),
                0xabc_def,
            ))
            .unwrap();

        // What a repair starts from is what is on disk: the *generated*
        // definition, carrying its record.
        let repaired = generator
            .generate(&GenerationRequest::repair(original.definition().clone()))
            .unwrap();

        for role in [MapRole::Albedo, MapRole::Height] {
            assert_eq!(
                original.maps().get(role).unwrap().pixels(),
                repaired.maps().get(role).unwrap().pixels(),
                "{} came back different",
                role.as_str()
            );
        }
    }

    #[test]
    fn a_repair_does_not_re_derive_the_seed_it_reads_the_recorded_one() {
        // A repair request carries no seed of its own — `GenerationRequest`
        // sets zero, because the seed is not the caller's to choose. So a
        // generator that re-derived would derive from zero and land somewhere
        // else entirely. That is what the second assertion pins down.
        let generator = ProceduralGenerator::new().unwrap();
        let original = generator
            .generate(&GenerationRequest::generate(
                definition("nexora:material/oak", MaterialCategory::Wood, 32),
                7,
            ))
            .unwrap();
        let recorded = original
            .definition()
            .provenance()
            .generation
            .as_ref()
            .unwrap()
            .seed;

        let repaired = generator
            .generate(&GenerationRequest::repair(original.definition().clone()))
            .unwrap();
        let used = repaired
            .definition()
            .provenance()
            .generation
            .as_ref()
            .unwrap()
            .seed;

        assert_eq!(used, recorded, "the repair rendered with a different seed");
        assert_ne!(
            used,
            ProceduralGenerator::effective_seed(original.definition(), 0),
            "re-deriving from the repair request would have given this, so the \
             assertion above is capable of failing"
        );
    }

    #[test]
    fn a_repair_of_a_material_with_no_record_is_refused() {
        // There is nothing to reproduce *from*, and inventing a seed would
        // quietly repaint the material.
        let generator = ProceduralGenerator::new().unwrap();
        let err = generator
            .generate(&GenerationRequest::repair(definition(
                "nexora:material/oak",
                MaterialCategory::Wood,
                16,
            )))
            .expect_err("a repair without a record must say so");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("record"), "{err}");
    }

    #[test]
    fn a_repair_of_something_another_generator_made_is_refused() {
        let generator = ProceduralGenerator::new().unwrap();
        let mut produced = generator
            .generate(&GenerationRequest::generate(
                definition("nexora:material/oak", MaterialCategory::Wood, 16),
                1,
            ))
            .unwrap()
            .definition()
            .clone();

        let mut provenance = produced.provenance().clone();
        let mut trace = provenance.generation.clone().unwrap();
        trace.generator = id("nexora:generator/some_model");
        provenance.generation = Some(trace);
        produced = produced.revised(provenance).unwrap();

        let err = generator
            .generate(&GenerationRequest::repair(produced))
            .expect_err("this build cannot reproduce another generator's pixels");
        assert!(err.to_string().contains("another one made"), "{err}");
    }

    #[test]
    fn height_appears_only_when_it_or_something_derived_from_it_is_wanted() {
        let generator = ProceduralGenerator::new().unwrap();
        let plain = SurfaceMaterial::builder(
            id("nexora:material/flat"),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .build()
        .unwrap();

        let only_albedo = generator
            .generate(&GenerationRequest::generate(plain.clone(), 1))
            .unwrap();
        assert_eq!(only_albedo.maps().len(), 1);
        assert!(!only_albedo.maps().contains(MapRole::Height));

        // A normal map is derived from height, so height is produced for it.
        let with_normal = generator
            .generate(&GenerationRequest::generate(plain, 1).only(vec![MapRole::Normal]))
            .unwrap();
        assert!(with_normal.maps().contains(MapRole::Height));
    }

    #[test]
    fn every_category_tiles_on_both_axes() {
        // One seam measure in this crate, not two: `validator::seam_ratio` and
        // its threshold are what the validator holds finished materials to,
        // and they are what the generator is held to here.
        use crate::validator::{seam_ratio, MAX_SEAM_RATIO};
        for category in MaterialCategory::ALL {
            for seed in 0..4u64 {
                let generated = generate(definition("nexora:material/tile", category, 64), seed);
                let albedo = generated.maps().get(MapRole::Albedo).unwrap();
                for vertical in [false, true] {
                    let ratio = seam_ratio(albedo, vertical);
                    assert!(
                        ratio <= MAX_SEAM_RATIO,
                        "{category:?} seed {seed} {} seam ratio {ratio:.3}",
                        if vertical { "vertical" } else { "horizontal" }
                    );
                }
            }
        }
    }

    #[test]
    fn a_plank_texture_actually_has_planks_in_it() {
        // Structure, asserted rather than eyeballed. Four boards across the
        // texture puts a gap at every quarter and a board centre halfway
        // between two gaps, so the gap column must be darker than the board
        // column that follows it — four times over.
        let generated = generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 64),
            9,
        );
        let albedo = generated.maps().get(MapRole::Albedo).unwrap();
        let width = 64usize;

        let column_luma = |x: usize| -> f64 {
            let sum: u32 = (0..width)
                .map(|y| {
                    let at = (y * width + x) * 4;
                    u32::from(albedo.pixels()[at])
                        + u32::from(albedo.pixels()[at + 1])
                        + u32::from(albedo.pixels()[at + 2])
                })
                .sum();
            f64::from(sum) / (width as f64 * 3.0 * 255.0)
        };

        let mut board_centres = Vec::new();
        for board in 0..4 {
            let gap = column_luma(board * 16);
            let centre = column_luma(board * 16 + 8);
            assert!(
                gap < centre,
                "board {board}: the gap at {} is not darker than the board at {}",
                board * 16,
                board * 16 + 8
            );
            board_centres.push(centre);
        }

        // And the boards are not all the same shade, or they would read as one
        // panel with lines drawn on it.
        let spread = board_centres.iter().copied().fold(f64::MIN, f64::max)
            - board_centres.iter().copied().fold(f64::MAX, f64::min);
        assert!(spread > 0.05, "boards differ by only {spread:.3}");
    }

    #[test]
    fn a_surface_uses_more_than_the_middle_of_its_own_palette() {
        // The failure this caught during development: summed value noise
        // clusters around 0.5, so without the contrast gain every material came
        // out as the middle two stops of its ramp and looked washed out.
        for category in MaterialCategory::ALL {
            let generated = generate(definition("nexora:material/sample", category, 64), 4);
            let albedo = generated.maps().get(MapRole::Albedo).unwrap();
            let (mut darkest, mut brightest) = (255u8, 0u8);
            for texel in albedo.pixels().chunks_exact(4) {
                let luma = texel[0] / 3 + texel[1] / 3 + texel[2] / 3;
                darkest = darkest.min(luma);
                brightest = brightest.max(luma);
            }
            assert!(
                brightest - darkest > 24,
                "{category:?} spans only {} levels",
                brightest - darkest
            );
        }
    }
}
