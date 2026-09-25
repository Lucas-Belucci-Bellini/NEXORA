# ADR-0022 — One PNG decoder, and it lives in the engine

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Closes:** `DEBT-0037`
- **Amends:** the Texture Forge audit (`docs/texture-forge/AUDITORIA.md` §6),
  which placed PNG entirely in the tool because, at the time, only the tool
  touched a PNG

## Context

ADR-0021 gave the runtime a resource system that can resolve, verify and cache
a texture — and nothing that could turn the verified bytes into pixels. The
only PNG decoder, and the `inflate` beneath it, lived in `tools/texture-forge`,
interleaved with the encoder and compressor. The engine cannot depend on a
tool (`NEXORA DEPENDENCY MATRIX.md`).

## Options

1. **Copy the decoder into the engine.** Two decoders of one format, free to
   drift — the failure the project already refuses for physics and ballistics
   elsewhere ("one implementation, two hosts").
2. **Move the decoding half into an engine crate, and have the tool use it.**

## Decision

Option 2. A new crate, `engine/image` (foundation + asset + resource), holds:

- `inflate` — stored and fixed-Huffman blocks, the RFC 1951 length/distance
  tables, and **`inflate_bounded`**. The limit a PNG passes is the scanline
  size its own header declares, so the *decompression ratio* check of
  `RESOURCE AND ASSET SYSTEM.md` is exact: a stream that expands one byte past
  its header's promise stops there, instead of at a global 64 MiB ceiling.
- `png::decode`, `paeth`, `adler32`, `SIGNATURE`.
- `TextureLoader` — the `nexora_resource::Loader` that turns a verified
  resource into a `TextureMap` of the role the caller asked for, optionally
  refusing anything larger than a given edge (the slice uses 16, the first
  generation's size). Its cache cost is decoded bytes, because the budget is a
  memory budget.

The forge keeps the encoder and compressor, imports the tables and `paeth`,
and **re-exports** the decoder, so its callers did not change. Its validator
now decodes with the same code the runtime does.

## Consequences

- The runtime goes from `nexora:texture/stone/basalt/albedo` to pixels without
  the tool that wrote them. The headless slice does it for all sixteen
  first-generation stones with `--resources`, and CI runs it.
- A tampered file is refused by the resource index's hash before the decoder
  sees it (ADR-0021); a well-hashed file that is not a PNG this project writes
  is refused by the decoder, by name.
- **Known limit, deliberate:** dynamic-Huffman deflate blocks are not read. So
  the runtime cannot open a PNG written by an arbitrary external tool — only
  what the forge wrote, which is what the resource index vouches for. The
  import path for external images (audit §4.6) is where a full inflater
  belongs, behind its own limits.

## Compatibility

No format changed. `nexora_texture_forge::{decode_png, adler32, Decoded}` and
`deflate::{inflate, MAX_INFLATED_BYTES}` remain available through the
re-exports.
