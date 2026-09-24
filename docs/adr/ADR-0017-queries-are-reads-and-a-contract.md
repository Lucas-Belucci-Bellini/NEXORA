# ADR-0017 — Queries are reads, and a contract

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Completes:** `NEXORA ARCHITECTURE RULES.md` §4 (*Command = intenção,
  Event = fato, Query = leitura*) — commands were ADR-0010, events existed
  from the start, queries were the missing third
- **Touches:** public API (a new contract type), mod boundary

## Context

The freeze checklist listed *Event / Command / Query semantics* as partial:
*"queries are not"* built. Every read in the engine was a direct call into the
owning crate's internals — `World::get_block` returning a session-local
runtime id — which is fine inside the owner and wrong everywhere else:

- `NEXORA PUBLIC API AND CONTRACTS.md` lists `QUERY → read information` as a
  public contract type, and says runtime handles are not stable identifiers.
- `NEXORA DATA OWNERSHIP AND SOURCE OF TRUTH.md`: *READ → query / snapshot /
  public view*.
- `Event Bus.md` §30: queries are a direct API, not events — *"o Event Bus não
  deve virar RPC universal."*
- `Mod Runtime.md` §22/§79/§80: mods get `WorldQueryCapability`, not raw world
  memory; worker threads *query a snapshot*.

## Decision

A new crate, `engine/query` (foundation + runtime + command):

- **`Query<S>`** answers from `&S`. There is no mutable path, so a query that
  changes something is a type error.
- **`QueryDefinition`**: id, `QueryVersion`, the owning system, an access
  policy, a result budget. The access policy **is the command system's
  `SourcePolicy`** — one identity model (`Actor`, `Source`) for both halves of
  the public contract, closed until opened. A definition that admits nobody,
  or has a zero budget, is refused at registration. A second registration of
  the same id is refused: a query has one owner.
- **`QueryService::ask`** checks definition → version → actor → source, calls
  the handler with its budget, then **re-counts the answer**: a handler that
  ignores its budget is refused and counted as an overrun, so the budget is the
  service's property, not the handler's courtesy.
- `ask` takes `&self`; counters are atomic. Asking needs no exclusive access,
  which is what §80's "worker thread queries a snapshot" requires.

In `engine/simulation` (the crate that may see the world), the first three
world queries: `nexora:block_at`, `nexora:blocks_in_region` and
`nexora:world_time`. They answer in **identifiers**, never runtime ids; a cell
that is not resident is `Unavailable`, never air; a region query is bounded
both by results (32,768) and by **cost** (32,768 cells scanned), because a
million cells of air answer with nothing and still cost a million lookups.

## Consequences

- The headless slice reads every probe back through `nexora:block_at`, as an
  ordinary consumer, and asks two questions that must be refused (a region far
  past the scan limit, a contract version that does not exist):
  `queries 76 answered, 2 refused`. Its save is byte-identical to the build
  before this ADR — reads changed nothing.
- **Not built:** snapshots (queries read the live world, so they are safe only
  where the world is not being written concurrently — today, everywhere,
  because simulation is single-threaded per authority), per-actor query quotas,
  and entity queries through the contract (`engine/entity` has its own query
  API, which is the owner's internal one).

## Alternatives considered

- **Queries as events with a reply.** Rejected by `Event Bus.md` §30 by name.
- **A query module inside `engine/command`.** Rejected: that crate's contract
  is "intent", and reads are not intent. The dependency on it is for the shared
  identity model only.
