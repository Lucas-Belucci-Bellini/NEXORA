//! The PBR pipeline: the maps that are *derived* rather than drawn.
//!
//! A generator draws colour and the relief it meant to draw. Normals,
//! roughness, metallic and ambient occlusion all follow from those two by
//! arithmetic — which is why they belong to a pipeline. Putting them in the
//! generator would mean every future backend, including an image model, had to
//! reimplement them identically or the material set would drift.
//!
//! ```text
//! albedo + height ──► normal      Sobel gradient of the height field
//!                 ──► roughness   the declared value, varied by local relief
//!                 ──► metallic    the declared value
//!                 ──► ambient occlusion   height against its neighbourhood
//! ```
//!
//! # Every derivation wraps
//!
//! The generator's output tiles, and a derived map that sampled its edges
//! without wrapping would not — the normal map would carry a bright line down
//! two sides of every texture. So every neighbourhood read here is
//! `rem_euclid`, and a test checks the derived maps for the same seam property
//! the source has.
//!
//! # What the normal map's slope means
//!
//! Nothing physical, yet. A material declares `metres_per_tile` but not how
//! deep its relief is in metres, so there is no honest way to turn a height in
//! `0.0..=1.0` into a real gradient. `normal_strength` is therefore a
//! stylistic control, and that gap is recorded rather than papered over with a
//! constant that would look like physics. See DEBT-0030.

