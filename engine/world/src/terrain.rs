//! The terrain generators: what height each column's surface has, by the
//! generator version the world was made with (ADR-0039, DEBT-0054).
//!
//! `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md` rule 1: a change of
//! algorithm changes the generator version. A world records the version it
//! was made with, so both generators stay: a world saved under version 1 keeps
//! generating version 1's columns wherever it grows, and never meets a seam
//! where the algorithm changed under it.
//!
//! * **Version 1** — the Phase 0 surface: each column's height drawn on its
//!   own from a position-seeded stream, 64 ± 12. It proved determinism, chunks
//!   and persistence, and it is a field of pillars: neighbours differ by up to
//!   24 blocks, and a player cannot walk from one to the next.
//! * **Version 2** — continuous value noise: three octaves of heights drawn on
//!   lattices 64, 16 and 8 blocks apart, interpolated bilinearly between
//!   lattice points and summed. Neighbouring columns differ by at most one
//!   block anywhere, which is what a player climbs without jumping.
//!
//! # Integers only
//!
//! Every value is an integer in 1/256 of a block, every division is
//! Euclidean, and no floating point is involved: the same seed gives the same
//! surface on every platform, and the C++ reference in `benchmarks/cpp`
//! reproduces it bit for bit (the cross-stack conformance digest).
//!
//! # Why one block is the bound
//!
//! Along either axis, bilinear interpolation changes by at most the largest
//! difference between neighbouring lattice values divided by the spacing: an
//! octave of amplitude `a` on spacing `s` moves at most `2a/s` per block. For
//! (64, 8), (16, 3) and (8, 1) that is 0.25 + 0.375 + 0.25 = 0.875, and each
//! octave's rounding adds less than 1/256, so the sum moves by at most 227 of
//! the 256 units of a block between neighbouring columns. Two values less
//! than a block apart floor to integers at most one apart. The amplitudes sum
//! to 12, so the surface stays inside version 1's range, 52 to 76, and every
//! band a client draws from [`crate::world::World::surface_range`] still holds.

use nexora_foundation::rng::{positional_rng, SeedStream};

/// The height every surface varies around, in blocks.
pub(crate) const BASE_HEIGHT: i64 = 64;

/// How far the surface strays from [`BASE_HEIGHT`], either way, in blocks.
pub(crate) const AMPLITUDE: i64 = 12;

/// Version 1: each column on its own.
#[must_use]
pub(crate) fn column_noise(seed: u64, x: i64, z: i64) -> i64 {
    let mut rng = positional_rng(seed, SeedStream::Terrain, x, 0, z);
    let span = (AMPLITUDE * 2 + 1) as u64;
    BASE_HEIGHT + (rng.next_below(span) as i64) - AMPLITUDE
}

/// Units of a block in version 2's fixed point.
const FRACTION: i64 = 256;

/// One octave of version 2: heights drawn on a square lattice `spacing`
/// blocks apart, each within `± amplitude` blocks.
#[derive(Debug, Clone, Copy)]
struct Octave {
    spacing: i64,
    amplitude: i64,
}

/// Version 2's octaves, broadest first. Their amplitudes sum to
/// [`AMPLITUDE`]; their slopes, `2 · amplitude / spacing`, to 0.875 blocks
/// per block.
const OCTAVES: [Octave; 3] = [
    Octave {
        spacing: 64,
        amplitude: 8,
    },
    Octave {
        spacing: 16,
        amplitude: 3,
    },
    Octave {
        spacing: 8,
        amplitude: 1,
    },
];

// The bound the module documentation proves, checked where it is stated:
// the octaves' amplitudes stay inside version 1's range, and their slopes,
// plus one unit of rounding each, stay under one block per block.
const _: () = {
    let mut amplitude = 0;
    let mut slope_units = 0;
    let mut index = 0;
    while index < OCTAVES.len() {
        let octave = OCTAVES[index];
        amplitude += octave.amplitude;
        slope_units += 2 * octave.amplitude * FRACTION / octave.spacing + 1;
        index += 1;
    }
    assert!(amplitude == AMPLITUDE);
    assert!(slope_units < FRACTION);
};

impl Octave {
    /// The height at lattice point `(i, j)` of octave `index`, in units.
    ///
    /// Drawn from the terrain stream at `y = index + 1`, which keeps each
    /// octave's lattice apart from the others and from version 1's columns.
    fn lattice(self, seed: u64, index: usize, i: i64, j: i64) -> i64 {
        let units = self.amplitude * FRACTION;
        let span = (units * 2 + 1) as u64;
        positional_rng(seed, SeedStream::Terrain, i, index as i64 + 1, j).next_below(span) as i64
            - units
    }

    /// This octave at column `(x, z)`, in units, given the four lattice
    /// heights around it: bilinear in exact integers, floored once.
    fn blend(self, corners: [i64; 4], x: i64, z: i64) -> i64 {
        let s = self.spacing;
        let (tx, tz) = (x.rem_euclid(s), z.rem_euclid(s));
        let [v00, v10, v01, v11] = corners;
        let numerator =
            v00 * (s - tx) * (s - tz) + v10 * tx * (s - tz) + v01 * (s - tx) * tz + v11 * tx * tz;
        numerator.div_euclid(s * s)
    }
}

