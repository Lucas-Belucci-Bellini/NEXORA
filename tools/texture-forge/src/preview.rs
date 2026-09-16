//! A lit render of a material, for a person to look at.
//!
//! The brief §14 asks for previews. This is the one part of the forge whose
//! output nothing in the game ever reads — which is exactly why it is worth
//! having: five grey PNGs beside a colour one do not tell anybody whether the
//! surface *works*, and the fastest way to find out is to look at it lit.
//!
//! # It is tiled two by two, because that is where the defect is
//!
//! A one-tile preview shows the material and hides the only failure mode that
//! matters at this stage: the seam. Tiling it puts both seams — vertical and
//! horizontal — straight through the middle of the image, where the eye lands
//! first. The validator already measures the seam and gives a number
//! ([`crate::validator::seam_ratio`]); this is the same fact in the form a
//! person can act on.
//!
//! # The light does not move
//!
//! [`LIGHT`] is a constant. A preview lit differently each time is not a
//! preview, it is a mood — two materials could not be compared, and the same
//! material could not be compared with itself after an edit.
//!
//! # What it is not
//!
//! Not a renderer. It is Lambert plus a Blinn-Phong lobe, in linear light,
//! with ambient occlusion folded into the ambient term. It is close enough
//! that an inverted normal map or a flat roughness map is obvious, which is
//! the whole job. Anything more would be a second renderer to keep in step
//! with the real one, which is a cost with no matching benefit.

