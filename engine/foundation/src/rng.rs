//! Deterministic pseudo-random number generation.
//!
//! Implements `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md`. The
//! contract is that a world can be recreated from versioned inputs, which means
//! two things this module must guarantee:
//!
//! * **Platform independence.** The algorithm is fixed arithmetic on `u64`, so
//!   a world generated on one machine is identical on every other machine.
//! * **Stream separation.** Terrain, caves, loot and history each draw from
//!   their own derived stream. Without that, adding one call to the cave
//!   generator would shift every later terrain value and change the whole world.
//!
//! The generator is SplitMix64 (Steele, Lea and Flood, 2014), implemented here
//! from the published algorithm.

use crate::hashing::Fnv1a64;
use crate::version::GeneratorVersion;

/// SplitMix64 increment (the golden-ratio odd constant).
const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// A deterministic random stream.
///
/// Cloning a generator clones its position, so a subsystem can fork a stream
/// and replay it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Start a stream from a raw seed.
    #[must_use]
    pub const fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Draw the next 64 bits and advance the stream.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Draw the next 32 bits.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Draw a value in `0..bound`.
    ///
    /// Uses Lemire's multiply-shift reduction with rejection, so the result is
    /// unbiased rather than the skew that a plain `% bound` would introduce.
    /// Returns `0` when `bound` is `0`.
    pub fn next_below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        // Reject the short tail so every value in 0..bound is equally likely.
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let draw = self.next_u64();
            let (high, low) = widening_mul(draw, bound);
            if low >= threshold {
                return high;
            }
        }
    }

    /// Draw a value in `0.0..1.0`.
    ///
    /// Built from the top 53 bits, which is the full mantissa precision of an
    /// `f64` and avoids the uneven spacing of a naive division.
    pub fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / (1u64 << 53) as f64;
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    /// Draw a boolean with even odds.
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    /// Fork an independent sub-stream labelled by `label`.
    ///
    /// The parent stream advances once, so forking is itself deterministic.
    pub fn fork(&mut self, label: &str) -> Self {
        let mut hasher = Fnv1a64::new();
        hasher.write_u64(self.next_u64());
        hasher.write_str(label);
        Self::from_seed(hasher.finish())
    }
}

/// 64x64 -> 128 bit multiply, returned as `(high, low)`.
const fn widening_mul(a: u64, b: u64) -> (u64, u64) {
    let product = (a as u128) * (b as u128);
    ((product >> 64) as u64, product as u64)
}

/// A named generation domain.
///
/// The stream list follows the "Deterministic domains" section of
/// `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SeedStream {
    /// Height, density and land shape.
    Terrain,
    /// Biome placement.
    Biome,
    /// Cave and cavity carving.
    Cave,
    /// Plant and forest placement.
    Vegetation,
    /// Weather evolution.
    Weather,
    /// Structure placement.
    Structure,
    /// Settlement and civilization seeding.
    Civilization,
    /// Agent decision jitter.
    Ai,
    /// Historical event generation.
    History,
    /// Loot rolls.
    Loot,
}

impl SeedStream {
    /// Stable stream label. Changing one of these changes every world.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Terrain => "terrain",
            Self::Biome => "biome",
            Self::Cave => "cave",
            Self::Vegetation => "vegetation",
            Self::Weather => "weather",
            Self::Structure => "structure",
            Self::Civilization => "civilization",
            Self::Ai => "ai",
            Self::History => "history",
            Self::Loot => "loot",
        }
    }
}

/// Derive a stream seed from the world seed.
#[must_use]
pub fn derive_stream_seed(world_seed: u64, stream: SeedStream) -> u64 {
    let mut hasher = Fnv1a64::new();
    hasher.write_u64(world_seed);
    hasher.write_str(stream.as_str());
    hasher.finish()
}

/// Open a stream generator for a world.
#[must_use]
pub fn stream_rng(world_seed: u64, stream: SeedStream) -> Rng {
    Rng::from_seed(derive_stream_seed(world_seed, stream))
}

/// Open a generator for one stream at one position.
///
/// Position-derived generators are what make world generation order-independent:
/// generating chunk B before chunk A must produce the same world as the reverse,
/// so a chunk's values cannot come from a shared running stream.
#[must_use]
pub fn positional_rng(world_seed: u64, stream: SeedStream, x: i64, y: i64, z: i64) -> Rng {
    let mut hasher = Fnv1a64::new();
    hasher.write_u64(derive_stream_seed(world_seed, stream));
    hasher.write_u64(x as u64);
    hasher.write_u64(y as u64);
    hasher.write_u64(z as u64);
    Rng::from_seed(hasher.finish())
}

/// The versioned inputs that together identify a reproducible world.
///
/// Matches the "Reproduction key" section of the seed document. Two worlds with
/// equal keys must generate identically; if they do not, one of these fields is
/// missing a version bump.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReproductionKey {
    /// Immutable world seed, fixed at creation.
    pub world_seed: u64,
    /// Generation algorithm version.
    pub generator_version: GeneratorVersion,
    /// Engine build that generated the world.
    pub engine_version: crate::version::EngineVersion,
    /// Content set version.
    pub content_version: crate::version::ContentVersion,
    /// Sorted `(mod_id, version)` pairs participating in generation.
    pub mods: Vec<(String, String)>,
}