/// Version 2 at one column.
#[must_use]
pub(crate) fn continuous(seed: u64, x: i64, z: i64) -> i64 {
    let mut total = 0;
    for (index, octave) in OCTAVES.iter().enumerate() {
        let (i, j) = (x.div_euclid(octave.spacing), z.div_euclid(octave.spacing));
        let corners = [
            octave.lattice(seed, index, i, j),
            octave.lattice(seed, index, i + 1, j),
            octave.lattice(seed, index, i, j + 1),
            octave.lattice(seed, index, i + 1, j + 1),
        ];
        total += octave.blend(corners, x, z);
    }
    BASE_HEIGHT + total.div_euclid(FRACTION)
}

/// Version 2 over a rectangle of columns, row by row (`z` outer, `x` inner):
/// what [`continuous`] gives column by column, with each lattice point drawn
/// once rather than once for every column that reads it. A 32 × 32 chunk
/// draws 38 lattice points instead of 12,288.
#[must_use]
pub(crate) fn continuous_area(
    seed: u64,
    min_x: i64,
    min_z: i64,
    size_x: i64,
    size_z: i64,
) -> Vec<i64> {
    let mut totals = vec![0i64; (size_x * size_z) as usize];
    for (index, octave) in OCTAVES.iter().enumerate() {
        let s = octave.spacing;
        let (i0, j0) = (min_x.div_euclid(s), min_z.div_euclid(s));
        let (i1, j1) = (
            (min_x + size_x - 1).div_euclid(s) + 1,
            (min_z + size_z - 1).div_euclid(s) + 1,
        );
        let width = i1 - i0 + 1;
        let mut lattice = Vec::with_capacity((width * (j1 - j0 + 1)) as usize);
        for j in j0..=j1 {
            for i in i0..=i1 {
                lattice.push(octave.lattice(seed, index, i, j));
            }
        }
        let at = |i: i64, j: i64| lattice[((j - j0) * width + (i - i0)) as usize];
        for local_z in 0..size_z {
            for local_x in 0..size_x {
                let (x, z) = (min_x + local_x, min_z + local_z);
                let (i, j) = (x.div_euclid(s), z.div_euclid(s));
                let corners = [at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1)];
                totals[(local_z * size_x + local_x) as usize] += octave.blend(corners, x, z);
            }
        }
    }
    totals
        .into_iter()
        .map(|total| BASE_HEIGHT + total.div_euclid(FRACTION))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: [u64; 5] = [0, 1, 0x4E58_4F52, 987_654_321, u64::MAX];

    #[test]
    fn neighbouring_columns_differ_by_at_most_one_block() {
        for seed in SEEDS {
            let (min, size) = (-150i64, 300i64);
            let heights = continuous_area(seed, min, min, size, size);
            let at = |x: i64, z: i64| heights[(z * size + x) as usize];
            for z in 0..size {
                for x in 0..size {
                    if x + 1 < size {
                        assert!(
                            (at(x + 1, z) - at(x, z)).abs() <= 1,
                            "seed {seed} x {x} z {z}"
                        );
                    }
                    if z + 1 < size {
                        assert!(
                            (at(x, z + 1) - at(x, z)).abs() <= 1,
                            "seed {seed} x {x} z {z}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_surface_stays_in_the_range_and_is_not_flat() {
        for seed in SEEDS {
            let heights = continuous_area(seed, -256, -256, 512, 512);
            let (low, high) = (
                heights.iter().copied().min().unwrap(),
                heights.iter().copied().max().unwrap(),
            );
            assert!(low >= BASE_HEIGHT - AMPLITUDE && high <= BASE_HEIGHT + AMPLITUDE);
            assert!(high - low >= 8, "seed {seed}: relief {low}..{high}");
        }
    }

    #[test]
    fn an_area_is_its_columns_one_by_one() {
        // Including far out and across zero, where Euclidean division matters.
        for seed in SEEDS {
            for (min_x, min_z) in [(-40, -40), (0, 0), (1 << 40, -(1 << 40)), (-7, 33)] {
                let area = continuous_area(seed, min_x, min_z, 37, 21);
                for local_z in 0..21 {
                    for local_x in 0..37 {
                        assert_eq!(
                            area[(local_z * 37 + local_x) as usize],
                            continuous(seed, min_x + local_x, min_z + local_z)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_surface_and_another_does_not() {
        let a = continuous_area(7, -32, -32, 64, 64);
        assert_eq!(a, continuous_area(7, -32, -32, 64, 64));
        assert_ne!(a, continuous_area(8, -32, -32, 64, 64));
    }

    #[test]
    fn version_one_is_still_a_field_of_pillars() {
        // Kept, unchanged, for the worlds made with it.
        let jumps = (0..256)
            .filter(|x| (column_noise(3, x + 1, 0) - column_noise(3, *x, 0)).abs() > 1)
            .count();
        assert!(jumps > 100, "{jumps}");
    }
}
