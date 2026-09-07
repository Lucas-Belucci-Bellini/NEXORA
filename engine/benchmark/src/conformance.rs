//! Digests that prove a second stack computes the same thing.
//!
//! `NEXORA TECHNOLOGY BENCHMARK PLAN.md` rule 1 requires the **same logical
//! workload** across candidate stacks, and rule 5 forbids deciding from one.
//! Comparing two stacks therefore needs a step comparing *implementations*
//! before it compares *timings* — otherwise the honest reading of a fast
//! number is "the other one might just be doing less".
//!
//! `DEBT-0005` made that point within one language: the shift shortcut was
//! timed against `div_euclid` only after a test proved they agreed across
//! 20 000 coordinates. Across two languages the risk is far higher — Euclidean
//! division, rounding half away from zero, and unsigned-to-signed casts all
//! differ between Rust and C++ by default.
//!
//! Each function here folds a kernel's output into an FNV-1a 64 digest.
//! `benchmarks/cpp/src/conformance.cpp` emits the same names, and
//! `scripts/compare-stacks.sh` fails on any difference.

use nexora_foundation::error::Result;
use nexora_foundation::hashing::{crc32, fnv1a64, Fnv1a64};
use nexora_foundation::rng::{derive_stream_seed, positional_rng, Rng, SeedStream};
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape};
use nexora_foundation::time::CalendarConfig;
use nexora_physics::collision::sweep_axis;
use nexora_physics::math::{Aabb, Axis, Vec3};
use nexora_physics::voxel::{VoxelShape, VoxelSource};
use nexora_world::world::{World, WorldDescriptor};

/// The seed every digest that needs one uses.
const DIGEST_SEED: u64 = 0x0BEEF_0BEEF;

/// One named digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digest {
    /// Kernel name, matching the C++ reference.
    pub name: &'static str,
    /// The folded value.
    pub value: u64,
}

/// Every digest, in the order the C++ reference emits them.
///
/// # Errors
///
/// Returns an error when the benchmark world cannot be built.
pub fn digests() -> Result<Vec<Digest>> {
    let (chunk_cells, chunk_non_air, chunk_sections) = chunk_digest()?;
    Ok(vec![
        Digest {
            name: "rng.splitmix64",
            value: splitmix64_digest(),
        },
        Digest {
            name: "rng.next_below",
            value: next_below_digest(),
        },
        Digest {
            name: "rng.stream_seeds",
            value: stream_seed_digest(),
        },
        Digest {
            name: "rng.positional",
            value: positional_digest(),
        },
        Digest {
            name: "hash.fnv1a",
            value: fnv1a_digest(),
        },
        Digest {
            name: "hash.crc32",
            value: crc32_digest(),
        },
        Digest {
            name: "spatial.section_of",
            value: section_of_digest(),
        },
        Digest {
            name: "world.surface_height",
            value: surface_height_digest()?,
        },
        Digest {
            name: "world.chunk_cells",
            value: chunk_cells,
        },
        Digest {
            name: "world.chunk_non_air",
            value: chunk_non_air,
        },
        Digest {
            name: "world.chunk_sections",
            value: chunk_sections,
        },
        Digest {
            name: "physics.sweep",
            value: sweep_digest(),
        },
    ])
}

fn splitmix64_digest() -> u64 {
    let mut rng = Rng::from_seed(DIGEST_SEED);
    let mut hasher = Fnv1a64::new();
    for _ in 0..64 {
        hasher.write_u64(rng.next_u64());
    }
    hasher.finish()
}

fn next_below_digest() -> u64 {
    let mut rng = Rng::from_seed(12_345);
    let mut hasher = Fnv1a64::new();
    for _ in 0..2_000 {
        hasher.write_u64(rng.next_below(25));
        hasher.write_u64(rng.next_below(1));
        hasher.write_u64(rng.next_below(0));
        hasher.write_u64(rng.next_below(u64::MAX));
    }
    hasher.finish()
}

