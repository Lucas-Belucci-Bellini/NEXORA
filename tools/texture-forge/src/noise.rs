//! Tileable value noise.
//!
//! Every function here takes texture coordinates in `0.0..1.0` and a lattice
//! size in cells, and wraps the lattice at that size. **Tiling is therefore a
//! property of the noise rather than a correction applied afterwards** — which
//! matters, because the brief §7 makes seamlessness a validated requirement,
//! and a texture that tiles by construction passes that check for a reason
//! instead of by luck.
//!
//! The lattice value comes from FNV-1a over the wrapped cell coordinates and
//! the seed, so it is a pure function of its inputs: no state, no order
//! dependence, and the same result on every platform. That is the same
//! reasoning `nexora_foundation::rng::positional_rng` uses for world
//! generation, and it is here for the same reason — generating the right half
//! of a texture first must not change the left half.

use nexora_foundation::hashing::Fnv1a64;

/// Most octaves a single call will accumulate.
///
/// Each octave doubles the lattice, so eight octaves is a 256× lattice — far
/// past the point where another octave changes a pixel, and comfortably inside
/// what the cell period can express.
pub const MAX_OCTAVES: u32 = 8;

/// A deterministic source of tileable noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Noise {
    seed: u64,
}

impl Noise {
    /// A noise field for a seed.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// A related but independent field, named rather than numbered.
    ///
    /// A recipe needs several fields — grain, mottle, speckle — and deriving
    /// them from labels rather than from `seed + 1` keeps them independent when
    /// one of them is later removed.
    #[must_use]
    pub fn stream(self, label: &str) -> Self {
        let mut hasher = Fnv1a64::new();
        hasher.write_u64(self.seed);
        hasher.write_str(label);
        Self {
            seed: hasher.finish(),
        }
    }

