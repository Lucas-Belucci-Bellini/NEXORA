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
            "backend".to_owned(),
            Backend::Procedural.as_str().to_owned(),
        );
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

        let mut trace = GenerationTrace::new(
            self.id.clone(),
            PROCEDURAL_VERSION,
            Self::effective_seed(definition, request.seed),
        )
        .from_preset(Identifier::nexora(&format!(
            "preset/{}",
            definition.category().as_str()
        ))?);
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

    fn supports(&self, mode: &GenerationMode) -> bool {
        matches!(mode, GenerationMode::Generate)
    }

    fn generate(&self, request: &GenerationRequest) -> Result<GeneratedMaterial> {
        if !self.supports(&request.mode) {
            // Refused rather than quietly served as a plain generation: a
            // "repair" that silently regenerates everything is a repair that
            // changes a material nobody asked to change.
            return Err(unsupported("this generator does not serve that mode yet")
                .with_context("mode", request.mode.as_str())
                .with_context("supported", GenerationMode::Generate.as_str()));
        }

        let definition = request.definition.clone();
        let recipe = Recipe::for_category(definition.category())?;
        let seed = Self::effective_seed(&definition, request.seed);
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
        assert_eq!(trace.parameters["backend"], "procedural");
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

    #[test]
    fn a_mode_this_generator_does_not_serve_is_refused_not_quietly_reinterpreted() {
        let generator = ProceduralGenerator::new().unwrap();
        let mut request = GenerationRequest::generate(
            definition("nexora:material/oak", MaterialCategory::Wood, 16),
            1,
        );
        request.mode = GenerationMode::Repair {
            of: id("nexora:material/oak"),
        };

        let err = generator
            .generate(&request)
            .expect_err("repair is not implemented yet and must say so");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("repair"), "{err}");
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
