//! Recipes: what each family of surface is made of.
//!
//! One renderer, parameterised. `NEXORA ART DIRECTION AND PROCEDURAL
//! VARIATION.md` sets the goal — *"prefer a small library of high-quality
//! original primitives plus controlled procedural variation over massive
//! duplication of near-identical files"* — and a [`Recipe`] is what one of
//! those primitives looks like as data.
//!
//! Four structural layers compose into every surface:
//!
//! ```text
//! mottle   the field the whole surface varies over
//! strips   parallel boards, with a gap and a grain along them
//! courses  laid rows with mortar between them
//! speckle  isolated grains, brighter or darker than their surroundings
//! ```
//!
//! Wood is strips plus grain; brick is courses plus mottle; stone is mottle
//! plus cracks; sand is fine mottle plus speckle. None of them is a special
//! case in the renderer, which is what makes adding the next twenty families a
//! matter of writing parameters rather than writing code.
//!
//! # Why the palette is small on purpose
//!
//! Continuous noise mapped to 24-bit colour reads as photographic mush at 32
//! pixels. Snapping to a handful of ramp stops is what makes a texture read as
//! drawn. The step count is therefore a recipe parameter, not a post-process.

use nexora_asset::material::MaterialCategory;
use nexora_asset::texture::Resolution;
use nexora_foundation::error::Result;

use crate::color::{Ramp, Rgba};
use crate::noise::Noise;
use crate::raster::Canvas;

/// The broad variation the whole surface sits on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mottle {
    /// Lattice cells across the texture, horizontally.
    pub cells_x: u32,
    /// Lattice cells across the texture, vertically.
    pub cells_y: u32,
    /// How many octaves to sum.
    pub octaves: u32,
    /// How much of the final value this layer decides, `0.0..=1.0`.
    pub strength: f64,
    /// How much a ridged field is folded in, for cracks and veins.
    pub cracks: f64,
}

/// Parallel boards running down the texture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strips {
    /// How many boards span the texture. Tiling needs a whole number of them.
    pub count: u32,
    /// Fraction of a board's width taken by the gap beside it.
    pub gap: f64,
    /// How much brightness varies from board to board.
    pub jitter: f64,
    /// Lattice cells across one board, for the grain.
    pub grain_across: u32,
    /// Lattice cells along the board. Small, so the grain runs lengthwise.
    pub grain_along: u32,
    /// How strongly the grain shows.
    pub grain_strength: f64,
}

/// Laid rows, offset course by course.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Courses {
    /// Rows down the texture. Must be even for the offset to tile.
    pub rows: u32,
    /// Units across each row.
    pub columns: u32,
    /// Fraction of a unit the mortar occupies.
    pub mortar: f64,
    /// How much brightness varies from unit to unit.
    pub jitter: f64,
}

/// Isolated grains scattered over the surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speckle {
    /// Grain cells across the texture.
    pub cells: u32,
    /// Fraction of cells that carry a grain.
    pub density: f64,
    /// How far a grain departs from its surroundings.
    pub contrast: f64,
}

/// Everything one family of surface is made of.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    /// The palette, darkest first.
    pub ramp: Ramp,
    /// The broad variation.
    pub mottle: Mottle,
    /// Boards, when the surface has them.
    pub strips: Option<Strips>,
    /// Courses, when the surface has them.
    pub courses: Option<Courses>,
    /// Grains, when the surface has them.
    pub speckle: Option<Speckle>,
    /// How far the value spreads from the middle before it meets the ramp.
    ///
    /// Summed value noise clusters hard around `0.5`: left alone, every
    /// surface comes out as the middle third of its own palette. This is the
    /// gain that opens it back up, and it is a recipe parameter because how
    /// much contrast a surface wants is art direction, not arithmetic.
    pub contrast: f64,
    /// Palette steps. Fewer than two means no quantisation.
    pub levels: u32,
    /// How much of the brightness becomes relief in the height field.
    pub relief: f64,
}