fn stream_seed_digest() -> u64 {
    let mut hasher = Fnv1a64::new();
    let streams = [
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
    ];
    for seed in [0u64, 1, 0xDEAD_BEEF, u64::MAX] {
        for stream in streams {
            hasher.write_u64(derive_stream_seed(seed, stream));
        }
    }
    hasher.finish()
}

fn positional_digest() -> u64 {
    let mut hasher = Fnv1a64::new();
    let mut driver = Rng::from_seed(0xA5A5_A5A5);
    for _ in 0..512 {
        let x = (driver.next_u64() as i64) % 100_000;
        let y = (driver.next_u64() as i64) % 1_000;
        let z = (driver.next_u64() as i64) % 100_000;
        hasher.write_u64(positional_rng(0x5EED, SeedStream::Terrain, x, y, z).next_u64());
    }
    hasher.finish()
}

fn fnv1a_digest() -> u64 {
    let mut hasher = Fnv1a64::new();
    for input in [
        "",
        "a",
        "foobar",
        "nexora:block/stone",
        "the quick brown fox",
    ] {
        hasher.write_u64(fnv1a64(input.as_bytes()));
    }
    let mut prefixed = Fnv1a64::new();
    prefixed.write_str("ab");
    prefixed.write_str("c");
    hasher.write_u64(prefixed.finish());
    hasher.finish()
}

fn crc32_digest() -> u64 {
    let mut hasher = Fnv1a64::new();
    for input in ["", "123456789", "NEXORA", "nexora:save/world_header"] {
        hasher.write_u64(u64::from(crc32(input.as_bytes())));
    }
    let mut payload = vec![0u8; 4_096];
    let mut driver = Rng::from_seed(7);
    for byte in &mut payload {
        *byte = driver.next_u64() as u8;
    }
    hasher.write_u64(u64::from(crc32(&payload)));
    hasher.finish()
}

fn section_of_digest() -> u64 {
    let shape = ChunkShape::cubic_default();
    let mut hasher = Fnv1a64::new();
    let mut driver = Rng::from_seed(0x2545_F491_4F6C_DD1D);
    for _ in 0..20_000 {
        let x = (driver.next_u64() as i64) % 4_000_000;
        let y = (driver.next_u64() as i64) % 4_000_000;
        let z = (driver.next_u64() as i64) % 4_000_000;
        let coord = shape.section_of(BlockPos::new(x, y, z));
        hasher.write_u64(coord.x as u64);
        hasher.write_u64(coord.y as u64);
        hasher.write_u64(coord.z as u64);
    }
    for value in [-33i64, -32, -31, -1, 0, 1, 31, 32, 33] {
        let coord = shape.section_of(BlockPos::new(value, value, value));
        hasher.write_u64(coord.x as u64);
    }
    hasher.finish()
}

fn digest_world() -> Result<World> {
    let descriptor = WorldDescriptor::new("conformance", DIGEST_SEED)?;
    let mut world = World::create(descriptor, CalendarConfig::earthlike())?;
    world.bring_online()?;
    Ok(world)
}

fn surface_height_digest() -> Result<u64> {
    let world = digest_world()?;
    let mut hasher = Fnv1a64::new();
    for z in -32..32 {
        for x in -32..32 {
            hasher.write_u64(world.surface_height(x, z) as u64);
        }
    }
    Ok(hasher.finish())
}

fn chunk_digest() -> Result<(u64, u64, u64)> {
    let world = digest_world()?;
    let chunk = world.generate_chunk(ChunkCoord::new(0, 0))?;
    let shape = world.descriptor().shape;
    let volume = shape.volume();

    let mut hasher = Fnv1a64::new();
    let mut non_air = 0u64;
    let indices = chunk.section_indices();
    for section_y in &indices {
        hasher.write_u64(*section_y as u64);
        let Some(section) = chunk.section(*section_y) else {
            continue;
        };
        non_air += u64::from(section.non_air_count());
        for index in 0..volume {
            let local = shape.local_from_index(index)?;
            hasher.write_u64(u64::from(section.get(local)?.0));
        }
    }
    Ok((hasher.finish(), non_air, indices.len() as u64))
}