use nexora_asset::generator::{GeneratedMaterial, TexturePipeline};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::{ChannelLayout, MapRole, Resolution, TextureFormat, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::ContentPipelineVersion;

/// The PBR pipeline's stable identifier.
pub const PBR_PIPELINE: &str = "nexora:pipeline/pbr";

/// The PBR pipeline's version.
///
/// Changing any derivation changes the maps, and provenance claims those maps
/// can be rebuilt. The claim and the number move together.
pub const PBR_VERSION: ContentPipelineVersion = ContentPipelineVersion(1);

/// How far roughness may stray from the material's declared value.
///
/// The derivation is mean-preserving: a texel with more local relief than the
/// texture's average is rougher, one with less is smoother, and the average
/// over the whole map comes back to what the material declared. This is the
/// amplitude of that variation, not an offset.
pub const DEFAULT_ROUGHNESS_DETAIL: f64 = 0.30;

/// How strongly a dip below its surroundings darkens ambient occlusion.
pub const DEFAULT_OCCLUSION_STRENGTH: f64 = 1.6;

/// How far the occlusion comparison looks, in texels.
pub const DEFAULT_OCCLUSION_RADIUS: u32 = 2;

/// Derives the maps that follow from albedo and height.
#[derive(Debug, Clone)]
pub struct PbrPipeline {
    id: Identifier,
    /// Amplitude of the roughness variation around the declared value.
    pub roughness_detail: f64,
    /// How strongly a dip darkens.
    pub occlusion_strength: f64,
    /// How far the occlusion comparison looks, in texels.
    pub occlusion_radius: u32,
}

impl PbrPipeline {
    /// Build the pipeline with the documented defaults.
    ///
    /// # Errors
    ///
    /// Returns an error only if [`PBR_PIPELINE`] stops being a valid
    /// identifier, which would be a build-time mistake.
    pub fn new() -> Result<Self> {
        Ok(Self {
            id: Identifier::parse(PBR_PIPELINE)?,
            roughness_detail: DEFAULT_ROUGHNESS_DETAIL,
            occlusion_strength: DEFAULT_OCCLUSION_STRENGTH,
            occlusion_radius: DEFAULT_OCCLUSION_RADIUS,
        })
    }

    /// Which roles this pipeline can produce for a material.
    ///
    /// Only what the material asked for. A pipeline that emitted every map it
    /// could would quadruple the byte cost of a stone that wanted colour.
    fn derivable(definition: &SurfaceMaterial) -> Vec<MapRole> {
        [
            MapRole::Normal,
            MapRole::Roughness,
            MapRole::Metallic,
            MapRole::AmbientOcclusion,
        ]
        .into_iter()
        .filter(|role| definition.wanted_maps().contains(role))
        .collect()
    }
}

impl TexturePipeline for PbrPipeline {
    fn id(&self) -> &Identifier {
        &self.id
    }

    fn version(&self) -> ContentPipelineVersion {
        PBR_VERSION
    }

    fn apply(&self, material: GeneratedMaterial) -> Result<GeneratedMaterial> {
        let (definition, mut maps, preview) = material.into_parts();
        let wanted = Self::derivable(&definition);

        // Normal and occlusion both read relief. Asking for them without a
        // height field is a request that cannot be served, and saying so here
        // is more useful than emitting a flat normal map that looks correct.
        let needs_height = wanted
            .iter()
            .any(|role| matches!(role, MapRole::Normal | MapRole::AmbientOcclusion));
        let heights = match maps.get(MapRole::Height) {
            Some(map) => Some(HeightField::from_map(map)?),
            None if needs_height => {
                return Err(missing(
                    "this material asks for a map derived from height, and has no height map",
                )
                .with_context("material", definition.id().to_string())
                .with_context(
                    "wanted",
                    wanted
                        .iter()
                        .map(|role| role.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ))
            }
            None => None,
        };

        let resolution = definition.resolution();
        for role in wanted {
            if maps.contains(role) {
                continue;
            }
            let map = match role {
                MapRole::Normal => {
                    let field = heights.as_ref().expect("checked above");
                    normal_map(field, definition.pbr().normal_strength, resolution)?
                }
                MapRole::Roughness => roughness_map(
                    heights.as_ref(),
                    &definition,
                    self.roughness_detail,
                    resolution,
                )?,
                MapRole::Metallic => {
                    constant_map(MapRole::Metallic, definition.pbr().metallic, resolution)?
                }
                MapRole::AmbientOcclusion => {
                    let field = heights.as_ref().expect("checked above");
                    occlusion_map(
                        field,
                        self.occlusion_strength,
                        self.occlusion_radius,
                        resolution,
                    )?
                }
                // `derivable` returns only those four.
                other => {
                    return Err(missing("this pipeline does not derive that map")
                        .with_context("map", other.as_str()))
                }
            };
            maps.insert(map)?;
        }

        let recorded = record_pipeline(definition, self)?;
        GeneratedMaterial::transformed(self, recorded, maps, preview)
    }
}

/// Stamp the pipeline into the material's origin record.
fn record_pipeline(definition: SurfaceMaterial, pipeline: &PbrPipeline) -> Result<SurfaceMaterial> {
    let mut provenance = definition.provenance().clone();
    let Some(trace) = provenance.generation.as_mut() else {
        return Err(
            missing("a generated material must carry a trace before a pipeline runs")
                .with_context("material", definition.id().to_string()),
        );
    };
    trace.pipeline = Some(pipeline.id().clone());
    trace.pipeline_version = Some(pipeline.version());
    trace.parameters.insert(
        "roughness_detail".to_owned(),
        format!("{:.4}", pipeline.roughness_detail),
    );
    trace.parameters.insert(
        "occlusion_strength".to_owned(),
        format!("{:.4}", pipeline.occlusion_strength),
    );
    trace.parameters.insert(
        "occlusion_radius".to_owned(),
        pipeline.occlusion_radius.to_string(),
    );
    // A revision, not an edit: the material that went in still exists.
    definition.revised(provenance)
}

/// A height map read back as numbers, with wrapping neighbourhood access.
struct HeightField {
    width: i64,
    height: i64,
    values: Vec<f64>,
}

impl HeightField {
    fn from_map(map: &TextureMap) -> Result<Self> {
        if map.format().channels != ChannelLayout::Grey || map.format().bits_per_channel != 8 {
            return Err(missing("a height map must be single-channel eight-bit")
                .with_context("channels", map.format().channels.as_str()));
        }
        Ok(Self {
            width: i64::from(map.resolution().width),
            height: i64::from(map.resolution().height),
            values: map
                .pixels()
                .iter()
                .map(|byte| f64::from(*byte) / 255.0)
                .collect(),
        })
    }

    /// Sample with wrapping, which is what keeps every derived map seamless.
    fn at(&self, x: i64, y: i64) -> f64 {
        let x = x.rem_euclid(self.width) as usize;
        let y = y.rem_euclid(self.height) as usize;
        self.values[y * self.width as usize + x]
    }

    /// The Sobel gradient at one texel, in height units per texel.
    fn gradient(&self, x: i64, y: i64) -> (f64, f64) {
        let dx = (self.at(x + 1, y - 1) + 2.0 * self.at(x + 1, y) + self.at(x + 1, y + 1))
            - (self.at(x - 1, y - 1) + 2.0 * self.at(x - 1, y) + self.at(x - 1, y + 1));
        let dy = (self.at(x - 1, y + 1) + 2.0 * self.at(x, y + 1) + self.at(x + 1, y + 1))
            - (self.at(x - 1, y - 1) + 2.0 * self.at(x, y - 1) + self.at(x + 1, y - 1));
        (dx / 8.0, dy / 8.0)
    }

    /// Mean height over a square neighbourhood, wrapping.
    fn neighbourhood(&self, x: i64, y: i64, radius: i64) -> f64 {
        let mut total = 0.0;
        let mut count = 0.0;
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                total += self.at(x + ox, y + oy);
                count += 1.0;
            }
        }
        total / count
    }
}

