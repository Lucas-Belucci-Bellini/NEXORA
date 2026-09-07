// Times the C++ kernels with the methodology the Rust harness uses.
//
// Same shape on purpose: warm-up, batched samples, **median and p95** rather
// than a mean (a mean hides the tail a budget has to survive), and relative
// sigma reported so noise is visible instead of assumed away. A comparison
// where the two sides measure differently is not a comparison.
//
// Run `scripts/compare-stacks.sh` rather than this binary alone: the digests
// have to match before any of these numbers mean anything.

#include <algorithm>
#include <chrono>
#include <cinttypes>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

#include "kernels.hpp"

namespace {

struct Budget {
  std::uint32_t warmup_iterations;
  std::uint32_t samples;
  std::uint32_t iterations_per_sample;

  static Budget standard(std::uint32_t iterations) { return {3, 25, iterations}; }
  static Budget coarse(std::uint32_t iterations) { return {1, 7, iterations}; }
  [[nodiscard]] Budget smoke() const { return {1, 3, iterations_per_sample}; }
};

struct Measurement {
  std::string name;
  std::string note;
  double median_ns;
  double p95_ns;
  double relative_sigma;
};

// The C++ equivalent of `std::hint::black_box`: stop the optimizer deleting
// work whose result nothing reads. Without it, half of these kernels vanish
// and the comparison rewards whichever compiler was better at deleting them.
template <typename T>
inline void consume(T&& value) {
  asm volatile("" : : "r,m"(value) : "memory");
}

// The value-returning form, the equivalent of passing a value *into*
// `std::hint::black_box`. Needed on the inlinable rungs of the FFI ladder: with
// nothing stopping it, the optimizer fuses successive iterations of an
// inlinable kernel and the loop then measures a fraction of the real work. A
// call is already a barrier, so the rungs that have one are unaffected -- but
// they get the same treatment anyway, or the ladder stops comparing like with
// like.
template <typename T>
[[nodiscard]] inline T opaque_value(T value) {
  asm volatile("" : "+r"(value));
  return value;
}

double percentile(std::vector<double> values, double fraction) {
  std::sort(values.begin(), values.end());
  if (values.empty()) {
    return 0.0;
  }
  const auto index = static_cast<std::size_t>(fraction * static_cast<double>(values.size() - 1));
  return values[index];
}

// Templated on the callable, not `std::function`. Rust's `measure` is generic
// over `F: FnMut()`, so the closure is monomorphised and the optimizer can see
// through it. A `std::function` here would add a type-erased indirect call to
// every iteration -- several nanoseconds on kernels that cost three, measured
// on one side of the comparison and not the other.
template <typename Operation>
Measurement measure(const std::string& name, const std::string& note, const Budget& budget,
                    Operation operation) {
  for (std::uint32_t i = 0; i < budget.warmup_iterations; ++i) {
    operation();
  }
  std::vector<double> per_iteration;
  per_iteration.reserve(budget.samples);
  for (std::uint32_t sample = 0; sample < budget.samples; ++sample) {
    const auto start = std::chrono::steady_clock::now();
    for (std::uint32_t i = 0; i < budget.iterations_per_sample; ++i) {
      operation();
    }
    const auto elapsed = std::chrono::steady_clock::now() - start;
    const auto nanos =
        static_cast<double>(std::chrono::duration_cast<std::chrono::nanoseconds>(elapsed).count());
    per_iteration.push_back(nanos / static_cast<double>(budget.iterations_per_sample));
  }

  double mean = 0.0;
  for (const double value : per_iteration) {
    mean += value;
  }
  mean /= static_cast<double>(per_iteration.size());
  double variance = 0.0;
  for (const double value : per_iteration) {
    variance += (value - mean) * (value - mean);
  }
  variance /= static_cast<double>(per_iteration.size());
  const double sigma = std::sqrt(variance);

  return Measurement{name, note, percentile(per_iteration, 0.5), percentile(per_iteration, 0.95),
                     mean > 0.0 ? sigma / mean * 100.0 : 0.0};
}

// ------------------------------------------------------------------- FFI
// Declared here and defined in ffi_boundary.cpp so the call cannot be inlined.
// That opaque call is the mechanical floor of any in-process FFI boundary.
extern "C" {
std::uint64_t nexora_ffi_mix(std::uint64_t value);
std::uint64_t nexora_ffi_digest(const std::uint8_t* data, std::size_t len);
}

// The same two operations, visible to the optimizer.
inline std::uint64_t local_mix(std::uint64_t value) {
  value ^= value >> 33;
  value *= 0xFF51AFD7ED558CCDULL;
  value ^= value >> 33;
  return value;
}

inline std::uint64_t local_digest(const std::uint8_t* data, std::size_t len) {
  return nexora::fnv1a64(data, len);
}

bool sweep_fixture(std::int64_t x, std::int64_t y, std::int64_t z) {
  (void)z;
  return y < 0 || (x == 4 && y < 3);
}

}  // namespace