/// A floor with a wall at `x == 4`, three cells tall. Mirrors the C++ fixture.
/// A step of solid cells at `x == 4`, over solid ground below `y == 0`.
///
/// `pub` so the timing suites can sweep against the *same* fixture the
/// conformance digest verifies, and so the C++ reference — which mirrors this
/// predicate exactly — is timed against identical geometry. A benchmark pair
/// running on two different worlds compares two different problems.
pub struct SweepFixture;

impl VoxelSource for SweepFixture {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        if position.y < 0 || (position.x == 4 && position.y < 3) {
            VoxelShape::SOLID
        } else {
            VoxelShape::Empty
        }
    }
}

/// A floor whose top face is `y = 1`, for the standing case below.
struct RaisedFloor;

impl VoxelSource for RaisedFloor {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        if position.y < 1 {
            VoxelShape::SOLID
        } else {
            VoxelShape::Empty
        }
    }
}

fn sweep_digest() -> u64 {
    let mut hasher = Fnv1a64::new();
    let mut driver = Rng::from_seed(0x9E37_79B9_7F4A_7C15);
    // The arithmetic is written out rather than fused: `mul_add` would round
    // once where the C++ expression rounds twice, and the two stacks would stop
    // agreeing on the last bit.
    let component = |driver: &mut Rng| ((driver.next_u64() & 0xFFFF) as f64 / 65_535.0) * 8.0 - 4.0;
    for _ in 0..4_000 {
        let cx = component(&mut driver);
        let cy = component(&mut driver);
        let cz = component(&mut driver);
        let aabb = Aabb {
            min: Vec3::new(cx - 0.3, cy - 0.9, cz - 0.3),
            max: Vec3::new(cx + 0.3, cy + 0.9, cz + 0.3),
        };
        for axis in Axis::ALL {
            let delta = component(&mut driver);
            let sweep = sweep_axis(&SweepFixture, aabb, axis, delta);
            hasher.write_u64(u64::from(sweep.is_blocked()));
            hasher.write_u64(sweep.allowed.to_bits());
        }
    }

    // The exact case that produced the fall-through-the-floor defect.
    let feet = 1.9 - 0.9;
    let standing = Aabb {
        min: Vec3::new(0.2, feet, 0.2),
        max: Vec3::new(0.8, feet + 1.8, 0.8),
    };
    let resting = sweep_axis(&RaisedFloor, standing, Axis::Y, -0.0027);
    hasher.write_u64(u64::from(resting.is_blocked()));
    hasher.write_u64(resting.allowed.to_bits());
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_digest_has_a_distinct_name() {
        let digests = digests().expect("digests");
        let mut names: Vec<_> = digests.iter().map(|entry| entry.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(names.len(), before, "two digests share a name");
        assert!(before >= 12);
    }

    #[test]
    fn digests_are_stable_across_calls() {
        // If a digest depended on anything but its inputs, the comparison
        // against another stack would fail for a reason that has nothing to do
        // with that stack.
        let first = digests().expect("digests");
        for _ in 0..3 {
            assert_eq!(digests().expect("digests"), first);
        }
    }

    #[test]
    fn the_chunk_digest_describes_a_real_chunk() {
        let digests = digests().expect("digests");
        let find = |name: &str| {
            digests
                .iter()
                .find(|entry| entry.name == name)
                .expect("present")
                .value
        };
        assert!(
            find("world.chunk_sections") > 1,
            "no sections were generated"
        );
        assert!(
            find("world.chunk_non_air") > 1_000_000,
            "the chunk is empty"
        );
    }
}