use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::{MapRole, MapSet, Preview, Resolution, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// How many copies of the material appear along each axis.
///
/// Two, so both seams cross the middle of the image. One hides them; three
/// shrinks each tile without showing a seam the second copy did not already.
pub const TILES: u32 = 2;

/// Largest edge a preview is rendered at.
///
/// A bound on cost, not a judgement about detail: the maps themselves are
/// right there at full resolution. A material larger than `MAX_EDGE / TILES`
/// is box-filtered down by a whole-number factor, which its power-of-two edge
/// always permits exactly.
pub const MAX_EDGE: u32 = 256;

/// Direction the light comes from, in tangent space, normalised.
///
/// From the upper left and somewhat toward the viewer — the convention every
/// height-field illustration has used since engraving, and the one that makes
/// a bump read as a bump rather than a dent.
pub const LIGHT: [f64; 3] = [
    -0.408_248_290_463_863,
    0.408_248_290_463_863,
    0.816_496_580_927_726,
];

/// How much light reaches a surface facing away from the lamp.
///
/// Without it, half of every bump is pure black and the albedo underneath
/// cannot be judged at all. It is a *floor* the diffuse term rises from,
/// never an addition on top of it.
pub const AMBIENT: f64 = 0.28;

/// Specular reflectance of a non-metal facing the light head on.
///
/// The standard 0.04: water, plastic, wood, stone and skin all sit within a
/// hair of it, which is why every PBR renderer hard-codes the same number.
/// A metal reflects almost everything instead, so `metallic` interpolates
/// this up to 1.
///
/// Getting it wrong is not subtle. At 0.45 — the first guess here — a rough
/// bark picked up a white wash worth **56% of its own brightness**, and the
/// preview came out twice as bright as its albedo at half the saturation.
pub const DIELECTRIC_REFLECTANCE: f64 = 0.04;

/// Blinn-Phong exponent at each end of the roughness range.
///
/// A mirror is `SHININESS_MAX`; a fully rough surface is `SHININESS_MIN`,
/// which is low enough to read as a broad sheen rather than a highlight.
pub const SHININESS_MIN: f64 = 2.0;
/// See [`SHININESS_MIN`].
pub const SHININESS_MAX: f64 = 180.0;

/// Renders previews.
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewRenderer;

impl PreviewRenderer {
    /// A renderer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Render a material's maps into one lit, tiled image.
    ///
    /// # Errors
    ///
    /// Returns an error when the material has no albedo — there is nothing to
    /// light — or when the rendered image is not a valid [`Preview`].
    pub fn render(&self, definition: &SurfaceMaterial, maps: &MapSet) -> Result<Preview> {
        let Some(albedo) = maps.get(MapRole::Albedo) else {
            return Err(
                nothing_to_show("a material with no albedo cannot be previewed")
                    .with_context("material", definition.id().to_string()),
            );
        };
        let source = albedo.resolution();
        let (width, height) = (source.width, source.height);

        // Whole-number reduction only. Edges are powers of two, so a power-of-
        // two cap divides exactly and the box filter needs no partial texels.
        let factor = reduction(width.max(height));
        let out = Resolution::new(width * TILES / factor, height * TILES / factor)?;

        let normal = maps.get(MapRole::Normal);
        let roughness = maps.get(MapRole::Roughness);
        let metallic = maps.get(MapRole::Metallic);
        let occlusion = maps.get(MapRole::AmbientOcclusion);
        let fallback = definition.pbr();

        let samples = f64::from(factor * factor);
        let mut pixels = Vec::with_capacity(out.texels() as usize * 3);
        for oy in 0..out.height {
            for ox in 0..out.width {
                let mut sum = [0.0_f64; 3];
                for sy in 0..factor {
                    for sx in 0..factor {
                        // Modulo is the tiling: past the material's edge, the
                        // sample wraps to the opposite side, which is exactly
                        // what the game does and exactly what makes the seam
                        // visible when the two sides do not meet.
                        let x = (ox * factor + sx) % width;
                        let y = (oy * factor + sy) % height;
                        let lit = shade(
                            channels(albedo, x, y),
                            normal.map_or([0.0, 0.0, 1.0], |map| unpack_normal(map, x, y)),
                            roughness.map_or(fallback.roughness, |map| grey(map, x, y)),
                            metallic.map_or(fallback.metallic, |map| grey(map, x, y)),
                            occlusion.map_or(1.0, |map| grey(map, x, y)),
                        );
                        for channel in 0..3 {
                            sum[channel] += lit[channel];
                        }
                    }
                }
                for value in sum {
                    pixels.push(encode_srgb(value / samples));
                }
            }
        }
        Preview::new(out, pixels)
    }
}

/// The whole-number factor that brings `TILES * edge` under [`MAX_EDGE`].
fn reduction(edge: u32) -> u32 {
    let mut factor = 1;
    while edge * TILES / factor > MAX_EDGE {
        factor *= 2;
    }
    factor
}

/// One texel's colour, linearised, with any alpha ignored.
fn channels(map: &TextureMap, x: u32, y: u32) -> [f64; 3] {
    let count = map.format().channels.count() as usize;
    let base = (y as usize * map.resolution().width as usize + x as usize) * count;
    let pixels = map.pixels();
    let read = |offset: usize| decode_srgb(pixels[base + offset.min(count - 1)]);
    if count >= 3 {
        [read(0), read(1), read(2)]
    } else {
        let value = read(0);
        [value, value, value]
    }
}

/// One texel of a single-channel map, as a plain 0..1 number.
///
/// Not linearised: roughness, metallic and occlusion are coefficients, not
/// colours, and `MapRole::color_space` already says so.
fn grey(map: &TextureMap, x: u32, y: u32) -> f64 {
    let count = map.format().channels.count() as usize;
    let base = (y as usize * map.resolution().width as usize + x as usize) * count;
    f64::from(map.pixels()[base]) / 255.0
}

/// One texel of a normal map, back into a unit vector.
fn unpack_normal(map: &TextureMap, x: u32, y: u32) -> [f64; 3] {
    let count = map.format().channels.count() as usize;
    let base = (y as usize * map.resolution().width as usize + x as usize) * count;
    let pixels = map.pixels();
    let axis = |offset: usize| f64::from(pixels[base + offset.min(count - 1)]) / 255.0 * 2.0 - 1.0;
    let vector = [axis(0), axis(1), axis(2)];
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length <= f64::EPSILON {
        // A black texel is not a direction. Facing the viewer is the one
        // answer that cannot make the render lie about relief.
        return [0.0, 0.0, 1.0];
    }
    [vector[0] / length, vector[1] / length, vector[2] / length]
}

/// Light one texel.
///
/// # Why the terms are weighted rather than summed
///
/// The obvious version — `albedo * lambert + albedo * AMBIENT + specular` —
/// is what this was first, and it was wrong in two ways that only showed up
/// on looking at the output. A surface facing the lamp got
/// `albedo * (0.82 + 0.28)`, brighter than its own colour, and the specular
/// lobe added the *same* white to a rough bark as to a polished plate.
/// Measured against the albedo it renders, the result was twice as bright at
/// half the saturation: a grey-blue picture of a brown texture.
///
/// So ambient is a floor the diffuse term rises from, never an addition —
/// a fully lit texel is exactly its own colour — the lobe is scaled by
/// `1 - roughness`, because a rough surface spreads the same light over a
/// wider lobe and so peaks lower, and its strength is
/// [`DIELECTRIC_REFLECTANCE`] rather than a number picked to look bright.
///
/// # Why metals keep a body
///
/// A metal has no diffuse term: it reflects its surroundings and nothing else.
/// Modelled honestly, with one lamp and no environment, an iron plate previews
/// as **black** — measured at luma 14 against its albedo's 137 — which tells
/// the person looking at it nothing whatsoever.
///
/// There is no environment map here and there should not be one, so the body
/// term stands in for the reflected surroundings: the material's own colour at
/// the same intensity a dielectric's diffuse would receive. For the rough
/// metals this tool produces that is close to the truth anyway — a blurred
/// reflection of an evenly lit room *is* a tinted body — and `metallic` still
/// does its real work in the highlight, which goes from a dim white spot to a
/// bright colour-tinted one.
fn shade(
    albedo: [f64; 3],
    normal: [f64; 3],
    roughness: f64,
    metallic: f64,
    occlusion: f64,
) -> [f64; 3] {
    let lambert = dot(normal, LIGHT).max(0.0);
    let floor = AMBIENT * occlusion;
    let light = floor + (1.0 - floor) * lambert;

    // The viewer looks straight down the surface, so the halfway vector is the
    // light plus the view direction, normalised.
    let half = normalise([LIGHT[0], LIGHT[1], LIGHT[2] + 1.0]);
    let shininess = SHININESS_MIN + (SHININESS_MAX - SHININESS_MIN) * (1.0 - roughness).powi(2);
    let reflectance = DIELECTRIC_REFLECTANCE + (1.0 - DIELECTRIC_REFLECTANCE) * metallic;
    let lobe =
        dot(normal, half).max(0.0).powf(shininess) * reflectance * (1.0 - roughness) * occlusion;

    let mut lit = [0.0; 3];
    for channel in 0..3 {
        // A metal tints its highlight with its own colour; a dielectric's is
        // the colour of the lamp. Interpolating between the two is the
        // smallest model that gets both looking like themselves.
        let tint = 1.0 + (albedo[channel] - 1.0) * metallic;
        lit[channel] = albedo[channel] * light + tint * lobe;
    }
    lit
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalise(v: [f64; 3]) -> [f64; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length <= f64::EPSILON {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / length, v[1] / length, v[2] / length]
}

/// sRGB byte to linear 0..1.
///
/// Lighting in sRGB space is the single mistake that makes a render look
/// plausible and be wrong: mid-tones come out washed and the highlight sits in
/// the wrong place. Two transfer functions are cheaper than explaining that.
fn decode_srgb(byte: u8) -> f64 {
    let value = f64::from(byte) / 255.0;
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear 0..1 back to an sRGB byte, clamped.
fn encode_srgb(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.003_130_8 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

fn nothing_to_show(message: &'static str) -> Error {
    Error::new(Domain::Content, "preview", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::{MaterialCategory, PbrParameters};
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::TextureFormat;
    use nexora_foundation::ident::Identifier;

    fn definition(edge: u32, wants: &[MapRole]) -> SurfaceMaterial {
        let mut builder = SurfaceMaterial::builder(
            Identifier::parse("nexora:material/sample").unwrap(),
            MaterialCategory::Stone,
            Resolution::square(edge).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .pbr(PbrParameters {
            metallic: 0.0,
            roughness: 0.5,
            ..PbrParameters::DEFAULT
        });
        for role in wants {
            builder = builder.wants(*role);
        }
        builder.build().unwrap()
    }

    fn map(role: MapRole, edge: u32, fill: &[u8]) -> TextureMap {
        let layout = role.natural_layout();
        let count = layout.count() as usize;
        let mut pixels = Vec::with_capacity((edge * edge) as usize * count);
        for _ in 0..(edge * edge) {
            for channel in 0..count {
                pixels.push(fill[channel.min(fill.len() - 1)]);
            }
        }
        TextureMap::new(
            role,
            TextureFormat::eight_bit(layout),
            Resolution::square(edge).unwrap(),
            pixels,
        )
        .unwrap()
    }

    fn albedo_only(edge: u32, colour: &[u8]) -> MapSet {
        let mut maps = MapSet::new();
        maps.insert(map(MapRole::Albedo, edge, colour)).unwrap();
        maps
    }

    #[test]
    fn a_preview_is_the_material_tiled_twice_on_each_axis() {
        let preview = PreviewRenderer::new()
            .render(&definition(16, &[]), &albedo_only(16, &[180, 120, 60, 255]))
            .unwrap();
        assert_eq!(preview.resolution().width, 32);
        assert_eq!(preview.resolution().height, 32);
        assert_eq!(preview.pixels().len(), 32 * 32 * 3);
    }

    #[test]
    fn a_large_material_is_reduced_by_a_whole_number_and_never_exceeds_the_cap() {
        // The cap is a bound on cost; the reduction has to stay exact, because
        // a fractional one would need partial texels the box filter has not
        // got.
        for edge in [16_u32, 64, 128, 256, 512, 1024] {
            let factor = reduction(edge);
            assert!(factor.is_power_of_two(), "{edge} gave {factor}");
            assert_eq!(
                edge * TILES % factor,
                0,
                "{edge} does not divide by {factor}"
            );
            assert!(
                edge * TILES / factor <= MAX_EDGE,
                "{edge} produced {} which is over the cap",
                edge * TILES / factor
            );
        }
        // And a material already under the cap is not touched.
        assert_eq!(reduction(64), 1);
        assert_eq!(reduction(128), 1);
        assert_eq!(reduction(256), 2);
    }

    #[test]
    fn the_reduced_preview_is_the_size_the_reduction_says() {
        let preview = PreviewRenderer::new()
            .render(
                &definition(512, &[]),
                &albedo_only(512, &[128, 128, 128, 255]),
            )
            .unwrap();
        assert_eq!(preview.resolution().width, MAX_EDGE);
        assert_eq!(preview.resolution().height, MAX_EDGE);
    }

    #[test]
    fn a_material_with_no_albedo_is_refused_rather_than_rendered_black() {
        let error = PreviewRenderer::new()
            .render(&definition(16, &[]), &MapSet::new())
            .expect_err("there is nothing to light");
        assert!(error.to_string().contains("no albedo"), "{error}");
    }

    #[test]
    fn the_four_corners_of_the_tiling_are_the_same_pixel() {
        // What tiling means, checked rather than assumed: the texel at (0,0)
        // of each of the four copies is one texel of the source, so all four
        // must come out identical. If the modulo were wrong they would not.
        let edge = 16;
        let mut maps = albedo_only(edge, &[200, 100, 50, 255]);
        // Give it relief so the shading actually varies across the image and
        // the test is not passing on a flat colour.
        maps.insert(map(MapRole::Normal, edge, &[140, 110, 250]))
            .unwrap();

        let preview = PreviewRenderer::new()
            .render(&definition(edge, &[MapRole::Normal]), &maps)
            .unwrap();
        let at = |x: u32, y: u32| {
            let base = (y as usize * preview.resolution().width as usize + x as usize) * 3;
            &preview.pixels()[base..base + 3]
        };
        assert_eq!(at(0, 0), at(edge, 0));
        assert_eq!(at(0, 0), at(0, edge));
        assert_eq!(at(0, 0), at(edge, edge));
    }

    #[test]
    fn relief_changes_the_render_and_a_flat_normal_does_not() {
        let edge = 16;
        let plain = PreviewRenderer::new()
            .render(
                &definition(edge, &[]),
                &albedo_only(edge, &[200, 200, 200, 255]),
            )
            .unwrap();

        // A normal map encoding exactly (0,0,1) is what "no relief" means, so
        // it must render the same as having no normal map at all.
        let mut flat = albedo_only(edge, &[200, 200, 200, 255]);
        flat.insert(map(MapRole::Normal, edge, &[128, 128, 255]))
            .unwrap();
        let unchanged = PreviewRenderer::new()
            .render(&definition(edge, &[MapRole::Normal]), &flat)
            .unwrap();

        // A normal tilted toward the light must be brighter than one tilted
        // away, which is the property that makes an inverted map obvious.
        let mut toward = albedo_only(edge, &[200, 200, 200, 255]);
        toward
            .insert(map(MapRole::Normal, edge, &[40, 210, 200]))
            .unwrap();
        let mut away = albedo_only(edge, &[200, 200, 200, 255]);
        away.insert(map(MapRole::Normal, edge, &[210, 40, 200]))
            .unwrap();

        let lit = |maps: &MapSet| {
            u32::from(
                PreviewRenderer::new()
                    .render(&definition(edge, &[MapRole::Normal]), maps)
                    .unwrap()
                    .pixels()[0],
            )
        };
        assert_eq!(
            plain.pixels()[0],
            unchanged.pixels()[0],
            "a flat normal map is the same as no normal map"
        );
        assert!(
            lit(&toward) > lit(&away),
            "toward {} should out-light away {}",
            lit(&toward),
            lit(&away)
        );
    }

    #[test]
    fn occlusion_darkens_and_a_white_occlusion_map_does_not() {
        let edge = 8;
        let brightness = |value: u8| {
            let mut maps = albedo_only(edge, &[200, 200, 200, 255]);
            maps.insert(map(MapRole::AmbientOcclusion, edge, &[value]))
                .unwrap();
            u32::from(
                PreviewRenderer::new()
                    .render(&definition(edge, &[MapRole::AmbientOcclusion]), &maps)
                    .unwrap()
                    .pixels()[0],
            )
        };
        let none = u32::from(
            PreviewRenderer::new()
                .render(
                    &definition(edge, &[]),
                    &albedo_only(edge, &[200, 200, 200, 255]),
                )
                .unwrap()
                .pixels()[0],
        );
        assert_eq!(brightness(255), none, "fully lit is the same as unoccluded");
        assert!(brightness(0) < brightness(255), "occlusion must darken");
    }

    #[test]
    fn a_metal_previews_as_metal_rather_than_as_black() {
        // What the physically honest model gave: iron at luma 14 against an
        // albedo of 137, because a metal with no environment to reflect has
        // nothing to show. A preview nobody can read is not a preview.
        let edge = 16;
        let iron = [130_u8, 136, 144, 255];
        let mut maps = albedo_only(edge, &iron);
        maps.insert(map(MapRole::Metallic, edge, &[255])).unwrap();
        maps.insert(map(MapRole::Roughness, edge, &[90])).unwrap();

        let preview = PreviewRenderer::new()
            .render(
                &definition(edge, &[MapRole::Metallic, MapRole::Roughness]),
                &maps,
            )
            .unwrap();
        let (mean, _) = statistics(&preview);
        let luma = (mean[0] + mean[1] + mean[2]) / 3.0;
        let source = (f64::from(iron[0]) + f64::from(iron[1]) + f64::from(iron[2])) / 3.0;
        assert!(
            luma >= source * 0.5,
            "a metal came back at {luma:.0} against an albedo of {source:.0}"
        );
    }

    #[test]
    fn the_transfer_functions_are_inverses() {
        // The one place a sign or an exponent error would go unnoticed: the
        // render would simply look a bit off, and nobody could say why.
        for byte in 0_u8..=255 {
            assert_eq!(
                encode_srgb(decode_srgb(byte)),
                byte,
                "round trip failed at {byte}"
            );
        }
    }

    #[test]
    fn the_light_is_a_unit_vector() {
        // Everything downstream assumes it. A light of length 1.2 would make
        // every surface 20% brighter and look like a tuning problem.
        let length = (LIGHT[0] * LIGHT[0] + LIGHT[1] * LIGHT[1] + LIGHT[2] * LIGHT[2]).sqrt();
        assert!((length - 1.0).abs() < 1e-12, "length is {length}");
    }

    #[test]
    fn rendering_the_same_material_twice_gives_the_same_image() {
        let maps = albedo_only(32, &[90, 140, 70, 255]);
        let once = PreviewRenderer::new()
            .render(&definition(32, &[]), &maps)
            .unwrap();
        let twice = PreviewRenderer::new()
            .render(&definition(32, &[]), &maps)
            .unwrap();
        assert_eq!(once, twice);
    }

    /// Mean channel values of a preview, and the spread between them.
    fn statistics(preview: &Preview) -> ([f64; 3], f64) {
        let pixels = preview.pixels();
        let count = (pixels.len() / 3) as f64;
        let mut mean = [0.0; 3];
        for texel in pixels.chunks_exact(3) {
            for channel in 0..3 {
                mean[channel] += f64::from(texel[channel]);
            }
        }
        for value in &mut mean {
            *value /= count;
        }
        let spread = mean[0].max(mean[1]).max(mean[2]) - mean[0].min(mean[1]).min(mean[2]);
        (mean, spread)
    }

    #[test]
    fn a_preview_does_not_wash_out_the_material_it_depicts() {
        // The bug this test exists for: summing `albedo * lambert`,
        // `albedo * AMBIENT` and a full-strength white lobe produced a picture
        // twice as bright as its albedo at half the saturation — brown bark
        // rendered grey-blue. The failure was invisible in every other test,
        // because every other test asks whether a number moved, not whether
        // the image still looks like the material.
        let edge = 32;
        let brown = [88_u8, 61, 38, 255];
        let mut maps = albedo_only(edge, &brown);
        maps.insert(map(MapRole::Roughness, edge, &[220])).unwrap();

        let preview = PreviewRenderer::new()
            .render(&definition(edge, &[MapRole::Roughness]), &maps)
            .unwrap();
        let (mean, spread) = statistics(&preview);

        let source_spread = f64::from(brown[0] - brown[2]);
        let source_luma = f64::from(brown[0] + brown[1] + brown[2]) / 3.0;
        let luma = (mean[0] + mean[1] + mean[2]) / 3.0;

        assert!(
            luma <= source_luma * 1.05,
            "a lit texture must not be brighter than its own albedo: {luma:.1} vs {source_luma:.1}"
        );
        assert!(
            spread >= source_spread * 0.7,
            "the render kept only {spread:.1} of the albedo's {source_spread:.1} colour spread"
        );
        // And it is still visible, not crushed to black by the correction.
        assert!(luma >= source_luma * 0.5, "too dark: {luma:.1}");
    }

    #[test]
    fn a_fully_lit_texel_is_its_own_colour_and_never_brighter() {
        // Ambient is a floor the diffuse term rises from, not something added
        // on top. A normal pointing straight at the lamp must come back as the
        // albedo itself.
        let facing = LIGHT;
        let lit = shade([0.5, 0.5, 0.5], facing, 1.0, 0.0, 1.0);
        for channel in lit {
            assert!(
                (channel - 0.5).abs() < 1e-9,
                "a fully lit mid grey came back as {channel}"
            );
        }
        // And one facing away sits at the floor, not at zero.
        let away = [-facing[0], -facing[1], -facing[2]];
        let dark = shade([0.5, 0.5, 0.5], away, 1.0, 0.0, 1.0);
        assert!((dark[0] - 0.5 * AMBIENT).abs() < 1e-9, "{}", dark[0]);
    }

    #[test]
    fn a_rough_surface_gets_less_highlight_than_a_polished_one() {
        // Measured at each lobe's own peak, which is the normal pointing at
        // the halfway vector. Comparing them at some other normal says nothing:
        // a narrow lobe is dark everywhere except where it is bright, so the
        // polished surface would "lose" simply for being aimed elsewhere.
        let peak_normal = normalise([LIGHT[0], LIGHT[1], LIGHT[2] + 1.0]);
        let peak = |roughness: f64| shade([0.2, 0.2, 0.2], peak_normal, roughness, 0.0, 1.0)[0];
        assert!(
            peak(0.1) > peak(0.9),
            "polished {} should out-shine rough {}",
            peak(0.1),
            peak(0.9)
        );
        // A fully rough surface has no highlight at all: just albedo and light.
        let matte = shade([0.2, 0.2, 0.2], peak_normal, 1.0, 0.0, 1.0)[0];
        let expected = 0.2 * (AMBIENT + (1.0 - AMBIENT) * dot(peak_normal, LIGHT).max(0.0));
        assert!((matte - expected).abs() < 1e-9, "{matte} vs {expected}");
    }

    #[test]
    fn only_a_highlight_may_clip_and_it_may_clip_by_very_little() {
        // A clamp hides an overflow rather than preventing it. A specular
        // highlight blowing out is what a highlight *is*, so the bound is the
        // surface's own reflectance above white — a mirror facing the lamp
        // saturates, a plastic one barely clips, and neither is unbounded.
        for albedo in [0.0, 0.5, 1.0] {
            for roughness in [0.0, 0.5, 1.0] {
                for metallic in [0.0, 0.5, 1.0] {
                    for occlusion in [0.0, 0.5, 1.0] {
                        for normal in [
                            [0.0, 0.0, 1.0],
                            LIGHT,
                            normalise([LIGHT[0], LIGHT[1], LIGHT[2] + 1.0]),
                            [-LIGHT[0], -LIGHT[1], -LIGHT[2]],
                        ] {
                            let lit = shade([albedo; 3], normal, roughness, metallic, occlusion);
                            let reflectance =
                                DIELECTRIC_REFLECTANCE + (1.0 - DIELECTRIC_REFLECTANCE) * metallic;
                            for channel in lit {
                                assert!(
                                    channel >= 0.0 && channel <= 1.0 + reflectance,
                                    "albedo {albedo} roughness {roughness} metallic {metallic} \
                                     occlusion {occlusion} gave {channel}"
                                );
                            }
                        }
                    }
                }
            }
        }

        // And a matte surface never exceeds its own albedo at all.
        for normal in [[0.0, 0.0, 1.0], LIGHT, [0.0, 1.0, 0.0]] {
            let lit = shade([0.6; 3], normal, 1.0, 0.0, 1.0);
            assert!(lit[0] <= 0.6 + f64::EPSILON, "{}", lit[0]);
        }
    }
}
