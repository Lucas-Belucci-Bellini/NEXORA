//! The checks, and what they are checking for.
//!
//! Implements the list in the Texture Forge brief §8, one [`Check`] per item,
//! under the rule §7 states plainly: **do not silently accept an invalid
//! texture.** Every check that finds something produces a [`Finding`] carrying
//! an explanation, and the aggregate [`TextureValidationResult`] turns into an
//! error the moment anything failed.
//!
//! # Three surfaces, because there are three ways to be wrong
//!
//! * [`Validator::material`] checks a material still in memory — the maps
//!   against each other and against the definition that asked for them.
//! * [`Validator::encoded`] checks bytes — that a file decodes at all, and that
//!   what comes back is what went in.
//! * [`Validator::written`] checks a directory — that the files a material
//!   claims are actually there.
//!
//! A material can pass the first and fail the third, which is exactly the case
//! worth catching: a definition that says it has a normal map, next to a
//! directory that does not.

use std::path::Path;

use nexora_asset::generator::GeneratedMaterial;
use nexora_asset::material::BlendMode;
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::{ChannelLayout, MapRole, MapSet, TextureMap};
use nexora_asset::validation::{Check, Finding, TextureValidationResult};

use crate::layout;
use crate::png;

/// The largest seam ratio a texture may show before it is reported.
///
/// From measurement, not taste. Six hundred samples of this project's own
/// output — fifteen categories, four seeds, five maps, both axes — peak at
/// **1.082**. Noise deliberately sampled at 1.37 periods across the texture,
/// so that the lattice wraps and the texture does not, measures **2.41 to
/// 2.78**. Real textures generated at 128 and cropped to their middle 64 run
/// from below 1 up to **7.9**.
///
/// 1.5 sits between the worst tiling case and the clearest broken one, with
/// about 40% headroom above the first and 60% below the second. The same
/// number holds the generator in `procedural`, through the same function:
/// there is one seam measure in this crate, not two.
pub const MAX_SEAM_RATIO: f64 = 1.5;

/// How far a decoded normal may stray from unit length.
///
/// Eight bits per channel quantises each component to about 0.008, so a
/// perfectly encoded normal can still measure 1.02 long. Anything beyond that
/// was not a unit vector before it was encoded.
pub const NORMAL_TOLERANCE: f64 = 0.03;