impl Recipe {
    /// The default recipe for a family.
    ///
    /// These are art direction expressed as numbers. They are starting points a
    /// material document overrides, not constants the renderer depends on.
    ///
    /// # Errors
    ///
    /// Returns an error only if a ramp is malformed, which would be a mistake
    /// in this table rather than in a caller.
    pub fn for_category(category: MaterialCategory) -> Result<Self> {
        let recipe = match category {
            MaterialCategory::Stone => Self {
                ramp: Ramp::between(Rgba::opaque(58, 58, 62), Rgba::opaque(148, 148, 152), 6)?,
                mottle: Mottle {
                    cells_x: 4,
                    cells_y: 4,
                    octaves: 5,
                    strength: 1.0,
                    cracks: 0.35,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 24,
                    density: 0.10,
                    contrast: 0.22,
                }),
                contrast: 2.4,
                levels: 6,
                relief: 0.55,
            },
            MaterialCategory::Wood => Self {
                // The reference: dark, warm, vertical boards.
                ramp: Ramp::between(Rgba::opaque(46, 30, 18), Rgba::opaque(146, 104, 66), 8)?,
                mottle: Mottle {
                    cells_x: 2,
                    cells_y: 2,
                    octaves: 3,
                    strength: 0.25,
                    cracks: 0.0,
                },
                strips: Some(Strips {
                    count: 4,
                    // A tenth of a board. At 32 texels and four boards that is
                    // still under a pixel; the gap therefore also darkens the
                    // board edge, so the separation survives being small.
                    gap: 0.10,
                    jitter: 0.30,
                    grain_across: 10,
                    grain_along: 2,
                    grain_strength: 0.60,
                }),
                courses: None,
                speckle: None,
                contrast: 2.6,
                levels: 8,
                relief: 0.35,
            },
            MaterialCategory::Metal => Self {
                ramp: Ramp::between(Rgba::opaque(74, 78, 86), Rgba::opaque(186, 192, 200), 5)?,
                mottle: Mottle {
                    // Brushed: fine across, long along.
                    cells_x: 48,
                    cells_y: 2,
                    octaves: 3,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: None,
                contrast: 2.2,
                levels: 5,
                relief: 0.12,
            },
            MaterialCategory::Soil => Self {
                ramp: Ramp::between(Rgba::opaque(38, 26, 16), Rgba::opaque(112, 84, 56), 6)?,
                mottle: Mottle {
                    cells_x: 6,
                    cells_y: 6,
                    octaves: 5,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 32,
                    density: 0.22,
                    contrast: 0.30,
                }),
                contrast: 2.3,
                levels: 6,
                relief: 0.70,
            },
            MaterialCategory::Sand => Self {
                ramp: Ramp::between(Rgba::opaque(168, 142, 96), Rgba::opaque(226, 206, 158), 5)?,
                mottle: Mottle {
                    cells_x: 12,
                    cells_y: 12,
                    octaves: 4,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 48,
                    density: 0.30,
                    contrast: 0.14,
                }),
                contrast: 2.6,
                levels: 5,
                relief: 0.25,
            },
            MaterialCategory::Concrete => Self {
                ramp: Ramp::between(Rgba::opaque(104, 104, 100), Rgba::opaque(176, 176, 170), 5)?,
                mottle: Mottle {
                    cells_x: 5,
                    cells_y: 5,
                    octaves: 4,
                    strength: 1.0,
                    cracks: 0.18,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 40,
                    density: 0.16,
                    contrast: 0.18,
                }),
                contrast: 2.2,
                levels: 5,
                relief: 0.30,
            },
            MaterialCategory::Brick => Self {
                ramp: Ramp::between(Rgba::opaque(96, 42, 32), Rgba::opaque(178, 96, 74), 6)?,
                mottle: Mottle {
                    cells_x: 8,
                    cells_y: 8,
                    octaves: 4,
                    strength: 0.45,
                    cracks: 0.0,
                },
                strips: None,
                courses: Some(Courses {
                    rows: 4,
                    columns: 2,
                    mortar: 0.10,
                    jitter: 0.24,
                }),
                speckle: None,
                contrast: 2.0,
                levels: 6,
                relief: 0.80,
            },
            MaterialCategory::Glass => Self {
                ramp: Ramp::between(Rgba::opaque(150, 178, 186), Rgba::opaque(206, 226, 232), 4)?,
                mottle: Mottle {
                    cells_x: 3,
                    cells_y: 3,
                    octaves: 2,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: None,
                contrast: 1.8,
                levels: 4,
                relief: 0.05,
            },
            MaterialCategory::Ceramic => Self {
                ramp: Ramp::between(Rgba::opaque(178, 168, 152), Rgba::opaque(238, 232, 222), 4)?,
                mottle: Mottle {
                    cells_x: 3,
                    cells_y: 3,
                    octaves: 3,
                    strength: 1.0,
                    cracks: 0.22,
                },
                strips: None,
                courses: None,
                speckle: None,
                contrast: 2.0,
                levels: 4,
                relief: 0.10,
            },
            MaterialCategory::Plastic => Self {
                ramp: Ramp::between(Rgba::opaque(52, 92, 128), Rgba::opaque(112, 158, 198), 4)?,
                mottle: Mottle {
                    cells_x: 2,
                    cells_y: 2,
                    octaves: 2,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: None,
                contrast: 1.8,
                levels: 4,
                relief: 0.06,
            },
            MaterialCategory::Fabric => Self {
                ramp: Ramp::between(Rgba::opaque(72, 60, 78), Rgba::opaque(150, 134, 158), 5)?,
                mottle: Mottle {
                    // A weave: two tight lattices at right angles.
                    cells_x: 32,
                    cells_y: 32,
                    octaves: 2,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 32,
                    density: 0.35,
                    contrast: 0.12,
                }),
                contrast: 2.2,
                levels: 5,
                relief: 0.20,
            },
            MaterialCategory::Organic => Self {
                ramp: Ramp::between(Rgba::opaque(52, 46, 28), Rgba::opaque(134, 126, 82), 6)?,
                mottle: Mottle {
                    cells_x: 5,
                    cells_y: 5,
                    octaves: 5,
                    strength: 1.0,
                    cracks: 0.28,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 28,
                    density: 0.18,
                    contrast: 0.24,
                }),
                contrast: 2.4,
                levels: 6,
                relief: 0.50,
            },
            MaterialCategory::Mineral => Self {
                ramp: Ramp::between(Rgba::opaque(38, 62, 78), Rgba::opaque(126, 190, 214), 7)?,
                mottle: Mottle {
                    cells_x: 4,
                    cells_y: 4,
                    octaves: 4,
                    strength: 0.8,
                    cracks: 0.55,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 20,
                    density: 0.08,
                    contrast: 0.40,
                }),
                contrast: 2.6,
                levels: 7,
                relief: 0.65,
            },
            MaterialCategory::Vegetation => Self {
                ramp: Ramp::between(Rgba::opaque(30, 58, 28), Rgba::opaque(108, 158, 72), 6)?,
                mottle: Mottle {
                    cells_x: 7,
                    cells_y: 7,
                    octaves: 4,
                    strength: 1.0,
                    cracks: 0.30,
                },
                strips: None,
                courses: None,
                speckle: Some(Speckle {
                    cells: 26,
                    density: 0.20,
                    contrast: 0.26,
                }),
                contrast: 2.4,
                levels: 6,
                relief: 0.45,
            },
            MaterialCategory::Custom => Self {
                ramp: Ramp::between(Rgba::opaque(80, 80, 80), Rgba::opaque(180, 180, 180), 5)?,
                mottle: Mottle {
                    cells_x: 6,
                    cells_y: 6,
                    octaves: 4,
                    strength: 1.0,
                    cracks: 0.0,
                },
                strips: None,
                courses: None,
                speckle: None,
                contrast: 2.2,
                levels: 5,
                relief: 0.40,
            },
        };
        Ok(recipe)
    }

    /// Render the recipe onto a canvas.
    ///
    /// Pure: the same recipe, resolution and seed produce the same pixels,
    /// every time and in any order. Nothing here reads a clock, a counter or a
    /// previous texel.
    #[must_use]
    pub fn render(&self, resolution: Resolution, seed: u64) -> Canvas {
        let mut canvas = Canvas::new(resolution);
        let noise = Noise::new(seed);
        let mottle_field = noise.stream("mottle");
        let crack_field = noise.stream("crack");
        let grain_field = noise.stream("grain");
        let jitter_field = noise.stream("jitter");
        let speckle_field = noise.stream("speckle");

        let (width, height) = (resolution.width, resolution.height);
        for y in 0..height {
            // Texel centres, so the pattern is symmetric about the texture
            // rather than biased towards the top-left corner.
            let v = (f64::from(y) + 0.5) / f64::from(height);
            for x in 0..width {
                let u = (f64::from(x) + 0.5) / f64::from(width);

                let mut value = self.mottle_value(mottle_field, crack_field, u, v);
                let mut relief = value;

                if let Some(strips) = self.strips {
                    let (shade, depth) = strip_layer(strips, grain_field, jitter_field, u, v);
                    value =
                        (value * (1.0 - strips.grain_strength)) + (shade * strips.grain_strength);
                    relief = relief.mul_add(0.35, depth * 0.65);
                }

                if let Some(courses) = self.courses {
                    let (shade, depth) = course_layer(courses, jitter_field, u, v);
                    value = value.mul_add(0.45, shade * 0.55);
                    relief = relief.mul_add(0.30, depth * 0.70);
                }

                if let Some(speckle) = self.speckle {
                    value += speckle_layer(speckle, speckle_field, u, v);
                }

                let value = expand(value, self.contrast);
                let colour = if self.levels >= 2 {
                    self.ramp.quantised(value)
                } else {
                    self.ramp.sample(value)
                };
                let height_value = (relief.clamp(0.0, 1.0) * self.relief).clamp(0.0, 1.0);
                canvas.put(x, y, colour, height_value);
            }
        }
        canvas
    }

    fn mottle_value(&self, field: Noise, cracks: Noise, u: f64, v: f64) -> f64 {
        let base = field.fbm(
            u,
            v,
            self.mottle.cells_x,
            self.mottle.cells_y,
            self.mottle.octaves,
        );
        let mut value = 0.5f64.mul_add(1.0 - self.mottle.strength, base * self.mottle.strength);
        if self.mottle.cracks > 0.0 {
            // A ridged field crests along thin lines; subtracting it cuts them
            // into the surface instead of laying them on top.
            let ridge = cracks.ridged(
                u,
                v,
                self.mottle.cells_x,
                self.mottle.cells_y,
                self.mottle.octaves.min(4),
            );
            value -= self.mottle.cracks * ridge.powi(4);
        }
        value.clamp(0.0, 1.0)
    }
}

