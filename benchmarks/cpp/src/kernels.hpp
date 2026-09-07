// NEXORA kernel reference in C++ — the second stack the language gate needs.
//
// `NEXORA TECHNOLOGY BENCHMARK PLAN.md` rule 5 forbids deciding from a single
// stack, and rule 1 requires the SAME LOGICAL WORKLOAD. This header is that
// workload: the same algorithms, the same constants, the same representations
// as the Rust engine, written the way a C++ engine would write them.
//
// Every value here is pinned by the conformance binary, which digests the
// output of each kernel and is compared against the Rust engine's digest of the
// same kernel. Timing an implementation that computes something *else* is the
// mistake DEBT-0005 was written to avoid, and it is much easier to make across
// two languages than within one.
//
// Zero dependencies beyond the C++ standard library, mirroring ADR-0002.
#pragma once

#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

namespace nexora {

// ---------------------------------------------------------------- hashing
// Mirrors engine/foundation/src/hashing.rs.

inline constexpr std::uint64_t kFnvOffsetBasis64 = 0xcbf29ce484222325ULL;
inline constexpr std::uint64_t kFnvPrime64 = 0x00000100000001b3ULL;

class Fnv1a64 {
 public:
  Fnv1a64() = default;

  void write(const void* data, std::size_t len) {
    const auto* bytes = static_cast<const std::uint8_t*>(data);
    for (std::size_t i = 0; i < len; ++i) {
      state_ ^= static_cast<std::uint64_t>(bytes[i]);
      state_ *= kFnvPrime64;
    }
  }

  // Little-endian, matching Rust's `to_le_bytes`.
  void write_u64(std::uint64_t value) {
    std::uint8_t buffer[8];
    for (int i = 0; i < 8; ++i) {
      buffer[i] = static_cast<std::uint8_t>(value >> (8 * i));
    }
    write(buffer, sizeof buffer);
  }

  void write_u32(std::uint32_t value) { write_u64(static_cast<std::uint64_t>(value)); }
  void write_i64(std::int64_t value) { write_u64(static_cast<std::uint64_t>(value)); }

  // Length-prefixed, so "ab"+"c" cannot collide with "a"+"bc".
  void write_str(const std::string& value) {
    write_u64(value.size());
    write(value.data(), value.size());
  }

  [[nodiscard]] std::uint64_t finish() const { return state_; }

 private:
  std::uint64_t state_ = kFnvOffsetBasis64;
};

inline std::uint64_t fnv1a64(const void* data, std::size_t len) {
  Fnv1a64 hasher;
  hasher.write(data, len);
  return hasher.finish();
}

// CRC-32 (IEEE 802.3, reflected, polynomial 0xEDB88320), computed the same
// branchless bit-at-a-time way the Rust engine computes it. A table-driven
// version would be faster and would no longer be the same workload.
inline std::uint32_t crc32(const void* data, std::size_t len) {
  const auto* bytes = static_cast<const std::uint8_t*>(data);
  std::uint32_t crc = 0xFFFFFFFFu;
  for (std::size_t i = 0; i < len; ++i) {
    crc ^= static_cast<std::uint32_t>(bytes[i]);
    for (int bit = 0; bit < 8; ++bit) {
      const std::uint32_t mask = ~(crc & 1u) + 1u;  // (crc & 1).wrapping_neg()
      crc = (crc >> 1) ^ (0xEDB88320u & mask);
    }
  }
  return ~crc;
}

// -------------------------------------------------------------------- rng
// Mirrors engine/foundation/src/rng.rs: SplitMix64.

inline constexpr std::uint64_t kGamma = 0x9E3779B97F4A7C15ULL;

class Rng {
 public:
  explicit Rng(std::uint64_t seed) : state_(seed) {}

  std::uint64_t next_u64() {
    state_ += kGamma;
    std::uint64_t z = state_;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
  }

