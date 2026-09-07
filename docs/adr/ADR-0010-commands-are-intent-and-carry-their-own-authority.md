# ADR-0010 — Commands are intent, and the boundary is enforced by the crate graph

- **Status:** ACCEPTED
- **Date:** 2026-09-07
- **Closes:** `DEBT-0007` (the event bus delivered only facts, with no path for
  request-and-response)
- **Extends:** [ADR-0003](ADR-0003-workspace-layout-enforces-dependency-matrix.md)
  (a fourth boundary made structural rather than reviewed)
- **Follows:** [ADR-0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md)
  and [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md)
  (same shape, same reason)

## Context

`NEXORA ARCHITECTURE RULES.md` §4 states the separation in three lines:

```text
Command = intenção
Event   = fato ocorrido
Query   = leitura
```

and then the prohibition: *"Não usar Event como comando disfarçado."*

Phase 0 built the event bus and no commands, per ADR-0005. `DEBT-0007` recorded
the consequence, and named the risk exactly: systems needing question-and-answer
*"podem ser tentados a usar fatos como comandos"* — the deviation being
**"fácil e difícil de reverter depois"**.

`NEXORA DEVELOPMENT ROADMAP.md` puts commands in **Phase 1 — Engine Bootstrap**,
alongside events. It is the earliest open architectural debt in the register.

## Problem

Three, and they pull in different directions.

**A missing concept gets substituted, not noticed.** Nothing in the engine
prevented a `BlockBreakRequested` event with a handler that performs the break.
`Event Bus.md` §44 even lists request events as a natural idea, while §45 warns
against turning the bus into RPC. The absence of commands is the pressure that
produces that mistake, so the fix is to supply the concept.

**A command system is a security boundary, and it is easy to build one that
isn't.** §71 puts network, script, mod and console input through validation,
authorization, quota and authority before the world changes. §72 refuses to let
*internal* be a synonym for *valid*: *"nunca assumir internal = always valid,
porque bugs internos também existem."*

**§134 lists what the system must not contain** — combat, AI, inventory, block,
crafting and economy rules, physics, worldgen. That list is easy to agree with
and easy to violate one convenient import at a time.

## Options

1. **Extend the event bus with request/response.** Smallest diff, and it is the
   exact deviation `NEXORA ARCHITECTURE RULES.md` §4 forbids. `Event Bus.md` §45
   warns against it in the bus's own document.
2. **A command module inside `nexora-runtime`.** Correct layer; §134's boundary
   stays a comment, because everything the runtime can reach, the module can.
3. **A separate crate depending on foundation and runtime only.**
4. **Build all of CMD-0…CMD-15.** Most of it has no caller: networking,
   inventory, scripts, AI and an admin console do not exist.

## Decision

**Option 3, scoped to CMD-0 through CMD-4.**

`nexora-command` declares the framework — identity, definitions, instances,
lifecycle, layered validation, registry, dispatch, queue, results — and depends
on `nexora-foundation` and `nexora-runtime`, **nothing else**.

```text
nexora-foundation ──► nexora-command ──┐
                                       ├──► nexora-simulation ──► headless
nexora-foundation ──► nexora-world ────┘
```

### §134 becomes a build error

The list of things the command system must not contain is exactly the list of
crates it cannot reach. `nexora-command` cannot hold a block rule because
`nexora-world` is not in its manifest, and adding it would be a visible change
to a file rather than an import nobody notices. This is the same mechanism
ADR-0007 used for physics and ADR-0008 for streaming; it is the third time it
has paid, and the pattern is now the default answer for a boundary the
documents state in prose.

Block handlers therefore live in `nexora-simulation`, the one crate allowed to
see both sides — mirroring §28's own split: *the handler adapts, the specialized
system decides.*

### Validation is layered, in a fixed order, with no bypass

§18 opens by rejecting one giant `validateCommand()`, so each concern is its own
`Validator` and the pipeline runs them in §17's order:

```text
structural → identity → authorization → cancellation → expiry → quota → domain
```