/// Spread a value away from the middle, then clamp.
///
/// Summed value noise is roughly normal about `0.5` with a standard deviation
/// near `0.15`, so without this a six-stop ramp only ever shows its middle two
/// stops — which is exactly what "the texture looks flat" means.
fn expand(value: f64, gain: f64) -> f64 {
    (value - 0.5).mul_add(gain, 0.5).clamp(0.0, 1.0)
}

/// One board's contribution: how bright it is, and how proud of the gap.
fn strip_layer(strips: Strips, grain: Noise, jitter: Noise, u: f64, v: f64) -> (f64, f64) {
    let count = strips.count.max(1);
    let position = u * f64::from(count);
    let index = position.floor();
    // Where in this board we are, `0.0` at one edge and `1.0` at the other.
    let across = position - index;

    // A per-board offset drawn from the lattice, so board three is the same
    // shade every time this texture is generated.
    let board = jitter.cell(index as i64, 0, count, 1);
    let shade_offset = (board - 0.5) * 2.0 * strips.jitter;

    // The grain runs along the board: many cells across, few along.
    // Sampled in texture space with a lattice that is fine across the boards
    // and coarse along them, which is what makes the grain run lengthwise. Two
    // octaves, because one is a smooth blur and three is sandpaper.
    let grain_value = grain.fbm(
        u,
        v,
        strips.grain_across.saturating_mul(count).max(1),
        strips.grain_along.max(1),
        2,
    );

    let gap = strips.gap.clamp(0.0, 0.45);
    let edge = across.min(1.0 - across);
    if edge < gap {
        // Inside the gap. Dark and recessed, and the closer to the centre of
        // the gap the darker, so two boards do not butt into a hard line.
        let depth = (edge / gap.max(f64::EPSILON)).clamp(0.0, 1.0);
        return (grain_value * 0.18 * depth, depth * 0.30);
    }

    let value = (grain_value + shade_offset).clamp(0.0, 1.0);
    // Slight rounding towards each board's edges reads as a chamfer.
    let from_edge = ((edge - gap) / (0.5 - gap).max(f64::EPSILON)).clamp(0.0, 1.0);
    (value, 0.65 + 0.35 * from_edge)
}