  // Lemire's multiply-shift reduction with rejection, so the result is
  // unbiased. Returns 0 when `bound` is 0, as the Rust version does.
  std::uint64_t next_below(std::uint64_t bound) {
    if (bound == 0) {
      return 0;
    }
    const std::uint64_t threshold = (~bound + 1u) % bound;  // wrapping_neg
    for (;;) {
      const std::uint64_t draw = next_u64();
      const __uint128_t wide = static_cast<__uint128_t>(draw) * bound;
      const std::uint64_t high = static_cast<std::uint64_t>(wide >> 64);
      const std::uint64_t low = static_cast<std::uint64_t>(wide);
      if (low >= threshold) {
        return high;
      }
    }
  }

 private:
  std::uint64_t state_;
};

// The stream list of NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md. Only
// the label matters to the derivation, and changing one changes every world.
inline std::uint64_t derive_stream_seed(std::uint64_t world_seed, const std::string& stream) {
  Fnv1a64 hasher;
  hasher.write_u64(world_seed);
  hasher.write_str(stream);
  return hasher.finish();
}

// Position-derived, which is what makes generation order-independent.
inline Rng positional_rng(std::uint64_t world_seed, const std::string& stream, std::int64_t x,
                          std::int64_t y, std::int64_t z) {
  Fnv1a64 hasher;
  hasher.write_u64(derive_stream_seed(world_seed, stream));
  hasher.write_i64(x);
  hasher.write_i64(y);
  hasher.write_i64(z);
  return Rng(hasher.finish());
}

// ---------------------------------------------------------------- spatial
// Mirrors engine/foundation/src/spatial.rs. C++ division truncates towards
// zero, so Euclidean division has to be written out: truncating here would
// mirror the world across the origin, which is one of the defects the Rust
// engine's tests caught.

inline std::int64_t div_euclid(std::int64_t lhs, std::int64_t rhs) {
  std::int64_t quotient = lhs / rhs;
  if (lhs % rhs < 0) {
    quotient += (rhs > 0) ? -1 : 1;
  }
  return quotient;
}

inline std::int64_t rem_euclid(std::int64_t lhs, std::int64_t rhs) {
  std::int64_t remainder = lhs % rhs;
  if (remainder < 0) {
    remainder += (rhs < 0) ? -rhs : rhs;
  }
  return remainder;
}

struct ChunkShape {
  std::uint32_t size_x = 32;
  std::uint32_t size_y = 32;
  std::uint32_t size_z = 32;

  [[nodiscard]] std::size_t volume() const {
    return static_cast<std::size_t>(size_x) * size_y * size_z;
  }

  [[nodiscard]] std::size_t index_of(std::uint32_t x, std::uint32_t y, std::uint32_t z) const {
    return (static_cast<std::size_t>(y) * size_z + z) * size_x + x;
  }
};

struct SectionCoord {
  std::int64_t x = 0;
  std::int64_t y = 0;
  std::int64_t z = 0;
};

inline SectionCoord section_of(const ChunkShape& shape, std::int64_t x, std::int64_t y,
                               std::int64_t z) {
  return SectionCoord{div_euclid(x, shape.size_x), div_euclid(y, shape.size_y),
                      div_euclid(z, shape.size_z)};
}

// ------------------------------------------------------------------ voxel
// Mirrors engine/world/src/voxel.rs: palette-compressed sections.

using BlockStateId = std::uint32_t;
inline constexpr BlockStateId kAir = 0;
inline constexpr std::uint8_t kMaxPaletteBits = 12;
inline constexpr std::size_t kMaxPaletteLen = std::size_t{1} << kMaxPaletteBits;

inline std::uint8_t bits_for(std::size_t len) {
  std::uint8_t bits = 1;
  while ((std::size_t{1} << bits) < len && bits < kMaxPaletteBits) {
    ++bits;
  }
  return bits;
}

inline std::size_t word_count(std::size_t volume, std::uint8_t bits) {
  const std::size_t per_word = 64 / bits;
  return (volume + per_word - 1) / per_word;
}

inline std::uint64_t read_packed(const std::vector<std::uint64_t>& words, std::uint8_t bits,
                                 std::size_t index) {
  const std::size_t per_word = 64 / bits;
  const std::size_t word = index / per_word;
  const std::size_t offset = (index % per_word) * bits;
  const std::uint64_t mask = (std::uint64_t{1} << bits) - 1;
  return word < words.size() ? (words[word] >> offset) & mask : 0;
}

inline void write_packed(std::vector<std::uint64_t>& words, std::uint8_t bits, std::size_t index,
                         std::uint64_t value) {
  const std::size_t per_word = 64 / bits;
  const std::size_t word = index / per_word;
  const std::size_t offset = (index % per_word) * bits;
  const std::uint64_t mask = (std::uint64_t{1} << bits) - 1;
  if (word < words.size()) {
    words[word] = (words[word] & ~(mask << offset)) | ((value & mask) << offset);
  }
}

enum class StorageKind { Uniform, Paletted, Direct };

class Section {
 public:
  explicit Section(ChunkShape shape, BlockStateId state = kAir)
      : shape_(shape), kind_(StorageKind::Uniform), uniform_(state) {
    non_air_ = state == kAir ? 0 : static_cast<std::uint32_t>(shape.volume());
  }