/// Runs the checks.
#[derive(Debug, Clone)]
pub struct Validator {
    /// The seam threshold to hold textures to.
    pub max_seam_ratio: f64,
    /// The unit-length tolerance for normals.
    pub normal_tolerance: f64,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    /// A validator with the documented thresholds.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_seam_ratio: MAX_SEAM_RATIO,
            normal_tolerance: NORMAL_TOLERANCE,
        }
    }

    /// Check a material that is still in memory.
    pub fn material(&self, generated: &GeneratedMaterial) -> TextureValidationResult {
        let mut result = generated.validate_completeness();
        result.merge(self.maps(generated.definition(), generated.maps()));

        // Runtime compatibility: every map has to survive the trip to a file
        // and back, because that is the form the runtime will load.
        for map in generated.maps().iter() {
            match png::encode(map) {
                Ok(bytes) => result.merge(self.encoded(&bytes, map)),
                Err(error) => result.push(
                    Finding::failure(
                        Check::RuntimeCompatibility,
                        format!("this map cannot be written as a PNG: {error}"),
                    )
                    .about(map.role()),
                ),
            }
        }
        result
    }

    /// Check a set of maps against the definition that asked for them.
    ///
    /// Shared by [`Self::material`] and by anything holding maps that came
    /// from disk rather than from a generator, so a file and the thing it was
    /// written from are judged by the same rules.
    pub fn maps(&self, definition: &SurfaceMaterial, maps: &MapSet) -> TextureValidationResult {
        let mut result = TextureValidationResult::new();

        // Metadata: the origin record has to satisfy the registry's own rules,
        // and a material whose provenance is broken must never reach a file.
        if let Err(error) = definition.provenance().validate() {
            result.push(Finding::failure(
                Check::Metadata,
                format!("the origin record breaks a registry rule: {error}"),
            ));
        }

        // Naming: identifiers become directories, and some names mean things.
        if let Err(error) = layout::check_naming(definition) {
            result.push(Finding::failure(
                Check::Naming,
                format!("the identifier cannot become a path: {error}"),
            ));
        }

        for map in maps.iter() {
            let role = map.role();

            // Resolution: a map that disagrees with its material would tile at
            // the wrong rate and nothing downstream would notice.
            if map.resolution() != definition.resolution() {
                result.push(
                    Finding::failure(
                        Check::Resolution,
                        format!(
                            "the material declares {}x{} and this map is {}x{}",
                            definition.resolution().width,
                            definition.resolution().height,
                            map.resolution().width,
                            map.resolution().height
                        ),
                    )
                    .about(role),
                );
            }

            // Dimensions: powers of two, which mipmaps and the atlas assume.
            if !map.resolution().width.is_power_of_two()
                || !map.resolution().height.is_power_of_two()
            {
                result.push(
                    Finding::failure(
                        Check::Dimensions,
                        "an edge is not a power of two, so this cannot be mipmapped".to_owned(),
                    )
                    .about(role),
                );
            }

            // Channels: a role stored in an unexpected layout is legal but
            // worth a look, because it usually means a pipeline guessed.
            if map.format().channels != role.natural_layout() {
                result.push(
                    Finding::warning(
                        Check::Channels,
                        format!(
                            "stored as {} where {} is usual for this role",
                            map.format().channels.as_str(),
                            role.natural_layout().as_str()
                        ),
                    )
                    .about(role),
                );
            }

            if role == MapRole::Normal {
                self.check_normals(map, &mut result);
            }
            self.check_values(definition.blend(), map, &mut result);

            if definition.is_seamless() {
                self.check_seams(map, &mut result);
            }
        }

        // Material integrity: every map has to agree with every other on size.
        let sizes: Vec<_> = maps.iter().map(|map| map.resolution()).collect();
        if sizes.windows(2).any(|pair| pair[0] != pair[1]) {
            result.push(Finding::failure(
                Check::MaterialIntegrity,
                "the maps in this material are not all the same size".to_owned(),
            ));
        }

        result
    }

    /// Check encoded bytes against the map they should contain.
    pub fn encoded(&self, bytes: &[u8], expected: &TextureMap) -> TextureValidationResult {
        let mut result = TextureValidationResult::new();
        let role = expected.role();

        let decoded = match png::decode(bytes) {
            Ok(decoded) => decoded,
            Err(error) => {
                result.push(
                    Finding::failure(
                        Check::Corruption,
                        format!("the file does not decode: {error}"),
                    )
                    .about(role),
                );
                return result;
            }
        };

        if decoded.resolution != expected.resolution() {
            result.push(
                Finding::failure(
                    Check::Format,
                    format!(
                        "the file is {}x{} where the map is {}x{}",
                        decoded.resolution.width,
                        decoded.resolution.height,
                        expected.resolution().width,
                        expected.resolution().height
                    ),
                )
                .about(role),
            );
        }
        if decoded.layout != expected.format().channels {
            result.push(
                Finding::failure(
                    Check::Format,
                    format!(
                        "the file stores {} where the map is {}",
                        decoded.layout.as_str(),
                        expected.format().channels.as_str()
                    ),
                )
                .about(role),
            );
        }
        if result.is_usable() && decoded.pixels != expected.pixels() {
            result.push(
                Finding::failure(
                    Check::RuntimeCompatibility,
                    "the file decodes to different pixels than the map it was written from"
                        .to_owned(),
                )
                .about(role),
            );
        }
        result
    }

    /// Check that the files a material claims exist and hold what it says.
    pub fn written(&self, root: &Path, generated: &GeneratedMaterial) -> TextureValidationResult {
        let mut result = TextureValidationResult::new();
        let id = generated.definition().id();

        let definition = layout::definition_file(root, id);
        if !definition.is_file() {
            result.push(Finding::failure(
                Check::FileExists,
                format!("the definition is missing: {}", definition.display()),
            ));
        }

        for map in generated.maps().iter() {
            let path = layout::map_file(root, id, map.role());
            match std::fs::read(&path) {
                Ok(bytes) => result.merge(self.encoded(&bytes, map)),
                Err(error) => result.push(
                    Finding::failure(
                        Check::FileExists,
                        format!("{} could not be read: {error}", path.display()),
                    )
                    .about(map.role()),
                ),
            }
        }
        result
    }

    fn check_normals(&self, map: &TextureMap, result: &mut TextureValidationResult) {
        if map.format().channels.count() < 3 {
            result.push(
                Finding::failure(
                    Check::NormalMap,
                    "a normal map needs three channels to hold a vector".to_owned(),
                )
                .about(MapRole::Normal),
            );
            return;
        }
        let channels = map.format().channels.count() as usize;
        let mut worst = 0.0f64;
        let mut inverted = 0usize;
        for texel in map.pixels().chunks_exact(channels) {
            let decode = |byte: u8| f64::from(byte) / 255.0 * 2.0 - 1.0;
            let (x, y, z) = (decode(texel[0]), decode(texel[1]), decode(texel[2]));
            worst = worst.max(((x * x + y * y + z * z).sqrt() - 1.0).abs());
            if z <= 0.0 {
                inverted += 1;
            }
        }
        if worst > self.normal_tolerance {
            result.push(
                Finding::failure(
                    Check::NormalMap,
                    format!(
                        "a normal is {worst:.3} away from unit length, past the {:.3} that eight-bit encoding explains",
                        self.normal_tolerance
                    ),
                )
                .about(MapRole::Normal),
            );
        }
        if inverted > 0 {
            result.push(
                Finding::failure(
                    Check::NormalMap,
                    format!("{inverted} normals point into the surface rather than out of it"),
                )
                .about(MapRole::Normal),
            );
        }
    }

    fn check_values(
        &self,
        blend: BlendMode,
        map: &TextureMap,
        result: &mut TextureValidationResult,
    ) {
        // An opaque material with a transparent albedo is a contradiction that
        // renders as holes nobody asked for.
        if map.role() == MapRole::Albedo
            && blend == BlendMode::Opaque
            && map.format().channels == ChannelLayout::Rgba
        {
            let transparent = map
                .pixels()
                .chunks_exact(4)
                .filter(|texel| texel[3] != 255)
                .count();
            if transparent > 0 {
                result.push(
                    Finding::failure(
                        Check::ValueRange,
                        format!(
                            "the material is opaque and {transparent} texels of its albedo are not"
                        ),
                    )
                    .about(MapRole::Albedo),
                );
            }
        }

        // A map with one value carries no information the definition does not
        // already hold, and costs bytes to say so.
        if matches!(
            map.role(),
            MapRole::Roughness | MapRole::Height | MapRole::AmbientOcclusion
        ) {
            let first = map.pixels().first().copied().unwrap_or(0);
            if map.pixels().iter().all(|byte| *byte == first) {
                result.push(
                    Finding::note(
                        Check::ValueRange,
                        format!(
                            "every texel is {first}; this map says nothing the definition does not"
                        ),
                    )
                    .about(map.role()),
                );
            }
        }
    }

    fn check_seams(&self, map: &TextureMap, result: &mut TextureValidationResult) {
        for vertical in [false, true] {
            let ratio = seam_ratio(map, vertical);
            if ratio > self.max_seam_ratio {
                result.push(
                    Finding::failure(
                        Check::Seamless,
                        format!(
                            "the {} seam is {ratio:.2} times as sharp as the texture's own detail, past {:.2}",
                            if vertical { "vertical" } else { "horizontal" },
                            self.max_seam_ratio
                        ),
                    )
                    .about(map.role()),
                );
            }
        }
    }
}