/// One laid unit's contribution: brightness, and height above the mortar.
fn course_layer(courses: Courses, jitter: Noise, u: f64, v: f64) -> (f64, f64) {
    let rows = courses.rows.max(1);
    let columns = courses.columns.max(1);

    let row_position = v * f64::from(rows);
    let row = row_position.floor();
    let down = row_position - row;

    // Every other course shifts by half a unit. With an even row count the
    // shift returns to zero at the seam, so the bond tiles.
    let offset = if (row as i64).rem_euclid(2) == 1 {
        0.5
    } else {
        0.0
    };
    let column_position = u.mul_add(f64::from(columns), offset);
    let column = column_position.floor();
    let across = column_position - column;

    let unit = jitter.cell(column as i64, row as i64, columns, rows);
    let shade = 0.5 + (unit - 0.5) * 2.0 * courses.jitter;

    let mortar = courses.mortar.clamp(0.0, 0.45);
    let edge = across.min(1.0 - across).min(down.min(1.0 - down));
    if edge < mortar {
        // Mortar: flat, pale, and set back from the face of the unit.
        let depth = (edge / mortar.max(f64::EPSILON)).clamp(0.0, 1.0);
        return (0.85 - 0.25 * depth, depth * 0.25);
    }
    (shade.clamp(0.0, 1.0), 1.0)
}

/// A grain's departure from its surroundings, or zero where there is none.
fn speckle_layer(speckle: Speckle, field: Noise, u: f64, v: f64) -> f64 {
    let cells = speckle.cells.max(1);
    let x = (u * f64::from(cells)).floor() as i64;
    let y = (v * f64::from(cells)).floor() as i64;
    let present = field.cell(x, y, cells, cells);
    if present >= speckle.density.clamp(0.0, 1.0) {
        return 0.0;
    }
    // Which way the grain departs is decided by a second hash, so grains are
    // not all brighter than the surface they sit on.
    let direction = field.stream("polarity").cell(x, y, cells, cells);
    let sign = if direction < 0.5 { -1.0 } else { 1.0 };
    sign * speckle.contrast
}