  [[nodiscard]] StorageKind kind() const { return kind_; }
  [[nodiscard]] std::uint32_t non_air() const { return non_air_; }
  [[nodiscard]] const ChunkShape& shape() const { return shape_; }

  [[nodiscard]] BlockStateId get_by_index(std::size_t index) const {
    switch (kind_) {
      case StorageKind::Uniform:
        return uniform_;
      case StorageKind::Paletted: {
        const auto palette_index = static_cast<std::size_t>(read_packed(words_, bits_, index));
        return palette_index < palette_.size() ? palette_[palette_index] : kAir;
      }
      case StorageKind::Direct:
        return index < cells_.size() ? cells_[index] : kAir;
    }
    return kAir;
  }

  [[nodiscard]] BlockStateId get(std::uint32_t x, std::uint32_t y, std::uint32_t z) const {
    return get_by_index(shape_.index_of(x, y, z));
  }

  BlockStateId set(std::uint32_t x, std::uint32_t y, std::uint32_t z, BlockStateId state) {
    return set_by_index(shape_.index_of(x, y, z), state);
  }

  BlockStateId set_by_index(std::size_t index, BlockStateId state) {
    const BlockStateId previous = get_by_index(index);
    if (previous == state) {
      return previous;
    }

    switch (kind_) {
      case StorageKind::Uniform: {
        // First divergence: grow into a two-entry palette.
        const BlockStateId existing = uniform_;
        const std::size_t volume = shape_.volume();
        bits_ = bits_for(2);
        words_.assign(word_count(volume, bits_), 0);
        write_packed(words_, bits_, index, 1);
        palette_ = {existing, state};
        kind_ = StorageKind::Paletted;
        break;
      }
      case StorageKind::Paletted: {
        std::size_t palette_index = palette_.size();
        for (std::size_t i = 0; i < palette_.size(); ++i) {
          if (palette_[i] == state) {
            palette_index = i;
            break;
          }
        }
        if (palette_index == palette_.size()) {
          if (palette_.size() >= kMaxPaletteLen) {
            materialize_into_direct();
            cells_[index] = state;
            apply_air_delta(previous, state);
            return previous;
          }
          palette_.push_back(state);
          const std::uint8_t needed = bits_for(palette_.size());
          if (needed > bits_) {
            repack(needed);
          }
          palette_index = palette_.size() - 1;
        }
        write_packed(words_, bits_, index, palette_index);
        break;
      }
      case StorageKind::Direct:
        cells_[index] = state;
        break;
    }

    apply_air_delta(previous, state);
    return previous;
  }

