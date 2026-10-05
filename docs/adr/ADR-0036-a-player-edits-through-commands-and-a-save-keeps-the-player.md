# ADR-0036 — A player edits the world through commands, and a save keeps the player

- **Status:** ACCEPTED
- **Date:** 2026-10-05
- **Amends:** [ADR-0035](ADR-0035-the-player-is-a-body-the-simulation-steers.md)
  (the player is saved; it is still not an entity) and
  [ADR-0032](ADR-0032-the-client-is-the-runtime-in-a-window.md) (the client
  edits the world it draws, and can go on from a world file)
- **Builds on:** [ADR-0010](ADR-0010-commands-are-intent-and-carry-their-own-authority.md)
  (commands), [ADR-0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md)
  (the ray query), [ADR-0033](ADR-0033-the-pass-culls-back-faces-and-splits-quads-at-every-corner.md)
  (corners), [ADR-0030](ADR-0030-the-first-render-pass-is-checked-against-a-ray-cast.md)
  (frames checked against a ray cast)
- **Informs:** the roadmap's Phase 2 (*interação*, *mundo pequeno navegável*),
  `DEBT-0050` (split: the save half resolved here)

## Context

The roadmap's Phase 2 is a vertical slice, and the loop that makes one a game
rather than a viewer is *world → player → interaction → state change → save →
reload → the world continues*. Before this ADR the client drew a generated
world and walked a player through it; nothing changed the world, and nothing
the player did survived the window closing.

The specifications fix the shape of the missing half:

- `PLAYER SYSTEM.md` PLAYER-13 to PLAYER-16: *Player → Interaction Ray →
  Target → Interaction API*, the context being *player, target, position,
  face*; PLAYER-21 and PLAYER-22: *Build Intent → Build Engine*, *Mining
  Intent → Build/Destruction Engine* — **the player does not change a voxel
  itself**.
- `Build & Destruction Engine.md` BUILD-1: a placement verifies *posição
  válida, chunk disponível, permissão, **colisão**…*, never a bare
  `world.setBlock`.
- `Command System.md` §72: a client's command is untrusted, and the authority
  does not take the actor's word for where it is.
- `PLAYER SYSTEM.md` PLAYER-53 and PLAYER-54: the player's *location* is
  saved, and the save is split by domain, never *"Player.json com tudo"*.

The pieces existed separately: the block commands and their handlers
(ADR-0010), the physics ray query (`PHYSICS.md` §23), the player (ADR-0035),
the world's save sections (ADR-0004). No Build Engine exists; the block
commands' handlers are its stand-in, and the only code that writes a block for
a player.

## Decision

1. **The interaction ray is the eye's.** `nexora_simulation::interaction::aim`
   casts physics' grid walk from the player's eye along `Eye::forward` — the
   camera's own formula, so the block aimed at is the block under the centre
   of the frame — at most `INTERACTION_REACH` = 5 blocks. It answers a
   `Target`: the first solid block, and the cell in front of the face the ray
   entered (`None` when the eye is inside a block).
2. **Mining and building are intent, and the authority decides.** The mouse's
   primary button is `action/mine`, the secondary `action/build`, in the
   client's one table (`controls::DEFAULT_BUTTONS`, HID Button page). They are
   edges — one click, one edit — and the player is never handed them:
   `Intent::block` goes to a `BlockEditor`, the player's authority. It casts
   the ray **from its own copy of the player**, sends `nexora:break_block` or
   `nexora:place_block` naming the cell's integers, and the command pipeline —
   the standard layers, then `PlayerTargetValidator`, then the one handler —
   decides. `apply` reports every `Edit` with the pipeline's answer.
3. **Two rules on the authority's side.** Reach: `BlockTargetValidator`'s
   rule, called rather than repeated, from the eye the authority recorded.
   Collision: a build into a cell the body occupies — read with physics' own
   touching convention, `Player::occupied_cells` — is `InvalidState`, the same
   answer as building into a cell a block already holds. The ray's reach plus
   half a cell's diagonal is held below the server's `MAX_REACH_BLOCKS` at
   compile time, so a target this player can aim at is never refused for
   range by its own authority, while a request from anywhere else still is.
4. **The command carries a cell, not an angle.** The ray is trigonometry a
   platform's `libm` could round differently (DEBT-0051); it ends in a cell,
   and replaying the command changes the same block on any machine.
