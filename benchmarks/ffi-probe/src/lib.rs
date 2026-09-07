//! What one crossing of a language boundary costs.
//!
//! `NEXORA TECHNOLOGY BENCHMARK PLAN.md` lists FFI overhead as a metric of the
//! language-selection gate, and `ADR-0004` leaves open the possibility of a
//! polyglot engine. Neither can be settled by argument: the cost of a boundary
//! is a number, and until this crate existed the workspace was single-language
//! and the number was unmeasurable.
//!
//! # Three rungs, because one number would not say anything
//!
//! Comparing a cross-language call against an *inlined* local one conflates two
//! separate costs and blames both on the boundary. So each operation is offered
//! three ways:
//!
//! | rung | what it is | what it costs |
//! | --- | --- | --- |
//! | [`inlined`] | ordinary Rust the optimizer can see through | the work itself |
//! | [`opaque`] | `#[inline(never)]` Rust, same crate | the work, plus a call |
//! | [`crossing`] | `extern "C"` into a C++ translation unit | the work, plus a call, plus the boundary |
//!
//! The interesting quantities are the *differences*: `opaque - inlined` is what
//! not inlining costs, and `crossing - opaque` is what the language boundary
//! costs on top of that. Reporting only the third would attribute the first two
//! to a decision that did not cause them.
//!
//! # What this does not measure
//!
//! A C ABI call with integer and pointer arguments is the *cheapest* boundary
//! there is. It does not include marshalling a struct, converting a string,
//! copying a buffer to satisfy an ownership rule, catching a panic before it
//! unwinds into foreign frames, or validating that a pointer from the other
//! side is what it claims to be. A real polyglot engine pays those too. This
//! crate measures the floor, and the floor is a lower bound, not an estimate.
//!
//! # Availability
//!
//! [`CROSS_LANGUAGE_LINKED`] is `false` unless the crate was built with the
//! `cpp` feature, and [`crossing`] then returns [`None`]. The default build of
//! this workspace needs no C++ compiler.

/// Whether the C++ translation unit is linked into this build.
///
/// When `false`, everything in [`crossing`] returns [`None`] and the benchmark
/// reports FFI overhead as unmeasured rather than substituting a Rust number.
pub const CROSS_LANGUAGE_LINKED: bool = cfg!(cpp_boundary_linked);

/// The FNV-1a 64-bit offset basis, from the published specification.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The FNV-1a 64-bit prime, from the published specification.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The 64-bit finaliser constant of the MurmurHash3 mixer.
const MIX_MULTIPLIER: u64 = 0xFF51_AFD7_ED55_8CCD;

/// Rung 1: the operations as ordinary Rust, fully visible to the optimizer.
pub mod inlined {
    use super::{FNV_OFFSET_BASIS, FNV_PRIME, MIX_MULTIPLIER};

    /// Mix one 64-bit word. A short, strictly serial dependency chain.
    #[inline(always)]
    #[must_use]
    pub fn mix(mut value: u64) -> u64 {
        value ^= value >> 33;
        value = value.wrapping_mul(MIX_MULTIPLIER);
        value ^= value >> 33;
        value
    }

    /// FNV-1a over a byte slice. One byte per iteration, serial by construction.
    #[inline(always)]
    #[must_use]
    pub fn digest(data: &[u8]) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for &byte in data {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /// Return the argument, plus one. The nearest thing to doing nothing that
    /// still cannot be folded away.
    #[inline(always)]
    #[must_use]
    pub fn noop(value: u64) -> u64 {
        value.wrapping_add(1)
    }
}

/// Rung 2: the same operations behind a call the optimizer will not inline.
///
/// Same language, same crate, same ABI — only the inlining is gone. This is the
/// control that stops [`crossing`] being credited with the cost of a call.
pub mod opaque {
    /// See [`inlined::mix`](super::inlined::mix).
    #[inline(never)]
    #[must_use]
    pub fn mix(value: u64) -> u64 {
        super::inlined::mix(value)
    }

    /// See [`inlined::digest`](super::inlined::digest).
    #[inline(never)]
    #[must_use]
    pub fn digest(data: &[u8]) -> u64 {
        super::inlined::digest(data)
    }