  // Rebuild the palette from what is actually referenced: repeated edits leave
  // entries no cell points at any more.
  void compact() {
    const std::size_t volume = shape_.volume();
    if (volume == 0) {
      return;
    }
    const BlockStateId first = get_by_index(0);
    bool all_same = true;
    for (std::size_t index = 1; index < volume; ++index) {
      if (get_by_index(index) != first) {
        all_same = false;
        break;
      }
    }
    if (all_same) {
      kind_ = StorageKind::Uniform;
      uniform_ = first;
      palette_.clear();
      words_.clear();
      cells_.clear();
      non_air_ = first == kAir ? 0 : static_cast<std::uint32_t>(volume);
      return;
    }

    std::vector<BlockStateId> cells = materialize();
    std::vector<BlockStateId> palette;
    for (const BlockStateId cell : cells) {
      bool present = false;
      for (const BlockStateId entry : palette) {
        if (entry == cell) {
          present = true;
          break;
        }
      }
      if (!present) {
        palette.push_back(cell);
        if (palette.size() > kMaxPaletteLen) {
          kind_ = StorageKind::Direct;
          cells_ = std::move(cells);
          palette_.clear();
          words_.clear();
          return;
        }
      }
    }

    const std::uint8_t bits = bits_for(palette.size());
    std::vector<std::uint64_t> words(word_count(volume, bits), 0);
    for (std::size_t index = 0; index < cells.size(); ++index) {
      std::uint64_t palette_index = 0;
      for (std::size_t i = 0; i < palette.size(); ++i) {
        if (palette[i] == cells[index]) {
          palette_index = i;
          break;
        }
      }
      write_packed(words, bits, index, palette_index);
    }
    kind_ = StorageKind::Paletted;
    palette_ = std::move(palette);
    bits_ = bits;
    words_ = std::move(words);
    cells_.clear();
  }

  [[nodiscard]] std::vector<BlockStateId> materialize() const {
    std::vector<BlockStateId> cells(shape_.volume());
    for (std::size_t index = 0; index < cells.size(); ++index) {
      cells[index] = get_by_index(index);
    }
    return cells;
  }

  [[nodiscard]] std::size_t storage_bytes() const {
    switch (kind_) {
      case StorageKind::Uniform:
        return sizeof(BlockStateId);
      case StorageKind::Paletted:
        return palette_.size() * sizeof(BlockStateId) + words_.size() * sizeof(std::uint64_t);
      case StorageKind::Direct:
        return cells_.size() * sizeof(BlockStateId);
    }
    return 0;
  }

 private:
  void apply_air_delta(BlockStateId previous, BlockStateId next) {
    if (previous == kAir && next != kAir) {
      ++non_air_;
    } else if (previous != kAir && next == kAir) {
      --non_air_;
    }
  }

  void repack(std::uint8_t needed) {
    const std::size_t volume = shape_.volume();
    std::vector<std::uint64_t> fresh(word_count(volume, needed), 0);
    for (std::size_t index = 0; index < volume; ++index) {
      write_packed(fresh, needed, index, read_packed(words_, bits_, index));
    }
    words_ = std::move(fresh);
    bits_ = needed;
  }

  void materialize_into_direct() {
    cells_ = materialize();
    kind_ = StorageKind::Direct;
    palette_.clear();
    words_.clear();
  }

