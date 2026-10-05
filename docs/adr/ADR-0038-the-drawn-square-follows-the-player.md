# ADR-0038 — The drawn square follows the player, through the engine's streaming

- **Status:** ACCEPTED
- **Date:** 2026-10-05
- **Amends:** [ADR-0032](ADR-0032-the-client-is-the-runtime-in-a-window.md)
  (the client draws a square that moves, not one fixed at start) and
  [ADR-0036](ADR-0036-a-player-edits-through-commands-and-a-save-keeps-the-player.md)
  (the edit area moves with it; a resumed player is always inside it)
- **Builds on:** [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md)
  (streaming decides residency through a backend), [ADR-0014](ADR-0014-a-region-file-is-authoritative-for-its-region.md) (retained
  columns), [ADR-0033](ADR-0033-the-pass-culls-back-faces-and-splits-quads-at-every-corner.md)
  (corners), [ADR-0030](ADR-0030-the-first-render-pass-is-checked-against-a-ray-cast.md)
  (frames checked against a ray cast)
- **Resolves:** `DEBT-0052`'s horizontal half
- **Opens:** `DEBT-0054`
- **Informs:** the roadmap's Phase 2 exit (*mundo pequeno navegável e
  carregamento/unloading estável*)

## Context

The client generated and meshed a square of columns around the origin before
the first frame, and drew exactly that until it closed (DEBT-0052). A player
walking off it walked onto columns nothing drew; the editor was fenced to it;
a saved player outside it was spawned again by search. The roadmap's Phase 2
lists *streaming inicial* and ends on *carregamento/unloading estável*.

The engine already had what residency needs: `nexora-streaming` decides what
is resident from interest, budgets and hysteresis (ADR-0008), and
`nexora_simulation::WorldResidency` is the one place voxel chunks meet it —
an unedited column is dropped and generated again identically, an edited one
is held in `RetainedChunks` and given back. The headless slice proves both on
every run. The client used neither: it called `World::load_or_generate`
itself, which is exactly the call that would bring an evicted, edited column
back from the seed without its edit.

## Decision

1. **The client names an interest; the engine decides residency.**
   `nexora_client::residency::ClientResidency` owns a `StreamingSystem` and
   `RetainedChunks`. Its one interest is the player's column, at the scene's
   radius plus one ring (the ring a drawn edge is meshed against), every
   column at full detail, with a hysteresis of one column so stepping back
   and forth over an edge does not generate the same columns again.
2. **The scene never generates.** `scene::build` and `Scene::recentre` refuse
   a square whose columns or ring are not resident, naming the first missing
   column. There is one loading path, through `WorldResidency`, so an edit in
   an evicted column survives walking away and back.
3. **The square follows the player's column.** In the frame's `World` stage,
   when the column the player ended the last frame in is not the square's
   centre, residency settles around it and the square moves: columns
   entering are meshed (through a dense snapshot, DEBT-0029), columns leaving
   are dropped, kept columns keep their meshes. `Shift` says, for each region
   now drawn, which index it had, and which regions' vertices changed: the
   entering ones, and the kept ones whose neighbours in the square changed,
   because their quads are split at their neighbours' corners (DEBT-0047).
   The device chunks are rebuilt by `restream` in the frame's own list:
   unchanged regions keep their chunks.
4. **A move settles inside the frame, with an unlimited budget.** The
   reference ray cast walks one box (ADR-0030), so the scene draws a full
   square or nothing; a move spread over frames would draw half a square.
   What a move costs a frame is reported (`streaming … slowest`), not
   budgeted: meshing still runs on the calling thread (DEBT-0018, DEBT-0027).
5. **The band is the generator's.** Every region spans the lowest surface the
   generator can produce less 8 blocks to the highest plus 8
   (`World::surface_range`), the same wherever the square is. Edits stay
   inside it (`DEBT-0052`, the vertical half).
6. **The edit area moves with the square** (`BlockEditor::set_area`), and a
   resumed world's square is centred on the saved player's column, so a saved
   player is always inside what is drawn. `Placed::OutsideDrawn` is gone.
7. **A move is not claimed drawn until the pixels say so.** The first frame
   after the square first moved is read back and judged against the ray cast
   of the square it moved to (`moved frame`), like the first frame and the
   first after an edit. One frame showing both an edit and a move is judged
   once.
8. **Before the save, retained columns go back into the world**
   (`RetainedChunks::flush_into`): the save writes the world, and an evicted
   edited column is not in it.

## Evidence (2026-10-05, the development container)

- **Bytes.** After every move of a walk — one column, diagonally, a jump of
  five that evicts an edited column, and back, at radius 1; diagonally and
  back at radius 2 — every region's mesh and vertex bytes equal those of a
  square built at the new centre from scratch, and the edit made before
  walking away is drawn on the way back. Mutation-checked: a `Shift` that
  re-uploads only the entering columns fails it.
- **Pixels.** A walk of moves at radius 2, the device chunks updated by the
  frame loop's own `restream`, drawn on lavapipe and judged by the client's
  own check after every move. Mutation-checked: `restream` handing a kept
  region the chunk at its new index instead of its old one draws 516 wrong
  pixels in the first moved frame.
- **Residency.** The scene and its ring are resident and nothing more; an
  edited column evicted on a walk comes back with its edit; a flush puts it
  back for the save.
- **The live client** draws the same first frame as before (97,505 of 97,505
  judged pixels matching on the CI seed), from 25 resident columns.

What the evidence does **not** include: a move in the live client. The
generated terrain is a column-by-column random height (`DEBT-0054`) — a field
of pillars a body cannot walk across — so in a window the player does not
reach the next column, and the report says `moved frame none: the player
stayed in its column`. The move path is the same code either way, and the
test above is where it is drawn and judged.

## Consequences

- The client's world is no longer bounded by where it started: what is drawn,
  edited and resident follows the player, and what falls behind is evicted
  or retained by the engine's rules.
- `DEBT-0052` keeps its vertical half: the band.
- Memory held for edits grows with how much was edited (20 KiB per edited
  column, DEBT-0020), not with how far the player went; the client does not
  yet spill retained columns to region files as the headless slice does.
- Columns resident in a resumed world's save but outside the interest are not
  tracked by the manager, so they stay resident for the run. They are bounded
  by what the save holds.
- `DEBT-0054`: the Phase 2 exit's *navegável* needs terrain a body can walk.

## Migration

None: the world's save is unchanged. A world file saved before this ADR
resumes centred on its player.

## Compatibility

`nexora-client`'s report: `world … of N resident` (was `generated`), and two
new lines, `streaming` and `moved frame`. The `start` line can no longer say
the saved player was outside the drawn columns.