int main(int argc, char** argv) {
  bool smoke = false;
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--smoke") == 0) {
      smoke = true;
    } else if (std::strcmp(argv[i], "--help") == 0) {
      std::printf(
          "usage: nexora-cpp-bench [--smoke]\n\n"
          "Times the C++ kernel reference. Run scripts/compare-stacks.sh instead:\n"
          "the conformance digests must match before these numbers mean anything.\n");
      return 0;
    }
  }
  const auto standard = [&](std::uint32_t n) {
    const Budget budget = Budget::standard(n);
    return smoke ? budget.smoke() : budget;
  };
  const auto coarse = [&](std::uint32_t n) {
    const Budget budget = Budget::coarse(n);
    return smoke ? budget.smoke() : budget;
  };

  std::vector<Measurement> results;
  const nexora::ChunkShape shape;

  // --- spatial -------------------------------------------------------------
  {
    std::int64_t counter = 0;
    results.push_back(measure(
        "spatial.section_of_division", "Block -> section using Euclidean division by a runtime extent",
        standard(1000), [&] {
          counter += 7;
          consume(nexora::section_of(shape, counter, -counter, counter * 3).x);
        }));
  }

  // --- voxel ---------------------------------------------------------------
  {
    nexora::Section paletted(shape, nexora::kStone);
    for (std::uint32_t i = 0; i < 64; ++i) {
      paletted.set_by_index(i * 97, static_cast<nexora::BlockStateId>(4 + i));
    }
    std::size_t cursor = 0;
    results.push_back(measure("voxel.get_paletted", "Read one cell from a palette-indexed section",
                              standard(1000), [&] {
                                cursor = (cursor + 313) % shape.volume();
                                consume(paletted.get_by_index(cursor));
                              }));

    nexora::Section uniform(shape, nexora::kStone);
    std::size_t uniform_cursor = 0;
    results.push_back(measure("voxel.get_uniform", "Read one cell from a uniform section",
                              standard(1000), [&] {
                                uniform_cursor = (uniform_cursor + 313) % shape.volume();
                                consume(uniform.get_by_index(uniform_cursor));
                              }));

    nexora::Section writable(shape, nexora::kStone);
    writable.set_by_index(0, nexora::kDirt);
    std::size_t write_cursor = 0;
    results.push_back(measure("voxel.set_existing_state",
                              "Write a cell whose state is already in the palette", standard(1000),
                              [&] {
                                write_cursor = (write_cursor + 313) % shape.volume();
                                consume(writable.set_by_index(write_cursor, nexora::kDirt));
                              }));
  }

  // --- world generation ----------------------------------------------------
  {
    std::int64_t column = 0;
    results.push_back(measure("worldgen.surface_height",
                              "One column's terrain height from the position-seeded stream",
                              standard(1000), [&] {
                                column += 1;
                                consume(nexora::surface_height(0x0BEEF0BEEFULL, column, -column));
                              }));

    nexora::ChunkShape small;
    small.size_x = 16;
    small.size_y = 16;
    small.size_z = 16;
    std::int64_t chunk16 = 0;
    results.push_back(measure("worldgen.chunk_16",
                              "Generate one chunk column, 16 cubed sections (the plan's size)",
                              coarse(1), [&] {
                                chunk16 += 1;
                                consume(
                                    nexora::generate_chunk(0x0BEEF0BEEFULL, small, chunk16, 0).non_air);
                              }));

    std::int64_t chunk32 = 0;
    results.push_back(measure("worldgen.chunk_32",
                              "Generate one chunk column, 32 cubed sections (engine default)",
                              coarse(1), [&] {
                                chunk32 += 1;
                                consume(
                                    nexora::generate_chunk(0x0BEEF0BEEFULL, shape, chunk32, 0).non_air);
                              }));
  }

  // --- physics -------------------------------------------------------------
  {
    const nexora::Aabb box{{0.2, 3.0, 0.2}, {0.8, 4.8, 0.8}};
    results.push_back(measure("physics.axis_sweep",
                              "One axis of a swept box against the voxel grid", standard(1000),
                              [&] {
                                const nexora::AxisSweep sweep =
                                    // Same single-scalar barrier as the Rust
                                    // side: without it the call is pure with
                                    // constant arguments and can be hoisted
                                    // out of the timing loop.
                                    nexora::sweep_axis(sweep_fixture, box, 1,
                                                       opaque_value(-0.16));
                                consume(sweep.allowed);
                              }));
  }

  // --- serialization -------------------------------------------------------
  {
    std::vector<std::uint8_t> payload(64 * 1024);
    nexora::Rng driver(11);
    for (auto& byte : payload) {
      byte = static_cast<std::uint8_t>(driver.next_u64());
    }
    results.push_back(measure("save.crc32_64kib",
                              "CRC-32 over 64 KiB, the per-section integrity check", standard(20),
                              [&] { consume(nexora::crc32(payload.data(), payload.size())); }));
  }

  // --- the FFI boundary ----------------------------------------------------
  // `NEXORA TECHNOLOGY BENCHMARK PLAN.md` lists FFI overhead as a metric, and
  // it is the one metric that could not be measured while there was only one
  // language in the build.
  {
    std::uint64_t value = 1;
    results.push_back(measure("ffi.opaque_call_scalar",
                              "One extern \"C\" call the compiler cannot inline, scalar argument",
                              standard(2000), [&] { value = nexora_ffi_mix(opaque_value(value)); }));
    consume(value);

    std::uint64_t local = 1;
    results.push_back(measure("ffi.inlinable_call_scalar",
                              "The same operation where the compiler can see through it",
                              standard(2000), [&] { local = local_mix(opaque_value(local)); }));
    consume(local);

    std::vector<std::uint8_t> buffer(4096);
    nexora::Rng driver(13);
    for (auto& byte : buffer) {
      byte = static_cast<std::uint8_t>(driver.next_u64());
    }
    results.push_back(measure("ffi.opaque_call_4kib",
                              "An extern \"C\" call that reads 4 KiB across the boundary",
                              standard(200),
                              [&] { consume(nexora_ffi_digest(buffer.data(), buffer.size())); }));
    results.push_back(measure("ffi.inlinable_call_4kib", "The same work with no boundary",
                              standard(200),
                              [&] { consume(local_digest(buffer.data(), buffer.size())); }));
  }

  std::printf("| measurement | median | p95 | rel. sigma | note |\n");
  std::printf("| --- | ---: | ---: | ---: | --- |\n");
  for (const Measurement& result : results) {
    std::printf("| `%s` | %.1f ns | %.1f ns | %.1f%% | %s |\n", result.name.c_str(),
                result.median_ns, result.p95_ns, result.relative_sigma, result.note.c_str());
  }
  return 0;
}
