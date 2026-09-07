# ADR-0009 — A second stack measures kernels, and one crate may say `unsafe`

- **Status:** ACCEPTED
- **Date:** 2026-09-07
- **Informs:** [ADR-0001](ADR-0001-rust-is-the-reference-implementation-not-the-locked-stack.md)
  (supplies the gate's missing input; does **not** close it)
- **Amends:** [ADR-0002](ADR-0002-zero-external-dependencies.md) (adds an
  optional toolchain requirement, still no crate dependency)
- **Constrained by:** `NEXORA LANGUAGE AND FFI BOUNDARY.md` — *"The final
  language map must be recorded only after the benchmark defined by the
  Technology Benchmark Plan."*

## Context

`NEXORA TECHNOLOGY BENCHMARK PLAN.md` defines the gate that chooses the engine's
language, and the startup brief §8 is blunt about it: *"A linguagem NÃO deve ser
escolhida por moda. Antes de travar a linguagem final, execute o benchmark."*

Every stage of that benchmark that needs neither a GPU nor a second language has
been measured for three increments now. `DEBT-0008` has stayed open on one
sentence: a benchmark with one implementation measures an implementation, not a
language. Two of the plan's metrics were unreachable in principle from a
single-language build:

- **relative kernel cost** — is Rust's number good, or merely the only number?
- **FFI overhead** — listed as a metric, and there was no boundary to cross.

## Problem

Three, and the third is the one that nearly produced a false result.

**Scope.** A second *engine* is not on the table. Reimplementing world,
persistence, physics and streaming in C++ would cost more than everything built
so far and would be thrown away by the gate it informs.

**`unsafe`.** Calling across a C ABI requires `unsafe`, and every crate in this
workspace sets `unsafe_code = "forbid"`, which by design cannot be lifted by an
`allow` at the item.

**Attribution.** A kernel timed in Rust against the same kernel timed in C++
produces a ratio, and the obvious reading of that ratio — *this language is
faster* — is frequently wrong. The compiler backend is a variable of the same
order, and it is invisible unless you deliberately vary it.

## Options

For scope:

1. **A second engine.** Answers everything, costs more than the project to date.
2. **A kernel reference.** The hot arithmetic — RNG, hashing, spatial mapping,
   voxel access, worldgen, one physics sweep, CRC — mirrored in C++ and timed
   under the same methodology.
3. **Leave `DEBT-0008` open until Phase 1.** Defers the decision the plan says
   must precede the code.

For `unsafe`:

4. **Relax the workspace lint to `deny`.** One measurement crate's needs would
   weaken the guarantee on every crate that ships.
5. **A crate outside `engine/`, with its own lint configuration.**

## Decision

**Option 2 and option 5**, plus a control the first two options do not imply.

### The comparison is of kernels, and says so everywhere

`benchmarks/cpp/` is a C++20 reference implementation of the arithmetic the
engine runs in its inner loops. It is **not** an engine, has no world, no
persistence, no streaming and no allocator strategy, and it is not a candidate
implementation of anything. It exists to answer *"is this number good?"*, and it
answers that only for the kernels it contains.

### Conformance is a gate, not a courtesy

`scripts/compare-stacks.sh` emits 12 digests from every build and **refuses to
time anything** if one differs. Two stacks that disagree about what they compute
cannot be compared on how fast they compute it, and a timing table is a
persuasive way to publish a wrong number. The digests cover the RNG and its
derived streams, both hashes, Euclidean division, the heightmap, all 2,032,268
non-air blocks of one generated chunk, and the floating-point sweep including
the `1.9 - 0.9` case that ADR-0007 was written about.

### Every available C++ compiler is timed, not one

This was added after a single-compiler run produced a finding that was not true.
CRC-32 over 64 KiB:

| build | median | reading if this were the only C++ column |
| --- | ---: | --- |
| Rust (LLVM) | 364 µs | — |
| C++ / g++ (GCC) | 668 µs | *"Rust is 1.8× faster than C++"* |
| C++ / clang++ (LLVM) | 353 µs | *"C++ is 3% faster than Rust"* |

Both C++ builds compile the same source with the same flags. The two languages
agree to within a few percent on the same backend; the languages differed by
1.8× only because the backends did. **Where two C++ builds differ from each
other by more than either differs from Rust, the row is about a compiler.**

### One crate may say `unsafe`, and it is not an engine crate

`benchmarks/ffi-probe/` sets `unsafe_code = "deny"` and carries exactly one
`extern "C"` block plus three `#[allow(unsafe_code)]` call sites, each with a
`SAFETY:` comment. Everything under `engine/` keeps `forbid`, unchanged. The
engine's guarantee is not weakened by a measurement probe that never ships,
and putting the probe under `benchmarks/` rather than `engine/` is what makes
that structural instead of a promise.

The C++ side is **optional**: without the `cpp` feature the probe compiles no
`unsafe` at all, `cargo build --workspace` needs no C++ compiler, and the
benchmark reports FFI overhead as unmeasured rather than substituting a Rust
number under a cross-language label.

### The FFI ladder has three rungs, because one number would mislead

| rung | what it is |
| --- | --- |
| inlined | ordinary Rust the optimizer sees through |
| opaque | `#[inline(never)]` Rust — same language, no inlining |
| crossing | `extern "C"` into a C++ translation unit |

Reporting only `crossing - inlined` charges the boundary for the cost of not
inlining, which any `dyn`, function pointer or cross-crate call without LTO pays
too. The costs are separable and they were separated.

## Consequences

- **`DEBT-0008` is no longer blocked on the absence of a second language.** It
  is not closed: the gate also wants a GPU stage and an engine-scale comparison,
  and this supplies neither. Appendix D says precisely what it does and does not
  settle.
- **The benchmark harness gained an optional feature and a build script.** No
  crate dependency was added, so ADR-0002 holds; a C++ compiler is now an
  optional *toolchain* requirement, exercised in CI and never in the default
  build.
- **Two measurements of mine were wrong before this ADR could be written**, both
  self-inflicted and both found by disbelieving a number rather than by a test:
  a `std::function` in the C++ harness that Rust's monomorphised `measure` did
  not have, and LLVM collapsing four mixes into one `imul` when no barrier
  stopped it. Appendix D records both, because a methodology that was wrong
  twice in one increment is the part of this most likely to be wrong again.
- **`physics.axis_sweep` and `save.crc32_64kib` now exist on the Rust side too**,
  sharing the C++ fixtures exactly. They were added because the C++ reference
  timed them and Rust did not, and a comparison table with an unmatched row
  invites the reader to pair it with something that is not it.
- **The language map is still not locked**, and this ADR does not lock it.
  `NEXORA LANGUAGE AND FFI BOUNDARY.md` reserves that for the completed
  benchmark, and the gate is not complete.

## Migration

None. No engine crate changed behaviour; the two new Rust measurements are
additive, and the headless slice's byte-identical save comparison across thread
counts still passes.

## Compatibility

No save format change and no public API change. `benchmarks/` is not shipped and
is not part of the engine's dependency matrix.