    /// See [`inlined::noop`](super::inlined::noop).
    #[inline(never)]
    #[must_use]
    pub fn noop(value: u64) -> u64 {
        super::inlined::noop(value)
    }
}

/// Rung 3: the same operations, executed by code a C++ compiler produced.
///
/// Every function returns [`None`] when the crate was built without the `cpp`
/// feature. Callers must handle that rather than falling back to [`opaque`] —
/// a fallback here is how a single-language number ends up printed under a
/// cross-language heading.
pub mod crossing {
    // Declarations of the far side of the boundary. (A plain comment, not a doc
    // comment: rustdoc does not document extern blocks.)
    //
    // The one `unsafe` in this repository, and the reason this crate sits
    // outside `engine/`. Each of these is defined in
    // `benchmarks/cpp/src/ffi_boundary.cpp`, compiled by `build.rs` from that
    // exact file, and takes only integers and a pointer-plus-length that the
    // safe wrappers below derive from a live slice. See ADR-0009.
    #[cfg(cpp_boundary_linked)]
    #[allow(unsafe_code)]
    extern "C" {
        fn nexora_ffi_mix(value: u64) -> u64;
        fn nexora_ffi_digest(data: *const u8, len: usize) -> u64;
        fn nexora_ffi_noop(value: u64) -> u64;
    }

    /// Mix one 64-bit word on the far side of the boundary.
    #[must_use]
    pub fn mix(value: u64) -> Option<u64> {
        #[cfg(cpp_boundary_linked)]
        {
            // SAFETY: `nexora_ffi_mix` takes and returns a `u64` by value and
            // touches no memory. There is no pointer to be invalid and no
            // lifetime to outlive.
            #[allow(unsafe_code)]
            Some(unsafe { nexora_ffi_mix(value) })
        }
        #[cfg(not(cpp_boundary_linked))]
        {
            let _ = value;
            None
        }
    }

    /// Hash a byte slice on the far side of the boundary.
    #[must_use]
    pub fn digest(data: &[u8]) -> Option<u64> {
        #[cfg(cpp_boundary_linked)]
        {
            // SAFETY: the pointer and length come from a live shared borrow, so
            // the range is valid and immutable for the whole call, which is
            // synchronous. The callee only reads it -- the parameter is
            // `const std::uint8_t*` and the body is a read-only loop.
            #[allow(unsafe_code)]
            Some(unsafe { nexora_ffi_digest(data.as_ptr(), data.len()) })
        }
        #[cfg(not(cpp_boundary_linked))]
        {
            let _ = data;
            None
        }
    }

    /// Cross the boundary and come back, doing as little as possible in between.
    #[must_use]
    pub fn noop(value: u64) -> Option<u64> {
        #[cfg(cpp_boundary_linked)]
        {
            // SAFETY: as `mix` -- a `u64` in, a `u64` out, no memory touched.
            #[allow(unsafe_code)]
            Some(unsafe { nexora_ffi_noop(value) })
        }
        #[cfg(not(cpp_boundary_linked))]
        {
            let _ = value;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Anchors the Rust side to the FNV-1a 64 specification rather than to
    /// whatever the C++ side happens to produce -- two implementations agreeing
    /// on a wrong answer is still a wrong answer.
    ///
    /// The empty input is the offset basis by definition, and the single-byte
    /// case is one direct application of the published formula, spelled out
    /// here so the expected value is derived rather than remembered.
    #[test]
    fn digest_matches_the_published_fnv1a_vectors() {
        assert_eq!(inlined::digest(b""), FNV_OFFSET_BASIS);

        let one_byte = (FNV_OFFSET_BASIS ^ u64::from(b'a')).wrapping_mul(FNV_PRIME);
        assert_eq!(one_byte, 0xaf63_dc4c_8601_ec8c);
        assert_eq!(inlined::digest(b"a"), one_byte);

        assert_eq!(inlined::digest(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn the_two_rungs_of_rust_agree() {
        for value in [0u64, 1, 42, u64::MAX, 0x0123_4567_89ab_cdef] {
            assert_eq!(inlined::mix(value), opaque::mix(value));
            assert_eq!(inlined::noop(value), opaque::noop(value));
        }
        let payload: Vec<u8> = (0u16..=511).map(|byte| byte as u8).collect();
        assert_eq!(inlined::digest(&payload), opaque::digest(&payload));
    }

    /// The measurement is only meaningful if all three rungs compute the same
    /// thing. When the C++ side is not linked this asserts the *absence* is
    /// reported honestly instead of silently substituting Rust.
    #[test]
    fn every_rung_agrees_or_says_it_is_absent() {
        let payload: Vec<u8> = (0u32..4096)
            .map(|byte| byte.wrapping_mul(31) as u8)
            .collect();

        if CROSS_LANGUAGE_LINKED {
            for value in [0u64, 1, 42, u64::MAX, 0x0123_4567_89ab_cdef] {
                assert_eq!(crossing::mix(value), Some(inlined::mix(value)));
                assert_eq!(crossing::noop(value), Some(inlined::noop(value)));
            }
            assert_eq!(crossing::digest(&payload), Some(inlined::digest(&payload)));
        } else {
            assert_eq!(crossing::mix(1), None);
            assert_eq!(crossing::noop(1), None);
            assert_eq!(crossing::digest(&payload), None);
        }
    }
}
