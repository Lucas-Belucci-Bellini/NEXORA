// Proves the C++ reference computes the same thing as the Rust engine.
//
// Run before believing any timing. `NEXORA TECHNOLOGY BENCHMARK PLAN.md` rule 1
// requires the same logical workload; two stacks are far easier to drift apart
// than two functions in one language, and a comparison between a correct
// implementation and a subtly wrong one is worse than no comparison at all.
//
// Emits `name<TAB>0x…` lines. `nexora-benchmark --conformance` emits the same
// names, and scripts/compare-stacks.sh fails on any difference.

#include <cinttypes>
#include <cstdio>
#include <string>
#include <vector>

#include "kernels.hpp"

namespace {

using nexora::Fnv1a64;

void emit(const char* name, std::uint64_t digest) {
  std::printf("%s\t0x%016" PRIx64 "\n", name, digest);
}

std::uint64_t digest_splitmix64() {
  nexora::Rng rng(0x0BEEF0BEEFULL);
  Fnv1a64 hasher;
  for (int i = 0; i < 64; ++i) {
    hasher.write_u64(rng.next_u64());
  }
  return hasher.finish();
}

std::uint64_t digest_next_below() {
  nexora::Rng rng(12345);
  Fnv1a64 hasher;
  for (int i = 0; i < 2000; ++i) {
    hasher.write_u64(rng.next_below(25));
    hasher.write_u64(rng.next_below(1));
    hasher.write_u64(rng.next_below(0));
    hasher.write_u64(rng.next_below(0xFFFF'FFFF'FFFF'FFFFULL));
  }
  return hasher.finish();
}

std::uint64_t digest_stream_seeds() {
  Fnv1a64 hasher;
  const char* streams[] = {"terrain", "biome",        "cave", "vegetation", "weather",
                           "structure", "civilization", "ai",   "history",    "loot"};
  for (std::uint64_t seed : {0ULL, 1ULL, 0xDEADBEEFULL, 0xFFFFFFFFFFFFFFFFULL}) {
    for (const char* stream : streams) {
      hasher.write_u64(nexora::derive_stream_seed(seed, stream));
    }
  }
  return hasher.finish();
}

std::uint64_t digest_positional_rng() {
  Fnv1a64 hasher;
  nexora::Rng driver(0xA5A5A5A5ULL);
  for (int i = 0; i < 512; ++i) {
    const auto x = static_cast<std::int64_t>(driver.next_u64()) % 100000;
    const auto y = static_cast<std::int64_t>(driver.next_u64()) % 1000;
    const auto z = static_cast<std::int64_t>(driver.next_u64()) % 100000;
    hasher.write_u64(nexora::positional_rng(0x5EED, "terrain", x, y, z).next_u64());
  }
  return hasher.finish();
}

std::uint64_t digest_fnv1a() {
  Fnv1a64 hasher;
  const char* inputs[] = {"", "a", "foobar", "nexora:block/stone", "the quick brown fox"};
  for (const char* input : inputs) {
    hasher.write_u64(nexora::fnv1a64(input, std::char_traits<char>::length(input)));
  }
  // And the length-prefixed string path, which is what the seed derivation uses.
  Fnv1a64 prefixed;
  prefixed.write_str("ab");
  prefixed.write_str("c");
  hasher.write_u64(prefixed.finish());
  return hasher.finish();
}

std::uint64_t digest_crc32() {
  Fnv1a64 hasher;
  const char* inputs[] = {"", "123456789", "NEXORA", "nexora:save/world_header"};
  for (const char* input : inputs) {
    hasher.write_u32(nexora::crc32(input, std::char_traits<char>::length(input)));
  }
  // A longer buffer, so the loop is exercised rather than just the short cases.
  std::vector<std::uint8_t> payload(4096);
  nexora::Rng driver(7);
  for (auto& byte : payload) {
    byte = static_cast<std::uint8_t>(driver.next_u64());
  }
  hasher.write_u32(nexora::crc32(payload.data(), payload.size()));
  return hasher.finish();
}

std::uint64_t digest_section_of() {
  const nexora::ChunkShape shape;
  Fnv1a64 hasher;
  nexora::Rng driver(0x2545F4914F6CDD1DULL);
  for (int i = 0; i < 20000; ++i) {
    const auto spread = [&driver]() {
      return static_cast<std::int64_t>(driver.next_u64()) % 4000000;
    };
    const std::int64_t x = spread();
    const std::int64_t y = spread();
    const std::int64_t z = spread();
    const nexora::SectionCoord coord = nexora::section_of(shape, x, y, z);
    hasher.write_i64(coord.x);
    hasher.write_i64(coord.y);
    hasher.write_i64(coord.z);
  }
  // The negative boundaries explicitly: truncating division mirrors the world
  // across the origin, and this is where that would show.
  for (std::int64_t value : {-33, -32, -31, -1, 0, 1, 31, 32, 33}) {
    const nexora::SectionCoord coord = nexora::section_of(shape, value, value, value);
    hasher.write_i64(coord.x);
  }
  return hasher.finish();
}

std::uint64_t digest_surface_height() {
  Fnv1a64 hasher;
  for (std::int64_t z = -32; z < 32; ++z) {
    for (std::int64_t x = -32; x < 32; ++x) {
      hasher.write_i64(nexora::surface_height(0x0BEEF0BEEFULL, x, z));
    }
  }
  return hasher.finish();
}

std::uint64_t digest_chunk_cells(std::uint64_t* non_air_out, std::uint64_t* sections_out) {
  const nexora::ChunkShape shape;
  const nexora::GeneratedChunk chunk = nexora::generate_chunk(0x0BEEF0BEEFULL, shape, 0, 0);
  Fnv1a64 hasher;
  for (const auto& entry : chunk.sections) {
    hasher.write_i64(entry.first);
    const std::size_t volume = shape.volume();
    for (std::size_t index = 0; index < volume; ++index) {
      hasher.write_u32(entry.second.get_by_index(index));
    }
  }
  *non_air_out = chunk.non_air;
  *sections_out = chunk.sections.size();
  return hasher.finish();
}

// A floor with a wall at x == 4, three cells tall.
bool solid_fixture(std::int64_t x, std::int64_t y, std::int64_t z) {
  (void)z;
  return y < 0 || (x == 4 && y < 3);
}

std::uint64_t digest_sweep() {
  Fnv1a64 hasher;
  nexora::Rng driver(0x9E3779B97F4A7C15ULL);
  for (int i = 0; i < 4000; ++i) {
    const auto component = [&driver]() {
      return (static_cast<double>(driver.next_u64() & 0xFFFF) / 65535.0) * 8.0 - 4.0;
    };
    const double cx = component();
    const double cy = component();
    const double cz = component();
    nexora::Aabb box{{cx - 0.3, cy - 0.9, cz - 0.3}, {cx + 0.3, cy + 0.9, cz + 0.3}};
    for (int axis = 0; axis < 3; ++axis) {
      const double delta = component();
      const nexora::AxisSweep sweep = nexora::sweep_axis(solid_fixture, box, axis, delta);
      hasher.write_u64(static_cast<std::uint64_t>(sweep.blocked));
      std::uint64_t bits = 0;
      std::memcpy(&bits, &sweep.allowed, sizeof bits);
      hasher.write_u64(bits);
    }
  }
  // The exact case that produced the fall-through-the-floor defect: feet at
  // 1.9 - 0.9, which is 0.9999999999999999 rather than 1.0.
  const double feet = 1.9 - 0.9;
  nexora::Aabb standing{{0.2, feet, 0.2}, {0.8, feet + 1.8, 0.8}};
  const nexora::AxisSweep resting = nexora::sweep_axis(
      [](std::int64_t x, std::int64_t y, std::int64_t z) {
        (void)x;
        (void)z;
        return y < 1;
      },
      standing, 1, -0.0027);
  hasher.write_u64(static_cast<std::uint64_t>(resting.blocked));
  std::uint64_t bits = 0;
  std::memcpy(&bits, &resting.allowed, sizeof bits);
  hasher.write_u64(bits);
  return hasher.finish();
}

}  // namespace

int main() {
  emit("rng.splitmix64", digest_splitmix64());
  emit("rng.next_below", digest_next_below());
  emit("rng.stream_seeds", digest_stream_seeds());
  emit("rng.positional", digest_positional_rng());
  emit("hash.fnv1a", digest_fnv1a());
  emit("hash.crc32", digest_crc32());
  emit("spatial.section_of", digest_section_of());
  emit("world.surface_height", digest_surface_height());
  std::uint64_t non_air = 0;
  std::uint64_t sections = 0;
  const std::uint64_t cells = digest_chunk_cells(&non_air, &sections);
  emit("world.chunk_cells", cells);
  emit("world.chunk_non_air", non_air);
  emit("world.chunk_sections", sections);
  emit("physics.sweep", digest_sweep());
  return 0;
}
