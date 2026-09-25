# ADR-0021 — Resources are indexed, and verified before they are decoded

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Implements:** `RESOURCE AND ASSET SYSTEM.md`,
  `NEXORA MEMORY AND RESOURCE OWNERSHIP.md` *Asset cache*
- **Touches:** serialization (a new on-disk format the runtime reads), public API

## Context

The architecture freeze checklist listed *Resource ownership* and *Asset
lifecycle* as defined and **not built**. Both belong to roadmap Phase 1
(*"Core, module lifecycle, jobs, resources, …"*) and Engineering Planning
Stage 1 (*"Resource System"*). Neither needs a GPU. Meanwhile the forge had
begun writing real assets — sixteen first-generation textures — that the
runtime had no way to find except by path.

## Decision

A new crate, `engine/resource` (foundation + asset), implements the pipeline
`ResourceID → Manifest → Resolver → Loader → Cache → Runtime Handle`:

- **The manifest is the pipeline's INDEX stage**, `resources.json` at the root
  it describes: id, kind, relative path, exact size, FNV-1a 64, dependencies,
  optional/fallback. It is checked whole at read time — unsafe paths, dangling
  dependencies or fallbacks, cycles, a fallback on a required entry, a
  fallback of another kind or one that is itself optional are all refused
  before anything loads.
- **Bytes are verified before any loader sees them.** Size and hash must match
  the manifest; a mismatch is `Recovery::Quarantine` and is counted. A decoder
  is a parser of arbitrary bytes, so it is only ever handed bytes the index
  vouches for.
- **The cache is bounded in bytes**, evicts unpinned entries lowest priority
  then least recently used, refuses (with `Recovery::Retry`, evicting nothing)
  rather than exceed its budget when pins fill it, and publishes counters.
  Values are `Arc`s: eviction drops the cache's reference, never a holder's.
- **Handles are typed by what their loader produces** and carry no path.
- The forge writes the index at the end of every successful `batch`, and on
  `nexora-texture-forge index`.

## Deviation from the specification, recorded

The spec's `load` returns a `Promise`. This one is synchronous: *what runs off
the tick thread* is the job system's decision, not the resource manager's —
the same answer `DEBT-0018` gives for chunk generation. An asynchronous facade
over the job system is additive.

## Consequences

- The runtime reaches the sixteen first-generation textures by identifier —
  `nexora:texture/stone/basalt/albedo` — through the index, verified, within a
  cache budget small enough to force evictions
  (`tools/texture-forge/tests/first_generation.rs`).
- **The runtime still cannot decode a PNG.** The decoder lives in the forge,
  entangled with the encoder. Until it moves (`DEBT-0037`), a texture resource
  is loaded as verified bytes.
- Resource packs (layering several roots) are not built; one manager reads one
  root.

## Compatibility

`resources.json` is schema 1. A newer schema is refused by name.