  ChunkShape shape_;
  StorageKind kind_;
  BlockStateId uniform_ = kAir;
  std::vector<BlockStateId> palette_;
  std::uint8_t bits_ = 1;
  std::vector<std::uint64_t> words_;
  std::vector<BlockStateId> cells_;
  std::uint32_t non_air_ = 0;
};

// ------------------------------------------------------------- world gen
// Mirrors engine/world/src/world.rs.

inline constexpr std::int64_t kTerrainBaseHeight = 64;
inline constexpr std::int64_t kTerrainAmplitude = 12;
inline constexpr std::int64_t kSoilDepth = 4;
inline constexpr std::int64_t kWorldMinY = -1920;

// Block state ids as the Rust world registers them: air 0, stone 1, dirt 2,
// grass 3, in that registration order.
inline constexpr BlockStateId kStone = 1;
inline constexpr BlockStateId kDirt = 2;
inline constexpr BlockStateId kGrass = 3;

inline std::int64_t surface_height(std::uint64_t seed, std::int64_t x, std::int64_t z) {
  Rng rng = positional_rng(seed, "terrain", x, 0, z);
  const auto span = static_cast<std::uint64_t>(kTerrainAmplitude * 2 + 1);
  return kTerrainBaseHeight + static_cast<std::int64_t>(rng.next_below(span)) - kTerrainAmplitude;
}

struct GeneratedChunk {
  std::vector<std::pair<std::int64_t, Section>> sections;
  std::uint64_t non_air = 0;
  std::size_t storage_bytes = 0;
};

// One chunk column, exactly as `World::generate_chunk` builds it.
inline GeneratedChunk generate_chunk(std::uint64_t seed, const ChunkShape& shape,
                                     std::int64_t chunk_x, std::int64_t chunk_z) {
  const auto size_x = static_cast<std::int64_t>(shape.size_x);
  const auto size_y = static_cast<std::int64_t>(shape.size_y);
  const auto size_z = static_cast<std::int64_t>(shape.size_z);
  const std::int64_t origin_x = chunk_x * size_x;
  const std::int64_t origin_z = chunk_z * size_z;

  // Heightmap first, so the vertical span is known before any section exists.
  std::vector<std::int64_t> heights;
  heights.reserve(static_cast<std::size_t>(size_x * size_z));
  for (std::int64_t local_z = 0; local_z < size_z; ++local_z) {
    for (std::int64_t local_x = 0; local_x < size_x; ++local_x) {
      heights.push_back(surface_height(seed, origin_x + local_x, origin_z + local_z));
    }
  }
  std::int64_t lowest = kTerrainBaseHeight;
  std::int64_t highest = kTerrainBaseHeight;
  if (!heights.empty()) {
    lowest = heights[0];
    highest = heights[0];
    for (const std::int64_t height : heights) {
      lowest = height < lowest ? height : lowest;
      highest = height > highest ? height : highest;
    }
  }

  const std::int64_t first_section = div_euclid(kWorldMinY, size_y);
  const std::int64_t last_section = div_euclid(highest, size_y);

  GeneratedChunk chunk;
  for (std::int64_t section_y = first_section; section_y <= last_section; ++section_y) {
    const std::int64_t section_min = section_y * size_y;
    const std::int64_t section_max = section_min + size_y - 1;

    // Wholly below the deepest soil: one uniform stone section, free to store.
    if (section_max < lowest - kSoilDepth) {
      chunk.sections.emplace_back(section_y, Section(shape, kStone));
      continue;
    }
    // Wholly above the highest surface: absent, which is air.
    if (section_min > highest) {
      continue;
    }

    Section section(shape, kAir);
    for (std::int64_t local_z = 0; local_z < size_z; ++local_z) {
      for (std::int64_t local_x = 0; local_x < size_x; ++local_x) {
        const std::int64_t surface =
            heights[static_cast<std::size_t>(local_z * size_x + local_x)];
        for (std::int64_t local_y = 0; local_y < size_y; ++local_y) {
          const std::int64_t world_y = section_min + local_y;
          if (world_y > surface) {
            continue;
          }
          BlockStateId state = kStone;
          if (world_y == surface) {
            state = kGrass;
          } else if (world_y >= surface - kSoilDepth) {
            state = kDirt;
          }
          section.set(static_cast<std::uint32_t>(local_x), static_cast<std::uint32_t>(local_y),
                      static_cast<std::uint32_t>(local_z), state);
        }
      }
    }
    section.compact();
    chunk.sections.emplace_back(section_y, std::move(section));
  }

  for (const auto& entry : chunk.sections) {
    chunk.non_air += entry.second.non_air();
    chunk.storage_bytes += entry.second.storage_bytes();
  }
  return chunk;
}

// ---------------------------------------------------------------- physics
// Mirrors engine/physics/src/collision.rs: one axis of a swept box against a
// voxel grid, including the plane snapping that keeps a body from falling
// through the floor it is standing on.

inline constexpr double kContactEpsilon = 1.0e-9;
inline constexpr std::int64_t kMaxSweepLayers = 4096;

// `std::round` rounds half away from zero, which is what Rust's `f64::round`
// does. A cast-based approximation disagrees on negatives and on halfway
// cases, and the two stacks then stop computing the same thing.
inline double snap_to_plane(double value) {
  const double nearest = std::round(value);
  const double scaled = std::fabs(value) * 2.220446049250313e-16 * 8.0;  // f64::EPSILON
  const double tolerance = kContactEpsilon > scaled ? kContactEpsilon : scaled;
  return std::fabs(value - nearest) <= tolerance ? nearest : value;
}

struct Aabb {
  double min[3];
  double max[3];
};

// A solid-cell predicate, the C++ shape of the VoxelSource trait.
using SolidFn = bool (*)(std::int64_t x, std::int64_t y, std::int64_t z);

struct AxisSweep {
  double allowed = 0.0;
  bool blocked = false;
};

inline std::int64_t floor_i64(double value) {
  return static_cast<std::int64_t>(std::floor(value));
}

inline std::int64_t ceil_i64(double value) {
  return static_cast<std::int64_t>(std::ceil(value));
}

inline void voxel_span(const Aabb& box, int axis, std::int64_t& low, std::int64_t& high) {
  low = floor_i64(snap_to_plane(box.min[axis]));
  high = ceil_i64(snap_to_plane(box.max[axis])) - 1;
  if (high < low) {
    high = low;
  }
}

inline AxisSweep sweep_axis(SolidFn solid, const Aabb& box, int axis, double delta) {
  AxisSweep sweep;
  if (delta == 0.0 || delta != delta) {
    return sweep;
  }
  const bool positive = delta > 0.0;
  const double start = positive ? box.max[axis] : box.min[axis];
  const double end = start + delta;

  const int first_other = (axis + 1) % 3;
  const int second_other = (axis + 2) % 3;
  std::int64_t a0 = 0;
  std::int64_t a1 = 0;
  std::int64_t b0 = 0;
  std::int64_t b1 = 0;
  voxel_span(box, first_other, a0, a1);
  voxel_span(box, second_other, b0, b1);

  const double snapped = snap_to_plane(start);
  const std::int64_t nearest = positive ? ceil_i64(snapped) : floor_i64(snapped) - 1;
  const std::int64_t furthest = positive ? ceil_i64(end) - 1 : floor_i64(end);
  const std::int64_t layers = positive ? furthest - nearest : nearest - furthest;
  if (layers < 0) {
    sweep.allowed = delta;
    return sweep;
  }
  const std::int64_t capped = layers < kMaxSweepLayers - 1 ? layers : kMaxSweepLayers - 1;

  for (std::int64_t offset = 0; offset <= capped; ++offset) {
    const std::int64_t layer = positive ? nearest + offset : nearest - offset;
    for (std::int64_t a = a0; a <= a1; ++a) {
      for (std::int64_t b = b0; b <= b1; ++b) {
        std::int64_t cell[3];
        cell[axis] = layer;
        cell[first_other] = a;
        cell[second_other] = b;
        if (solid(cell[0], cell[1], cell[2])) {
          const double plane = positive ? static_cast<double>(layer)
                                        : static_cast<double>(layer + 1);
          sweep.allowed = plane - start;
          sweep.blocked = true;
          return sweep;
        }
      }
    }
  }
  sweep.allowed = delta;
  return sweep;
}

}  // namespace nexora
