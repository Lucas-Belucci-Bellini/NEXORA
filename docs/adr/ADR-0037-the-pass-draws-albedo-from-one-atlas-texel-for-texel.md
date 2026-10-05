# ADR-0037 — The pass draws albedo from one atlas, and a frame is still checked texel for texel

- **Status:** ACCEPTED
- **Date:** 2026-10-05
- **Amends:** [ADR-0030](ADR-0030-the-first-render-pass-is-checked-against-a-ray-cast.md)
  (the ray cast reports the point and the surface it reached, and judges a
  texel, not only a face colour) and
  [ADR-0032](ADR-0032-the-client-is-the-runtime-in-a-window.md) (the client
  takes content and the forge's output, and draws them)
- **Builds on:** [ADR-0021](ADR-0021-the-runtime-resolves-assets-through-the-forges-index.md)
  and ADR-0022 (the runtime resolves, verifies and decodes the forge's
  textures), [ADR-0033](ADR-0033-the-pass-culls-back-faces-and-splits-quads-at-every-corner.md)
  (corners, culling), [ADR-0036](ADR-0036-a-player-edits-through-commands-and-a-save-keeps-the-player.md)
  (the edited frame)
- **Amends also:** [ADR-0033](ADR-0033-the-pass-culls-back-faces-and-splits-quads-at-every-corner.md)
  (the grazed-block rule of the reference applies to the face-colour pass)
- **Opens:** `DEBT-0053`
- **Informs:** the roadmap's Phase 2 (*texturas no pass*), the first
  generation's catalog (*integration_status*)

## Context

Twenty-six first-generation materials existed as 16×16 albedo PNGs, each
resolved by identifier, hash-checked and decoded by the runtime (ADR-0021,
ADR-0022), and none was drawn: the pass coloured a face by its direction
(ADR-0030). The generated world's own blocks — `nexora:block/stone`, `dirt`,
`grass` — had no material at all, so even a texturing pass would have drawn
the world as missing textures.

`RENDERER and GRAPHICS.md` RENDER-15 fixes the shape: *Block IDs → Texture
Atlas → GPU*. What this project adds is the rule that has held since
ADR-0030: a frame is not believed because it looks right, only when every
pixel read back from the surface agrees with a CPU ray cast. A textured frame
has to keep that rule, or texturing would be the first thing on screen nobody
checks.

## Decision

1. **One atlas of 16×16 tiles.** `nexora_render::atlas::Atlas` holds a tile
   per surface (a material's runtime id), `ATLAS_COLUMNS` = 16 wide and as
   many rows as needed, at most `MAX_TILES` = 8,192 (an 8,192-texel-tall
   image, the size every `wgpu` backend must support). Tile 0,
   `UNMAPPED_TILE`, is diagonal magenta and near-black bands: a block without
   a texture is visibly wrong, never silently another block. A tile that is
   not 16×16, not opaque or a second time the same surface is refused by
   name.
2. **The vertex carries the tile, the face carries the light.** A vertex's
   last word is `tile | face << 16`, flat-interpolated. The fragment shader
   takes the position's fraction within its block — a greedy rectangle of
   many blocks repeats its tile once per block — reads faces across X as
   `(z, y)`, across Y as `(x, z)`, across Z as `(x, y)`, counts vertical faces
   from the top, and fetches the texel with `textureLoad` at integer
   coordinates. No sampler: filtering, mip levels and address modes cannot
   make a frame differ from the reference.
3. **Light in linear space, sRGB at both ends.** The atlas is uploaded as
   `Rgba8UnormSrgb`, so the GPU decodes it; the albedo is multiplied by a
   fixed factor per face (`FACE_SHADE`: tops 1.0, bottoms 0.5, the two
   horizontal axes 0.8 and 0.9 — not a light model, RENDER-18 has none yet);
   the target is `Rgba8UnormSrgb`, so the GPU encodes the result.
4. **One rule, written twice and checked against itself.** `texel_of`,
   `FACE_SHADE` and `shade_texel` are Rust functions, and the WGSL is
   generated from the same constants (`textured_wgsl`). The reference ray
   cast now reports the surface and the exact point it reached;
   `check_frame_shaded(…, Shading::Albedo { atlas, encoded })` expects that
   point's texel, shaded, and accepts `ALBEDO_TOLERANCE` = 2 per channel —
   the rounding of two sRGB conversions done by different hardware.
5. **What the rasteriser may legitimately do, modelled — not tolerated.**
   A pixel that does not match is accepted only when the reference can name
   the quad that drew it, and is counted as `snapped`:
   - *the neighbour quad*: at a quad's edge the rasteriser may give the pixel
     to the quad across it, which reads its position at the pixel's centre —
     where the centre ray crosses **its own plane**, past its edge, so the
     fraction wraps to the far side of its tile. Two quads in one plane meet
     the centre ray at the same point; at a depth edge they do not.
   - *the texel edge*: the position is interpolated from vertices snapped to
     the sub-pixel grid, which Vulkan lets be as coarse as a sixteenth of a
     pixel; a texel boundary within `TEXEL_SNAP` = 1/16 px of the centre,
     on the drawing quad's plane, may fall either way. Geometry edges keep
     ADR-0030's `SNAP` = 1/64 px.
   - *the grazed block* (also in the face-colour pass): a block whose face
     the centre ray meets within `SNAP` of the face's edge may lose the pixel
     to what is behind it, or to its own face across that edge. With the eye
     at a block's centre, rays along a diagonal graze corners lined up with
     it — (2, 2), (8, 8) — and what shows lies in a wedge thinner than any
     ring of samples. The reference walks the centre ray on past each grazed
     block (`GRAZED_DEPTH` = 4) and also tries the block's other faces that
     face the eye. A back face is never one of them.
   The textured pixels the first two accept are `DEBT-0053`.
6. **Content restyles engine blocks.** Block content schema 2 adds
   `"surfaces": [{ "block", "surface" }]`: an engine block (one the world
   registers, not the document) drawn with a material the document lists.
   `content/first-generation/blocks.json` gives `nexora:block/stone` the
   mountain stone, `dirt` the dry earth and `grass` the grass. Schema 1 is
   still read; `surfaces` in it is refused, as is a block named twice or a
   surface the document does not list.
7. **The client draws what the forge wrote.** `nexora-client --content PATH
   --resources DIR` loads the blocks, opens the forge's `resources.json`
   through the `ResourceManager`, decodes each material's albedo at most 16×16
   and builds the atlas. Without `--content` the pass is the face-colour pass
   of ADR-0033, unchanged. A new report line says which: `textures N tiles in
   a WxH atlas, K blocks without a surface` or `textures none: faces coloured
   by direction`. Both the first frame and the first frame after an edit are
   judged against the textured ray cast. `--capture PATH` writes them as PPM
   for a person to look at; nothing is judged from the capture.

## Evidence (2026-10-05, the development container, lavapipe)

- **The pass alone** (`engine/render/tests/pass.rs`): a layered scene of
  three surfaces with deliberately asymmetric tiles, 14,549 judged pixels,
  14,549 matching, 0 snapped.
- **Mutations**, each caught: in the shader, no vertical flip (2,354 wrong
  of 14,549) or the texel off by one (2,455 wrong); in the mesh, reversed
  winding (in the client, 96,563 back faces in the face-colour frame and
  90,636 wrong texels in the textured one). Turning culling off is caught on
  one seed of six, with or without the grazed-block rule: since ADR-0033
  split every corner, a back face rarely reaches a pixel at all.
- **The client**, first-generation content and the forge's output — 32 tiles
  (31 materials and tile 0), 0 blocks without a surface:
  - forty seeds at 768×512, both passes: the face-colour frames
    15,573,617 of 15,573,620 judged pixels matching and 3 snapped; the
    textured frames 15,573,141 matching and 479 snapped. Every seed `OK`.
  - 384×256 (CI's size) and an edit through XTEST: the first frame and the
    edited frame `OK`; 1024×768: `OK`.
- **How the rules were found.** Each one came from a frame that failed and a
  pixel that could not be explained, dissected until the quad that drew it
  was named: the neighbour quad at 768×512 on the first seed; the depth edge,
  the texel edge and the grazed corners in a ten- and then forty-seed sweep.
  The grazed corners failed the **face-colour** pass too — seeds 9, 32 and
  36 at 768×512, before any texturing code ran — so the rule is ADR-0030's
  reference catching up with the rasteriser, not a tolerance for textures.
- **Looked at**: the captured frame shows mountain stone, dry earth and grass
  where the generator put stone, dirt and grass.
- **Local validation** (`--quick`, the same container): `client_textures`
  PASS. CI's Windows and macOS runners now run it too.

## Consequences

- The roadmap's Phase 2 has textures in the pass. The generated world is drawn
  in first-generation materials, and every material in `blocks.json` is a
  tile any block can show.
- `integration_status` can say **drawn**: a surface in the atlas, on the
  screen, checked by the ray cast.
- The light is a constant per face. Shadows, ambient occlusion and a sun are
  RENDER-18's, and each will need its own term in the reference.
- One atlas, built at start-up from everything the content lists: no
  streaming of textures, no arrays, no mip levels. At 16×16 and 8,192 tiles
  that is 8 MiB of texels, far from a limit today.
- Without `--content` nothing changes: the face-colour pass and its
  zero-tolerance check stay as they were.

## Migration

None. Block content schema 1 documents load as before; a world saved with
content still needs that content to load (ADR-0019's rule, unchanged).

## Compatibility

Save format unchanged. `nexora-client` gains `--content`, `--resources` and
`--capture`, and the `textures` report line. Block content schema 2.
`local-validation.py` gains the `client_textures` check.