fn normal_map(field: &HeightField, strength: f64, resolution: Resolution) -> Result<TextureMap> {
    let mut pixels = Vec::with_capacity(resolution.texels() as usize * 3);
    for y in 0..i64::from(resolution.height) {
        for x in 0..i64::from(resolution.width) {
            let (dx, dy) = field.gradient(x, y);
            // Tangent space, +Z out of the surface. The gradients are negated
            // because a surface that rises to the right tilts its normal left.
            let (nx, ny, nz) = normalise(-dx * strength, -dy * strength, 1.0);
            pixels.push(encode_signed(nx));
            pixels.push(encode_signed(ny));
            pixels.push(encode_signed(nz));
        }
    }
    TextureMap::new(
        MapRole::Normal,
        TextureFormat::eight_bit(ChannelLayout::Rgb),
        resolution,
        pixels,
    )
}

fn roughness_map(
    field: Option<&HeightField>,
    definition: &SurfaceMaterial,
    detail: f64,
    resolution: Resolution,
) -> Result<TextureMap> {
    let base = definition.pbr().roughness;
    let texels = resolution.texels() as usize;

    // No relief to vary with: the declared value, everywhere. Honest, and the
    // material said so.
    let Some(field) = field else {
        return constant_map(MapRole::Roughness, base, resolution);
    };

    let mut slopes = Vec::with_capacity(texels);
    for y in 0..i64::from(resolution.height) {
        for x in 0..i64::from(resolution.width) {
            let (dx, dy) = field.gradient(x, y);
            slopes.push(dx.hypot(dy));
        }
    }
    let peak = slopes.iter().copied().fold(0.0f64, f64::max);
    let pixels = if peak <= f64::EPSILON {
        // A perfectly flat height field carries no detail to vary with.
        vec![encode_unit(base); texels]
    } else {
        let normalised: Vec<f64> = slopes.iter().map(|slope| slope / peak).collect();
        let mean = normalised.iter().sum::<f64>() / texels as f64;
        normalised
            .iter()
            .map(|value| encode_unit(base + (value - mean) * detail))
            .collect()
    };

    TextureMap::new(
        MapRole::Roughness,
        TextureFormat::eight_bit(ChannelLayout::Grey),
        resolution,
        pixels,
    )
}