5. **An edit is a command at a world tick.** In the client, a click on a frame
   that buys no tick is latched to the next frame that does, and spent once
   (as the jump is). Edits land in the Simulation stage, before the player's
   physics tick, so a body standing on a block just mined falls on that tick.
   Because no build may bury the body, a tick that has to push the player out
   of terrain is still a defect, not a fix.
6. **What an edit changes is meshed again, and nothing else.** A region is
   meshed again when it holds the edited cell or one of its six neighbours;
   it and the eight regions around it are uploaded again, each split at the
   corners of the regions around *it* (ADR-0033). A test holds the result to a
   from-scratch build of the edited world, region by region, byte by byte, and
   fails when either neighbourhood is narrowed. The editor is limited to the
   drawn cells: an edit the client cannot show is not one it makes
   (`DEBT-0052`, until streaming reaches the pass).
7. **An edit is not claimed drawn until the pixels say so.** Besides the first
   frame, the client reads back the first frame shown after the world first
   changed, and judges it against the ray cast of the changed world. Each is
   judged the moment it is read back, against the world it showed; the time
   spent judging is moved out of the frame clock.
8. **The player is its own save section.** `nexora:save/player` (version 1)
   holds the `PlayerState` — feet, velocity, whether it stood on something,
   yaw and pitch, as bits — beside the world's three sections, which neither
   know nor need it. `Player::resume` puts the body back exactly; from there it
   runs, tick for tick, the ticks the saved player would have run, to the bit.
   A section with an unknown version, the wrong length or a bad flag is
   refused by name; a state no player can be in, or one inside terrain, is
   refused by `resume`.
9. **The client can go on.** `nexora-client --world PATH` resumes the world
   and the player from `PATH` when it exists, creates them otherwise, and
   writes both back at shutdown. A saved player outside the drawn columns, or
   refused by `resume`, is replaced by the spawn search, and the report says
   which.

## Evidence (2026-10-05, the development container)

- **Headless slice**, every run: through a frame of HID usages per tick, the
  player looks straight down, is refused a block in its own feet, mines the
  block it stands on and falls 1.00, jumps and builds under its feet at the
  top of the jump, lands on it, and finds nothing in reach looking up. The
  world and the player are saved into one container, encoded, decoded,
  loaded and resumed — the player to the bit — and the reloaded world and the
  one never saved run 140 more ticks side by side, mining and building,
  identical tick by tick and to the last byte of their saves. 30 seeds at
  radii 1 and 2: 60 of 60.
- **Client on Xvfb** (lavapipe), through XTEST: W, the down arrow, a click of
  the primary button, Space and a click of the secondary one: `1 mined, 1
  built`; the first frame after the edit, read back from the surface, held
  against the ray cast of the edited world in 97,606 of 97,606 judged
  pixels. The same world file opened three more times went on at ticks 134,
  269 and 404 with the player where it was saved.
- **Tests**: 1,323 in the workspace, among them the incremental-remesh
  equality (mutation-checked on both neighbourhoods), the resumed-player
  equality (mutation-checked on the ground flag) and the authority's two
  rules.

## Consequences

- The roadmap's Phase 2 has its interaction, and its *mundo pequeno
  navegável* can be changed and kept. Not yet: textures in the pass,
  streaming into it, a crosshair or a build preview (BUILD-25), choosing what
  to build (the inventory's, Phase 5), pointer look.
- `DEBT-0050` splits. *The player is not saved* is resolved here. *The player
  is not an entity, and its body lives in a physics world of its own* stays
  open, its trigger unchanged: the first NPC or crate in the client's frame.
- The ray only finds blocks physics finds solid: a non-solid block cannot be
  mined until one exists that needs to be.
- Placement does not check support, orientation or block rules (BUILD-4,
  BUILD-5): there is one block to build, and it needs none of them.

## Migration

None for worlds: the world's sections are unchanged, and a world without a
player section loads as before and spawns its player by search.

## Compatibility

Save format unchanged; one new section, `nexora:save/player`, version 1.
`nexora-client` gains `--world` and three report lines (`start`, `edits`,
`edited frame`); `nexora-headless` gains two (`interaction`, `save and
reload`). `local-validation.py` gains the `client_resume` check.