impl ReproductionKey {
    /// A stable fingerprint over every field.
    ///
    /// Mods are sorted before hashing so that load order does not change the
    /// fingerprint of an otherwise identical world.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut sorted = self.mods.clone();
        sorted.sort();

        let mut hasher = Fnv1a64::new();
        hasher.write_u64(self.world_seed);
        hasher.write_u64(u64::from(self.generator_version.0));
        hasher.write_u64(u64::from(self.engine_version.major));
        hasher.write_u64(u64::from(self.engine_version.minor));
        hasher.write_u64(u64::from(self.engine_version.patch));
        hasher.write_u64(u64::from(self.content_version.0));
        hasher.write_u64(sorted.len() as u64);
        for (id, version) in &sorted {
            hasher.write_str(id);
            hasher.write_str(version);
        }
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{ContentVersion, EngineVersion};

    /// Golden values. These pin the generator: if a refactor changes them, every
    /// existing world would generate differently, which is a breaking change and
    /// must go through a generator version bump rather than slip through review.
    #[test]
    fn splitmix64_output_is_pinned() {
        let mut rng = Rng::from_seed(0);
        let drawn: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
        assert_eq!(
            drawn,
            vec![
                0xE220_A839_7B1D_CDAF,
                0x6E78_9E6A_A1B9_65F4,
                0x06C4_5D18_8009_454F,
                0xF88B_B8A8_724C_81EC,
            ]
        );
    }

    #[test]
    fn same_seed_reproduces_the_same_sequence() {
        let mut a = Rng::from_seed(0xDEAD_BEEF);
        let mut b = Rng::from_seed(0xDEAD_BEEF);
        for _ in 0..1_000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn streams_are_independent_of_each_other() {
        let world_seed = 42;
        let mut seen = std::collections::HashSet::new();
        for stream in [
            SeedStream::Terrain,
            SeedStream::Biome,
            SeedStream::Cave,
            SeedStream::Vegetation,
            SeedStream::Weather,
            SeedStream::Structure,
            SeedStream::Civilization,
            SeedStream::Ai,
            SeedStream::History,
            SeedStream::Loot,
        ] {
            assert!(
                seen.insert(derive_stream_seed(world_seed, stream)),
                "stream {stream:?} collided with another stream"
            );
        }
    }

    #[test]
    fn positional_generation_is_order_independent() {
        let seed = 7;
        // Two positions must not influence each other, in either visit order.
        let a1 = positional_rng(seed, SeedStream::Terrain, 10, 0, -4).next_u64();
        let b1 = positional_rng(seed, SeedStream::Terrain, -100, 3, 55).next_u64();
        let b2 = positional_rng(seed, SeedStream::Terrain, -100, 3, 55).next_u64();
        let a2 = positional_rng(seed, SeedStream::Terrain, 10, 0, -4).next_u64();
        assert_eq!(a1, a2);
        assert_eq!(b1, b2);
        assert_ne!(a1, b1);
    }

    #[test]
    fn different_world_seeds_give_different_worlds() {
        assert_ne!(
            derive_stream_seed(1, SeedStream::Terrain),
            derive_stream_seed(2, SeedStream::Terrain)
        );
    }

    #[test]
    fn bounded_draws_stay_in_range_and_cover_it() {
        let mut rng = stream_rng(99, SeedStream::Loot);
        let mut hits = [0u32; 6];
        for _ in 0..60_000 {
            let roll = rng.next_below(6);
            assert!(roll < 6);
            hits[roll as usize] += 1;
        }
        // Every face must appear; a badly bounded generator collapses to a subset.
        assert!(
            hits.iter().all(|&count| count > 8_000),
            "uneven distribution: {hits:?}"
        );
        assert_eq!(rng.next_below(0), 0);
        assert_eq!(rng.next_below(1), 0);
    }

    #[test]
    fn unit_floats_stay_in_the_half_open_unit_interval() {
        let mut rng = stream_rng(5, SeedStream::Weather);
        for _ in 0..10_000 {
            let value = rng.next_f64();
            assert!((0.0..1.0).contains(&value), "{value} escaped 0.0..1.0");
        }
    }

    #[test]
    fn forked_streams_diverge_by_label() {
        let mut parent = Rng::from_seed(11);
        let mut left = parent.clone().fork("left");
        let mut right = parent.clone().fork("right");
        assert_ne!(left.next_u64(), right.next_u64());
        // Forking advances the parent, so it is itself reproducible.
        let forked_once = parent.fork("child").next_u64();
        let mut replay = Rng::from_seed(11);
        assert_eq!(replay.fork("child").next_u64(), forked_once);
    }

    #[test]
    fn reproduction_key_ignores_mod_load_order() {
        let base = ReproductionKey {
            world_seed: 1234,
            generator_version: GeneratorVersion(1),
            engine_version: EngineVersion::new(0, 0, 1),
            content_version: ContentVersion(1),
            mods: vec![
                ("alpha".into(), "1.0".into()),
                ("beta".into(), "2.0".into()),
            ],
        };
        let reordered = ReproductionKey {
            mods: vec![
                ("beta".into(), "2.0".into()),
                ("alpha".into(), "1.0".into()),
            ],
            ..base.clone()
        };
        assert_eq!(base.fingerprint(), reordered.fingerprint());

        let bumped = ReproductionKey {
            generator_version: GeneratorVersion(2),
            ..base.clone()
        };
        assert_ne!(base.fingerprint(), bumped.fingerprint());
    }
}