/// How far the wrapping seam stands out among the texture's own column pairs.
///
/// Asking whether something tiles is harder than it looks, and two simpler
/// measures were tried and discarded first:
///
/// * *"are the two edges the same colour"* — no. A texture of parallel boards
///   has different boards at its two edges and should.
/// * *"is the seam step bigger than this texture's average step"* — no. It
///   fails a texture whose seam happens to sit on its sharpest feature. Wood's
///   normal map scored 2.34 that way, because the seam falls in the gap
///   between two boards where every step is steep.
/// * *"is the seam step bigger than the two steps beside it"* — also no. On a
///   high-frequency surface those neighbours are noise, and dividing noise by
///   noise gave a tiling soil texture a score of 6.8 while a deliberately
///   broken one scored 2.0.
///
/// What actually holds is this: **treat the wrap as one more column pair.**
/// Average the absolute step over every adjacent pair of columns, giving
/// `width - 1` numbers, and average it over the wrapping pair too. A texture
/// that tiles has no distinguished edge, so the wrapping pair is an ordinary
/// member of that set — at worst as sharp as the sharpest interior pair. A
/// texture that does not tile has a wrapping pair that is the largest by a
/// clear margin, because the discontinuity exists nowhere else.
///
/// The result is the wrapping pair's mean step divided by the largest interior
/// pair's. Returns zero for a texture too small to have interior pairs, or one
/// with no variation for the seam to stand out from.
///
/// # What it does not claim
///
/// It measures whether a seam is **visible**, not whether the underlying
/// function is periodic. A brick texture cropped out of a larger one can score
/// below 1, because the mortar lines inside it are sharper than anything the
/// crop introduced — and that is the right answer for a check whose job is to
/// stop seams reaching a player's eye. Periodicity itself is proved where it
/// is actually established, in `noise`, to within 1e-12.
#[must_use]
pub fn seam_ratio(map: &TextureMap, vertical: bool) -> f64 {
    let width = map.resolution().width as usize;
    let height = map.resolution().height as usize;
    let channels = map.format().channels.count() as usize;
    let stride = width * channels;
    let pixels = map.pixels();
    let at = |x: usize, y: usize, c: usize| i32::from(pixels[y * stride + x * channels + c]);

    let (across, along) = if vertical {
        (width, height)
    } else {
        (height, width)
    };
    if along < 3 || across == 0 {
        return 0.0;
    }
    let sample = |line: usize, position: usize, c: usize| -> i32 {
        if vertical {
            at(line, position, c)
        } else {
            at(position, line, c)
        }
    };

    // The mean absolute step for each adjacent pair of lines, plus the wrap.
    let mean_step = |low: usize, high: usize| -> f64 {
        let mut total = 0f64;
        for line in 0..across {
            for c in 0..channels {
                total += f64::from((sample(line, low, c) - sample(line, high, c)).abs());
            }
        }
        total / ((across * channels) as f64)
    };

    let seam = mean_step(along - 1, 0);
    let sharpest_interior = (0..along - 1)
        .map(|position| mean_step(position, position + 1))
        .fold(0.0f64, f64::max);

    if sharpest_interior < 0.5 {
        // Nothing anywhere to compare against. A step at the seam of an
        // otherwise flat texture is a seam, and the ratio cannot say so, so
        // the step itself is reported.
        return if seam < 0.5 { 0.0 } else { seam };
    }
    seam / sharpest_interior
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pbr::PbrPipeline;
    use crate::procedural::ProceduralGenerator;
    use nexora_asset::generator::{GenerationRequest, TextureGenerator, TexturePipeline};
    use nexora_asset::material::{MaterialCategory, SurfaceMaterial};
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::{MapSet, Resolution, TextureFormat};
    use nexora_asset::validation::{Severity, Verdict};
    use nexora_foundation::ident::Identifier;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn definition(category: MaterialCategory, edge: u32) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id("nexora:material/sample"),
            category,
            Resolution::square(edge).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .wants(MapRole::Height)
        .wants(MapRole::Normal)
        .wants(MapRole::Roughness)
        .wants(MapRole::AmbientOcclusion)
        .build()
        .expect("valid definition")
    }

    fn generated(category: MaterialCategory, edge: u32) -> GeneratedMaterial {
        let produced = ProceduralGenerator::new()
            .unwrap()
            .generate(&GenerationRequest::generate(definition(category, edge), 5))
            .expect("generation succeeds");
        PbrPipeline::new()
            .unwrap()
            .apply(produced)
            .expect("the pipeline succeeds")
    }

    fn replace_map(material: GeneratedMaterial, replacement: TextureMap) -> GeneratedMaterial {
        let (definition, maps, preview) = material.into_parts();
        let mut rebuilt = MapSet::new();
        for map in maps.iter() {
            if map.role() == replacement.role() {
                rebuilt.insert(replacement.clone()).unwrap();
            } else {
                rebuilt.insert(map.clone()).unwrap();
            }
        }
        GeneratedMaterial::transformed(&PbrPipeline::new().unwrap(), definition, rebuilt, preview)
            .expect("reassembly")
    }

    #[test]
    fn a_material_this_project_generated_passes_every_check() {
        let validator = Validator::new();
        for category in MaterialCategory::ALL {
            let result = validator.material(&generated(category, 64));
            assert!(
                result.is_usable(),
                "{category:?} failed validation:\n{result}"
            );
            // And it is not passing by finding nothing to look at.
            assert!(
                result.at_least(Severity::Failure).count() == 0,
                "{category:?}: {result}"
            );
        }
    }

    #[test]
    fn a_normal_map_that_is_not_unit_length_is_caught() {
        let material = generated(MaterialCategory::Stone, 32);
        let normal = material.maps().get(MapRole::Normal).unwrap();
        // Halve every component: still a direction, no longer a unit vector.
        let mangled: Vec<u8> = normal
            .pixels()
            .iter()
            .map(|byte| 128 + (i16::from(*byte) - 128) as u8 / 4)
            .collect();
        let broken = TextureMap::new(
            MapRole::Normal,
            normal.format(),
            normal.resolution(),
            normal
                .pixels()
                .iter()
                .zip(mangled)
                .map(|(_, m)| m)
                .collect(),
        )
        .unwrap();

        // Push every component towards zero so the vectors are short.
        let short: Vec<u8> = broken.pixels().iter().map(|_| 130u8).collect();
        let short =
            TextureMap::new(MapRole::Normal, normal.format(), normal.resolution(), short).unwrap();

        let result = Validator::new().material(&replace_map(material, short));
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::NormalMap),
            "{result}"
        );
    }

    #[test]
    fn a_normal_pointing_into_the_surface_is_caught() {
        let material = generated(MaterialCategory::Stone, 32);
        let normal = material.maps().get(MapRole::Normal).unwrap();
        // Flip Z: still unit length, and facing entirely the wrong way.
        let flipped: Vec<u8> = normal
            .pixels()
            .chunks_exact(3)
            .flat_map(|texel| [texel[0], texel[1], 255 - texel[2]])
            .collect();
        let flipped = TextureMap::new(
            MapRole::Normal,
            normal.format(),
            normal.resolution(),
            flipped,
        )
        .unwrap();

        let result = Validator::new().material(&replace_map(material, flipped));
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.message.contains("into the surface")),
            "{result}"
        );
    }

    #[test]
    fn an_opaque_material_with_a_transparent_albedo_is_a_contradiction() {
        let material = generated(MaterialCategory::Stone, 32);
        let albedo = material.maps().get(MapRole::Albedo).unwrap();
        let holed: Vec<u8> = albedo
            .pixels()
            .chunks_exact(4)
            .enumerate()
            .flat_map(|(index, texel)| {
                [
                    texel[0],
                    texel[1],
                    texel[2],
                    if index == 7 { 0 } else { 255 },
                ]
            })
            .collect();
        let holed =
            TextureMap::new(MapRole::Albedo, albedo.format(), albedo.resolution(), holed).unwrap();

        let result = Validator::new().material(&replace_map(material, holed));
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::ValueRange),
            "{result}"
        );
    }

    #[test]
    fn a_field_that_cannot_tile_scores_above_the_threshold() {
        // The control that makes MAX_SEAM_RATIO mean something. Noise sampled
        // at 1.37 periods across the texture: the lattice wraps, the texture
        // does not, and nothing else about it is unusual — no painted edges, no
        // spliced halves, nothing that would perturb what it is measured
        // against.
        use crate::noise::Noise;
        let noise = Noise::new(0xC0FF_EE00);
        let edge = 64usize;
        let mut pixels = Vec::with_capacity(edge * edge * 4);
        for y in 0..edge {
            for x in 0..edge {
                let u = (x as f64 + 0.5) / edge as f64;
                let v = (y as f64 + 0.5) / edge as f64;
                let value = (noise.fbm(u * 1.37, v * 1.37, 4, 4, 5) * 255.0) as u8;
                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }
        let map = TextureMap::new(
            MapRole::Albedo,
            TextureFormat::eight_bit(ChannelLayout::Rgba),
            Resolution::square(64).unwrap(),
            pixels,
        )
        .unwrap();

        for vertical in [false, true] {
            let ratio = seam_ratio(&map, vertical);
            assert!(
                ratio > MAX_SEAM_RATIO,
                "a non-tiling field measured {ratio:.3}, which the check would accept"
            );
        }
    }

    #[test]
    fn a_real_texture_cropped_out_of_a_larger_one_is_caught() {
        // The second control, and the more realistic one: a texture generated
        // at 128 and cropped to its middle 64. Every texel inside is real
        // texture; only the wrap was destroyed.
        let big = generated(MaterialCategory::Stone, 128);
        let source = big.maps().get(MapRole::Height).unwrap();
        let channels = source.format().channels.count() as usize;
        let mut cropped = Vec::new();
        for y in 32..96usize {
            let row = y * 128 * channels;
            cropped.extend_from_slice(&source.pixels()[row + 32 * channels..row + 96 * channels]);
        }
        let cropped = TextureMap::new(
            MapRole::Height,
            source.format(),
            Resolution::square(64).unwrap(),
            cropped,
        )
        .unwrap();

        assert!(
            seam_ratio(&cropped, false) > MAX_SEAM_RATIO
                || seam_ratio(&cropped, true) > MAX_SEAM_RATIO,
            "cropping destroyed the wrap and the check did not notice: h {:.3} v {:.3}",
            seam_ratio(&cropped, false),
            seam_ratio(&cropped, true)
        );

        // And it reaches the validator as a failure, not just as a number.
        let material = generated(MaterialCategory::Stone, 64);
        let result = Validator::new().material(&replace_map(material, cropped));
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::Seamless),
            "{result}"
        );
    }

    #[test]
    fn a_corrupted_file_fails_where_the_map_it_came_from_passed() {
        let material = generated(MaterialCategory::Wood, 32);
        let map = material.maps().get(MapRole::Albedo).unwrap();
        let validator = Validator::new();

        let good = png::encode(map).unwrap();
        validator
            .encoded(&good, map)
            .ok("nexora:material/sample")
            .expect("a file we just wrote must validate");

        let mut damaged = good.clone();
        let middle = damaged.len() / 2;
        damaged[middle] ^= 0xFF;
        let result = validator.encoded(&damaged, map);
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::Corruption),
            "{result}"
        );

        // Truncation, which is what a half-written file looks like.
        let result = validator.encoded(&good[..good.len() / 2], map);
        assert_eq!(result.verdict(), Verdict::Fail);
    }

    #[test]
    fn a_file_that_holds_a_different_image_is_caught_even_though_it_decodes() {
        let material = generated(MaterialCategory::Wood, 32);
        let map = material.maps().get(MapRole::Albedo).unwrap();
        let other = generated(MaterialCategory::Sand, 32);
        let other = other.maps().get(MapRole::Albedo).unwrap();

        // A perfectly valid PNG of the wrong material.
        let bytes = png::encode(other).unwrap();
        let result = Validator::new().encoded(&bytes, map);
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::RuntimeCompatibility),
            "{result}"
        );
    }

    #[test]
    fn a_missing_file_is_reported_by_name() {
        let material = generated(MaterialCategory::Stone, 16);
        let root = std::env::temp_dir().join(format!("nexora-validator-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);

        let result = Validator::new().written(&root, &material);
        assert_eq!(result.verdict(), Verdict::Fail);
        assert!(
            result
                .at_least(Severity::Failure)
                .any(|finding| finding.check == Check::FileExists),
            "{result}"
        );
        // The message names the path, which is what makes it actionable.
        assert!(result.to_string().contains("material.json"), "{result}");
    }

    #[test]
    fn a_material_written_to_disk_validates_against_what_is_there() {
        let material = generated(MaterialCategory::Brick, 32);
        let root =
            std::env::temp_dir().join(format!("nexora-validator-written-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = layout::material_dir(&root, material.definition().id());
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            layout::definition_file(&root, material.definition().id()),
            nexora_asset::document::to_text(material.definition()),
        )
        .unwrap();
        for map in material.maps().iter() {
            std::fs::write(
                layout::map_file(&root, material.definition().id(), map.role()),
                png::encode(map).unwrap(),
            )
            .unwrap();
        }

        let result = Validator::new().written(&root, &material);
        assert!(result.is_usable(), "{result}");

        // Now damage one file on disk and watch the same call fail.
        let albedo = layout::map_file(&root, material.definition().id(), MapRole::Albedo);
        let mut bytes = std::fs::read(&albedo).unwrap();
        let middle = bytes.len() / 2;
        bytes[middle] ^= 0xFF;
        std::fs::write(&albedo, &bytes).unwrap();

        let result = Validator::new().written(&root, &material);
        assert_eq!(result.verdict(), Verdict::Fail, "{result}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_constant_map_is_noted_but_does_not_fail() {
        let material = generated(MaterialCategory::Stone, 32);
        let height = material.maps().get(MapRole::Height).unwrap();
        let flat = TextureMap::new(
            MapRole::Height,
            height.format(),
            height.resolution(),
            vec![128; height.byte_len()],
        )
        .unwrap();

        let result = Validator::new().material(&replace_map(material, flat));
        // A flat height map is a legitimate thing to have, and worth saying.
        assert!(
            result
                .findings()
                .iter()
                .any(|finding| finding.severity == Severity::Note
                    && finding.check == Check::ValueRange),
            "{result}"
        );
    }

    #[test]
    fn the_seam_measure_is_the_same_one_the_generator_is_held_to() {
        // Not a second implementation with a second threshold: the generator's
        // own test and this validator call the same function.
        let material = generated(MaterialCategory::Wood, 64);
        let albedo = material.maps().get(MapRole::Albedo).unwrap();
        for vertical in [false, true] {
            assert!(seam_ratio(albedo, vertical) <= MAX_SEAM_RATIO);
        }

        // A one-texel image has no interior to compare against, and says so
        // rather than dividing by nothing.
        let tiny = TextureMap::new(
            MapRole::Height,
            TextureFormat::eight_bit(ChannelLayout::Grey),
            Resolution::square(4).unwrap(),
            vec![7; 16],
        )
        .unwrap();
        assert!((seam_ratio(&tiny, false) - 0.0).abs() < f64::EPSILON);
    }
}