fn occlusion_map(
    field: &HeightField,
    strength: f64,
    radius: u32,
    resolution: Resolution,
) -> Result<TextureMap> {
    let radius = i64::from(radius.max(1));
    let mut pixels = Vec::with_capacity(resolution.texels() as usize);
    for y in 0..i64::from(resolution.height) {
        for x in 0..i64::from(resolution.width) {
            let here = field.at(x, y);
            let around = field.neighbourhood(x, y, radius);
            // Only dips occlude. A texel standing proud of its neighbours is
            // not shadowed by them, so the term is one-sided.
            let dip = (around - here).max(0.0);
            pixels.push(encode_unit(1.0 - dip * strength));
        }
    }
    TextureMap::new(
        MapRole::AmbientOcclusion,
        TextureFormat::eight_bit(ChannelLayout::Grey),
        resolution,
        pixels,
    )
}

fn constant_map(role: MapRole, value: f64, resolution: Resolution) -> Result<TextureMap> {
    TextureMap::new(
        role,
        TextureFormat::eight_bit(ChannelLayout::Grey),
        resolution,
        vec![encode_unit(value); resolution.texels() as usize],
    )
}

fn normalise(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let length = (x * x + y * y + z * z).sqrt();
    if length <= f64::EPSILON {
        return (0.0, 0.0, 1.0);
    }
    (x / length, y / length, z / length)
}

/// Encode `-1.0..=1.0` into a byte, the way a normal map stores a component.
fn encode_signed(value: f64) -> u8 {
    encode_unit(value.mul_add(0.5, 0.5))
}

