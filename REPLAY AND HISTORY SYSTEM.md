# NEXORA — REPLAY / HISTORY SYSTEM

> Replay records enough authoritative information to reproduce and inspect simulation intervals; it is not video capture.

## Modes
Input replay, command replay, event replay, simulation trace and network replay.

## Data
`ReplayHeader`, `ReplayFrame`, `CommandRecord`, `EventRecord`, `StateCheckpoint`, `ReplayIndex`.

## Determinism
Store seed/version, world/content/mod fingerprints, time origin and command/event sequence. Unsupported nondeterminism must be captured or rejected.

## Checkpoints
Use periodic snapshots plus records between checkpoints instead of infinite raw state.

## Uses
Bug reproduction, desync analysis, AI debugging, history inspection and benchmarks.

## API
```ts
interface IReplaySystem {
  start(options: ReplayOptions): ReplayID;
  record(record: ReplayRecord): void;
  checkpoint(id: ReplayID): void;
  stop(id: ReplayID): ReplayArtifact;
  play(id: ReplayID, options: ReplayOptions): ReplaySession;
}
```

## Security
Artifacts have explicit retention/access rules. Playback is sandboxed by default.

## Tests
Round-trip, deterministic divergence, checkpoint recovery, corruption and version mismatch.