    /// The value at one lattice cell, in `0.0..1.0`.
    ///
    /// Coordinates wrap, so the cell to the right of the last one is the first.
    #[must_use]
    pub fn cell(self, x: i64, y: i64, period_x: u32, period_y: u32) -> f64 {
        let wrapped_x = wrap(x, period_x);
        let wrapped_y = wrap(y, period_y);
        let mut hasher = Fnv1a64::new();
        hasher.write_u64(self.seed);
        hasher.write_u64(wrapped_x as u64);
        hasher.write_u64(wrapped_y as u64);
        // The top 53 bits are the ones a double can hold exactly.
        (hasher.finish() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Smoothly interpolated noise at a texture coordinate.
    ///
    /// `u` and `v` are in `0.0..1.0`; `period_x` and `period_y` are how many
    /// lattice cells span the texture on each axis. Independent periods are
    /// what make anisotropic patterns — wood grain that is long along one axis
    /// and tight across the other — still tile.
    #[must_use]
    pub fn sample(self, u: f64, v: f64, period_x: u32, period_y: u32) -> f64 {
        let period_x = period_x.max(1);
        let period_y = period_y.max(1);
        let x = u * f64::from(period_x);
        let y = v * f64::from(period_y);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (smoothstep(x - x0), smoothstep(y - y0));
        let (x0, y0) = (x0 as i64, y0 as i64);

        let top = lerp(
            self.cell(x0, y0, period_x, period_y),
            self.cell(x0 + 1, y0, period_x, period_y),
            fx,
        );
        let bottom = lerp(
            self.cell(x0, y0 + 1, period_x, period_y),
            self.cell(x0 + 1, y0 + 1, period_x, period_y),
            fx,
        );
        lerp(top, bottom, fy)
    }

    /// Summed octaves, each at twice the lattice and half the amplitude.
    ///
    /// The result is normalised to `0.0..1.0` so a recipe can change the octave
    /// count without every other constant moving with it.
    #[must_use]
    pub fn fbm(self, u: f64, v: f64, period_x: u32, period_y: u32, octaves: u32) -> f64 {
        let octaves = octaves.clamp(1, MAX_OCTAVES);
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut normaliser = 0.0;
        for octave in 0..octaves {
            let scale = 1u32 << octave;
            // Saturating: an octave whose lattice would exceed u32 stops
            // growing rather than wrapping to a tiny period, which would show
            // up as a sudden repeat rather than as detail.
            let px = period_x.saturating_mul(scale);
            let py = period_y.saturating_mul(scale);
            total += amplitude * self.sample(u, v, px, py);
            normaliser += amplitude;
            amplitude *= 0.5;
        }
        total / normaliser
    }

    /// Ridged noise: `fbm` folded so its midline becomes a crest.
    ///
    /// What cracks, veins and mortar lines are made of. The fold is what turns
    /// a blurry cloud into a network of lines.
    #[must_use]
    pub fn ridged(self, u: f64, v: f64, period_x: u32, period_y: u32, octaves: u32) -> f64 {
        1.0 - (self.fbm(u, v, period_x, period_y, octaves) * 2.0 - 1.0).abs()
    }
}

/// Hermite smoothing, so the lattice does not show as a grid of creases.
fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn wrap(value: i64, period: u32) -> i64 {
    value.rem_euclid(i64::from(period.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_and_coordinate_always_give_the_same_value() {
        let noise = Noise::new(0xABCD_1234);
        for step in 0..64 {
            let u = f64::from(step) / 64.0;
            assert!(
                (noise.sample(u, 0.25, 8, 8) - noise.sample(u, 0.25, 8, 8)).abs() < f64::EPSILON
            );
        }
        // A second field built from the same seed is the same field.
        assert!(
            (noise.sample(0.3, 0.7, 8, 8) - Noise::new(0xABCD_1234).sample(0.3, 0.7, 8, 8)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn different_seeds_give_different_fields() {
        let a = Noise::new(1);
        let b = Noise::new(2);
        let differences = (0..64)
            .filter(|step| {
                let u = f64::from(*step) / 64.0;
                (a.sample(u, 0.5, 8, 8) - b.sample(u, 0.5, 8, 8)).abs() > 1e-6
            })
            .count();
        assert!(
            differences > 50,
            "only {differences} of 64 samples differed"
        );
    }

    #[test]
    fn named_streams_are_independent_of_each_other() {
        let base = Noise::new(7);
        let grain = base.stream("grain");
        let mottle = base.stream("mottle");
        assert_ne!(grain, mottle);
        assert_ne!(grain, base);
        // Reproducible: the same label gives the same stream.
        assert_eq!(grain, Noise::new(7).stream("grain"));
    }

    #[test]
    fn the_field_tiles_exactly_at_the_period() {
        let noise = Noise::new(99);
        for step in 0..32 {
            let t = f64::from(step) / 32.0;
            // Sampling one full period further along must land on the same
            // value: this is the property the seamless check depends on.
            assert!(
                (noise.sample(t, 0.5, 8, 8) - noise.sample(t + 1.0, 0.5, 8, 8)).abs() < 1e-12,
                "u seam at {t}"
            );
            assert!(
                (noise.sample(0.5, t, 8, 8) - noise.sample(0.5, t + 1.0, 8, 8)).abs() < 1e-12,
                "v seam at {t}"
            );
        }
    }

    #[test]
    fn summed_octaves_also_tile() {
        // Each octave doubles the lattice, so they all share the same seam. If
        // one did not, fbm would tile at its coarsest octave only.
        let noise = Noise::new(5);
        for step in 0..32 {
            let t = f64::from(step) / 32.0;
            assert!(
                (noise.fbm(t, 0.25, 4, 4, 5) - noise.fbm(t + 1.0, 0.25, 4, 4, 5)).abs() < 1e-12,
                "fbm seam at {t}"
            );
            assert!(
                (noise.ridged(0.25, t, 4, 4, 4) - noise.ridged(0.25, t + 1.0, 4, 4, 4)).abs()
                    < 1e-12,
                "ridged seam at {t}"
            );
        }
    }

    #[test]
    fn anisotropic_periods_tile_on_both_axes_independently() {
        // Wood grain: long along v, tight across u.
        let noise = Noise::new(11);
        for step in 0..32 {
            let t = f64::from(step) / 32.0;
            assert!(
                (noise.sample(t, 0.5, 16, 2) - noise.sample(t + 1.0, 0.5, 16, 2)).abs() < 1e-12
            );
            assert!(
                (noise.sample(0.5, t, 16, 2) - noise.sample(0.5, t + 1.0, 16, 2)).abs() < 1e-12
            );
        }
    }

    #[test]
    fn every_value_stays_inside_the_unit_range() {
        let noise = Noise::new(0xFFFF_FFFF_FFFF_FFFF);
        for x in 0..48i64 {
            for y in 0..48i64 {
                let (u, v) = (x as f64 / 48.0, y as f64 / 48.0);
                for value in [
                    noise.sample(u, v, 6, 6),
                    noise.fbm(u, v, 3, 3, MAX_OCTAVES),
                    noise.ridged(u, v, 3, 3, 4),
                    noise.cell(x, y, 6, 6),
                ] {
                    assert!((0.0..=1.0).contains(&value), "{value} out of range");
                }
            }
        }
    }

    #[test]
    fn octave_counts_are_clamped_rather_than_trusted() {
        let noise = Noise::new(3);
        // Zero octaves would divide by zero; far too many would overflow the
        // lattice. Both are clamped, and both still produce a usable value.
        for octaves in [0, 1, MAX_OCTAVES, u32::MAX] {
            let value = noise.fbm(0.4, 0.6, 4, 4, octaves);
            assert!(value.is_finite() && (0.0..=1.0).contains(&value));
        }
        // A zero period would be a division by zero in the lattice.
        assert!(noise.sample(0.5, 0.5, 0, 0).is_finite());
    }

    #[test]
    fn negative_lattice_coordinates_wrap_to_the_far_side() {
        let noise = Noise::new(21);
        // Cell -1 of an 8-cell lattice is cell 7. Without euclidean wrapping
        // this is where a texture grows a bright line down one edge.
        assert!((noise.cell(-1, 0, 8, 8) - noise.cell(7, 0, 8, 8)).abs() < f64::EPSILON);
        assert!((noise.cell(0, -3, 8, 8) - noise.cell(0, 5, 8, 8)).abs() < f64::EPSILON);
        assert!((noise.cell(8, 8, 8, 8) - noise.cell(0, 0, 8, 8)).abs() < f64::EPSILON);
    }
}