/// Encode `0.0..=1.0` into a byte.
fn encode_unit(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn missing(message: &'static str) -> Error {
    Error::new(Domain::Content, "pbr-pipeline", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::procedural::ProceduralGenerator;
    use nexora_asset::generator::{GenerationRequest, TextureGenerator};
    use nexora_asset::material::{MaterialCategory, PbrParameters};
    use nexora_asset::provenance::Provenance;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn definition(
        category: MaterialCategory,
        roles: &[MapRole],
        pbr: PbrParameters,
    ) -> SurfaceMaterial {
        let mut builder = SurfaceMaterial::builder(
            id("nexora:material/sample"),
            category,
            Resolution::square(32).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .pbr(pbr);
        for role in roles {
            builder = builder.wants(*role);
        }
        builder.build().expect("valid definition")
    }

    fn run(definition: SurfaceMaterial) -> GeneratedMaterial {
        let generated = ProceduralGenerator::new()
            .unwrap()
            .generate(&GenerationRequest::generate(definition, 11))
            .expect("generation succeeds");
        PbrPipeline::new()
            .unwrap()
            .apply(generated)
            .expect("the pipeline succeeds")
    }

    /// A height field built from a closure, for testing derivations exactly.
    fn field(edge: u32, f: impl Fn(u32, u32) -> f64) -> HeightField {
        let resolution = Resolution::square(edge).unwrap();
        let pixels: Vec<u8> = (0..edge)
            .flat_map(|y| (0..edge).map(move |x| (x, y)))
            .map(|(x, y)| (f(x, y).clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect();
        let map = TextureMap::new(
            MapRole::Height,
            TextureFormat::eight_bit(ChannelLayout::Grey),
            resolution,
            pixels,
        )
        .unwrap();
        HeightField::from_map(&map).unwrap()
    }

    #[test]
    fn a_flat_surface_produces_a_flat_normal() {
        let flat = field(16, |_, _| 0.5);
        let map = normal_map(&flat, 1.0, Resolution::square(16).unwrap()).unwrap();
        // (0, 0, 1) encodes as (128, 128, 255).
        for normal in map.pixels().chunks_exact(3) {
            assert_eq!(
                normal,
                [128, 128, 255],
                "a flat height must give a flat normal"
            );
        }
    }

    #[test]
    fn a_slope_tilts_the_normal_the_way_the_surface_rises() {
        // Height rising towards +x. The normal must tilt towards -x, so its
        // red channel drops below the neutral 128.
        let ramp = field(16, |x, _| f64::from(x) / 16.0);
        let map = normal_map(&ramp, 1.0, Resolution::square(16).unwrap()).unwrap();
        // Away from the wrap, where the ramp actually rises.
        let at = |x: usize, y: usize| {
            let index = (y * 16 + x) * 3;
            &map.pixels()[index..index + 3]
        };
        assert!(
            at(8, 8)[0] < 128,
            "normal.x should tilt negative: {:?}",
            at(8, 8)
        );
        assert_eq!(at(8, 8)[1], 128, "a ramp in x must not tilt y");
        assert!(at(8, 8)[2] > 128, "normal.z stays positive");

        // The same ramp in y tilts the green channel instead.
        let ramp = field(16, |_, y| f64::from(y) / 16.0);
        let map = normal_map(&ramp, 1.0, Resolution::square(16).unwrap()).unwrap();
        let index = (8 * 16 + 8) * 3;
        assert_eq!(map.pixels()[index], 128);
        assert!(map.pixels()[index + 1] < 128);
    }

    #[test]
    fn zero_strength_flattens_the_normal_completely() {
        let ramp = field(16, |x, y| f64::from(x + y) / 32.0);
        let map = normal_map(&ramp, 0.0, Resolution::square(16).unwrap()).unwrap();
        for normal in map.pixels().chunks_exact(3) {
            assert_eq!(normal, [128, 128, 255]);
        }
    }

    #[test]
    fn every_normal_is_a_unit_vector() {
        let bumpy = field(32, |x, y| {
            (f64::from(x) * 0.7).sin().mul_add(0.25, 0.5) + (f64::from(y) * 0.3).cos() * 0.2
        });
        let map = normal_map(&bumpy, 3.0, Resolution::square(32).unwrap()).unwrap();
        for normal in map.pixels().chunks_exact(3) {
            let decode = |byte: u8| f64::from(byte) / 255.0 * 2.0 - 1.0;
            let (x, y, z) = (decode(normal[0]), decode(normal[1]), decode(normal[2]));
            let length = (x * x + y * y + z * z).sqrt();
            // Within the quantisation error of eight bits per channel.
            assert!(
                (length - 1.0).abs() < 0.02,
                "length {length} from {normal:?}"
            );
            assert!(z > 0.0, "a tangent-space normal points out of the surface");
        }
    }

    #[test]
    fn roughness_averages_to_what_the_material_declared() {
        // The invariant that makes the derivation honest: local relief moves a
        // texel above or below the declared roughness, and the map as a whole
        // still says what the material says.
        for base in [0.35, 0.5, 0.8] {
            let generated = run(definition(
                MaterialCategory::Stone,
                &[MapRole::Height, MapRole::Roughness],
                PbrParameters {
                    roughness: base,
                    ..PbrParameters::DEFAULT
                },
            ));
            let map = generated.maps().get(MapRole::Roughness).unwrap();
            let mean = map.pixels().iter().map(|b| f64::from(*b)).sum::<f64>()
                / map.pixels().len() as f64
                / 255.0;
            assert!(
                (mean - base).abs() < 0.02,
                "declared {base}, map averages {mean}"
            );
            // And it is not a constant: relief has to show.
            let distinct: std::collections::BTreeSet<u8> = map.pixels().iter().copied().collect();
            assert!(
                distinct.len() > 4,
                "roughness has {} levels",
                distinct.len()
            );
        }
    }

    #[test]
    fn a_dip_is_occluded_and_a_ridge_is_not() {
        // A single well in a flat plain.
        let well = field(16, |x, y| if x == 8 && y == 8 { 0.0 } else { 1.0 });
        let map = occlusion_map(&well, 1.6, 2, Resolution::square(16).unwrap()).unwrap();
        let at = |x: usize, y: usize| map.pixels()[y * 16 + x];
        assert!(
            at(8, 8) < 200,
            "the well must be occluded, got {}",
            at(8, 8)
        );
        assert_eq!(at(2, 2), 255, "the flat plain is unoccluded");

        // A single peak. Standing proud of the neighbourhood is not occlusion.
        let peak = field(16, |x, y| if x == 8 && y == 8 { 1.0 } else { 0.0 });
        let map = occlusion_map(&peak, 1.6, 2, Resolution::square(16).unwrap()).unwrap();
        assert_eq!(map.pixels()[8 * 16 + 8], 255, "a ridge is not occluded");
    }

    #[test]
    fn metallic_is_the_declared_value_everywhere() {
        let generated = run(definition(
            MaterialCategory::Metal,
            &[MapRole::Metallic],
            PbrParameters {
                metallic: 0.8,
                ..PbrParameters::DEFAULT
            },
        ));
        let map = generated.maps().get(MapRole::Metallic).unwrap();
        let expected = (0.8 * 255.0f64).round() as u8;
        assert!(map.pixels().iter().all(|byte| *byte == expected));
    }

    #[test]
    fn the_pipeline_derives_only_what_the_material_asked_for() {
        let generated = run(definition(
            MaterialCategory::Stone,
            &[MapRole::Height, MapRole::Normal],
            PbrParameters::DEFAULT,
        ));
        assert!(generated.maps().contains(MapRole::Albedo));
        assert!(generated.maps().contains(MapRole::Height));
        assert!(generated.maps().contains(MapRole::Normal));
        // Not asked for, not produced, not paid for.
        assert!(!generated.maps().contains(MapRole::Roughness));
        assert!(!generated.maps().contains(MapRole::Metallic));
        assert!(!generated.maps().contains(MapRole::AmbientOcclusion));
    }

    #[test]
    fn asking_for_a_derived_map_with_no_height_to_derive_it_from_fails_loudly() {
        // A definition that wants a normal map but no height map. The
        // generator produces height for exactly this reason, so to reach the
        // failure the height map has to be withheld deliberately.
        let definition = definition(
            MaterialCategory::Stone,
            &[MapRole::Normal],
            PbrParameters::DEFAULT,
        );
        let generated = ProceduralGenerator::new()
            .unwrap()
            .generate(&GenerationRequest::generate(definition, 1).only(vec![MapRole::Albedo]))
            .unwrap();
        assert!(!generated.maps().contains(MapRole::Height));

        let err = PbrPipeline::new()
            .unwrap()
            .apply(generated)
            .expect_err("a normal map cannot be derived from nothing");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("height"), "{err}");
    }

    #[test]
    fn the_pipeline_records_itself_and_advances_the_revision() {
        let generated = run(definition(
            MaterialCategory::Wood,
            &[MapRole::Height, MapRole::Normal, MapRole::Roughness],
            PbrParameters::DEFAULT,
        ));
        let provenance = generated.definition().provenance();
        provenance.validate().expect("the record stays valid");

        let trace = provenance.generation.as_ref().unwrap();
        assert_eq!(trace.pipeline.as_ref().unwrap().to_string(), PBR_PIPELINE);
        assert_eq!(trace.pipeline_version, Some(PBR_VERSION));
        assert_eq!(trace.parameters["occlusion_radius"], "2");
        // Generated at v2, transformed at v3: each step is a revision.
        assert_eq!(generated.definition().revision().0, 3);
    }

    #[test]
    fn output_attributed_to_a_different_pipeline_is_refused() {
        let generated = run(definition(
            MaterialCategory::Stone,
            &[MapRole::Height, MapRole::Normal],
            PbrParameters::DEFAULT,
        ));
        let (definition, maps, _) = generated.into_parts();

        // A second pipeline, with its own identity, trying to claim this work.
        let mut impostor = PbrPipeline::new().unwrap();
        impostor.id = id("nexora:pipeline/somebody_else");

        let err = GeneratedMaterial::transformed(&impostor, definition, maps, None)
            .expect_err("a mismatched trace must be refused");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(err.to_string().contains("pipeline"), "{err}");
    }

    #[test]
    fn every_derived_map_tiles_because_every_neighbourhood_read_wraps() {
        // The normal map is the one that shows a seam first: a Sobel that
        // clamped at the edge would carry a bright line down two sides.
        let generated = run(definition(
            MaterialCategory::Stone,
            &[
                MapRole::Height,
                MapRole::Normal,
                MapRole::AmbientOcclusion,
                MapRole::Roughness,
            ],
            PbrParameters::DEFAULT,
        ));
        let edge = 32usize;

        for role in [
            MapRole::Normal,
            MapRole::AmbientOcclusion,
            MapRole::Roughness,
        ] {
            let map = generated.maps().get(role).unwrap();
            let channels = map.format().channels.count() as usize;
            let stride = edge * channels;
            let at = |x: usize, y: usize, c: usize| {
                i32::from(map.pixels()[y * stride + x * channels + c])
            };

            let (mut seam, mut interior) = (0f64, 0f64);
            for a in 0..edge {
                for c in 0..channels {
                    for b in 0..edge - 1 {
                        interior += f64::from((at(a, b, c) - at(a, b + 1, c)).abs());
                    }
                    seam += f64::from((at(a, edge - 1, c) - at(a, 0, c)).abs());
                }
            }
            let interior = interior / ((edge * (edge - 1) * channels) as f64);
            let seam = seam / ((edge * channels) as f64);
            if interior > 0.01 {
                assert!(
                    seam / interior <= 2.0,
                    "{role:?} seam ratio {:.3}",
                    seam / interior
                );
            }
        }
    }

    #[test]
    fn a_pipeline_run_twice_is_a_no_op_on_maps_that_already_exist() {
        let pipeline = PbrPipeline::new().unwrap();
        let first = run(definition(
            MaterialCategory::Stone,
            &[MapRole::Height, MapRole::Normal],
            PbrParameters::DEFAULT,
        ));
        let before: Vec<u8> = first.maps().get(MapRole::Normal).unwrap().pixels().to_vec();
        let count = first.maps().len();

        let second = pipeline.apply(first).expect("a second pass is legal");
        assert_eq!(second.maps().len(), count, "no map was added twice");
        assert_eq!(
            second.maps().get(MapRole::Normal).unwrap().pixels(),
            before,
            "an existing map is left alone rather than recomputed differently"
        );
    }

    #[test]
    fn a_material_with_no_relief_still_gets_the_roughness_it_declared() {
        let map = roughness_map(
            None,
            &definition(
                MaterialCategory::Glass,
                &[MapRole::Roughness],
                PbrParameters {
                    roughness: 0.2,
                    ..PbrParameters::DEFAULT
                },
            ),
            0.3,
            Resolution::square(8).unwrap(),
        )
        .unwrap();
        let expected = (0.2 * 255.0f64).round() as u8;
        assert!(map.pixels().iter().all(|byte| *byte == expected));

        // And a perfectly flat height field is the same case.
        let flat = field(8, |_, _| 0.5);
        let map = roughness_map(
            Some(&flat),
            &definition(
                MaterialCategory::Glass,
                &[MapRole::Roughness],
                PbrParameters {
                    roughness: 0.2,
                    ..PbrParameters::DEFAULT
                },
            ),
            0.3,
            Resolution::square(8).unwrap(),
        )
        .unwrap();
        assert!(map.pixels().iter().all(|byte| *byte == expected));
    }
}
