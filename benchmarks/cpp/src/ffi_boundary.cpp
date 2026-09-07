// The far side of an FFI boundary.
//
// Deliberately in its own translation unit and compiled without link-time
// optimisation, so the call in bench.cpp is a real, opaque call rather than
// something the optimizer folds away. That is the mechanical floor of any
// in-process foreign-function boundary: the call, the argument marshalling and
// the lost inlining.

#include <cstddef>
#include <cstdint>

extern "C" std::uint64_t nexora_ffi_mix(std::uint64_t value) {
  value ^= value >> 33;
  value *= 0xFF51AFD7ED558CCDULL;
  value ^= value >> 33;
  return value;
}

extern "C" std::uint64_t nexora_ffi_digest(const std::uint8_t* data, std::size_t len) {
  std::uint64_t hash = 0xcbf29ce484222325ULL;
  for (std::size_t i = 0; i < len; ++i) {
    hash ^= static_cast<std::uint64_t>(data[i]);
    hash *= 0x00000100000001b3ULL;
  }
  return hash;
}

// Does nothing but cross the boundary and come back. Used to separate the cost
// of the crossing itself from the cost of whatever is on the far side.
extern "C" std::uint64_t nexora_ffi_noop(std::uint64_t value) { return value + 1; }