The order is a **security property, not an optimisation**. Nothing reaches a
layer that does real work until the cheap universal checks have passed;
reordering would let an unauthenticated caller make the server work, which is
how a validation pipeline becomes an amplification vector.

There is **no trusted path**. `Source::is_externally_controlled` exists for
quotas and diagnostics, and no code branches on it to skip a check — §72. A
test submits a server command that the definition does not permit and asserts it
is denied exactly like a player's.

### Deny by default, in three places

- A `SourcePolicy` starts empty and permits nothing. A definition that forgot to
  say who may send it is **unreachable**, not open.
- `AuthorityPolicy` defaults to `ServerRequired` (§43).
- `RatePolicy` defaults to a **finite** 64 per tick. A definition that never
  considered flooding still gets a ceiling — §63's example is 100,000
  `BreakBlockCommand`s, and the queue must not be what discovers that.

A definition that can never accept anything is refused **at registration**
rather than at the first rejection, so the mistake is loud where it was made.

### The quota is charged after the cheap layers, not before

Counting malformed requests against an actor's budget would let a flood of
garbage exhaust a legitimate player's allowance on their behalf. A test submits
ten malformed requests and then one good one, and asserts the good one still
executes.

### Priority orders, and only orders

§32: *"prioridade nunca deve permitir violar regras de segurança."* The queue
sorts by priority and has no path to the validation pipeline, so a `Critical`
command cannot skip a layer — it can only go first. Ties break on enqueue tick
then instance id, so the order is **total and independent of hashing**;
`NEXORA REPLAY AND DETERMINISM.md` needs the same inputs to produce the same
state, and "whatever order the queue happened to be in" does not replay.

### A command has exactly one authority

§110: a command routes to one handler, an event to many subscribers. Attaching
a second handler for the same command is an **error**, not a second
subscription. Replacing the first silently would mean whichever module loaded
last decides what breaking a block means.

## Consequences

- **`DEBT-0007` closes.** There is now a path for intent, so nothing has to
  reach for an event to express one.
- **The reachable part of §115 runs in the vertical slice**: intent → validation
  → one authority → a changed world → an event name. Networking, Build &
  Destruction, Loot and Item do not exist, so this is a reduced version and the
  slice says so.
- **The slice's command stage is a deliberate no-op on the world** — it places a
  block and breaks it again — so the byte-identical save comparison across
  worker counts keeps measuring determinism rather than whether the command
  stage ran the same way twice. A check inside the stage fails loudly if it ever
  leaves a block behind, and that check caught a real leaked world handle during
  development.
- **The slice refuses on purpose.** Three of its five commands are turned away
  (nothing to break, out of reach, wrong actor kind). A pipeline that has only
  ever been shown accepting things has not been shown to refuse anything, and
  refusing is the half that carries §71.
- **`Domain::Command` and `CommandVersion` join the foundation**, so command
  failures are attributable in diagnostics and a command's schema is versioned
  independently of the engine (§78, §79).
- **CMD-5 to CMD-15 are deliberately absent**, each recorded with a trigger:
  networking integration, transactions and rollback, scripts, NPC/AI, admin
  console, replay, dry-run and simulation, batching and region affinity. §35
  idempotency is **detected** (a duplicate instance id is refused by the queue)
  but not **transacted** — reservations, commit and rollback are CMD-9.
- **`CommandInstanceId` never issues `u64::MAX`.** The counter advances before
  the id is returned, so exhaustion is caught one id early. Handing out that
  last id would need a separate flag, and one wasted id out of 2^64 is a better
  trade than extra state on the path that must never be wrong.

## Migration

None. Nothing dispatched before. The existing direct `World::set_block` edits in
the slice are untouched — commands are an additional path, not a replacement,
and converting the edit stage would have conflated this change with a rewrite of
the probe verification that guards the save.

## Compatibility

No save format change: commands are not persisted in this phase. §58 and §59
(persistence and replay of the command log) are CMD-13, and `CommandVersion`
exists so that log can be read back by a build whose other contracts have moved
on.
