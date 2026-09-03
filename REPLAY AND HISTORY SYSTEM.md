# NEXORA — REPLAY / HISTORY SYSTEM

> Replay records enough authoritative information to reproduce, inspect or audit a simulation interval. It is not video capture.

## Modes
```text
Input Replay
Command Replay
Event Replay
Simulation Trace
Network Replay
Full Debug Capture
```

## Core data
`ReplayHeader`, `ReplayFrame`, `CommandRecord`, `EventRecord`, `StateCheckpoint`, `ReplayIndex`.

## Determinism
A replay stores world/simulation/API/mod fingerprints, seed/version, time origin and command/event sequence. Unsupported nondeterministic dependencies are recorded or rejected.

## Checkpoints
Use periodic state snapshots plus command/event records between checkpoints rather than infinite raw state history.

## Uses
Bug reproduction, desync analysis, AI debugging, world-history inspection, benchmark comparison and multiplayer diagnostics.

## Security / privacy
Production captures must follow configured retention and minimize sensitive player data. Admin access is audited.

## API sketch
```ts
interface IReplaySystem {
  start(options: ReplayOptions): ReplayID;
  record(record: ReplayRecord): void;
  checkpoint(id: ReplayID): void;
  stop(id: ReplayID): ReplayArtifact;
  play(id: ReplayID, options: ReplayOptions): ReplaySession;
}
```

## Integration
Command/Event Bus provide authoritative records; Persistence stores artifacts; Networking can capture protocol metadata; Testing consumes replays as fixtures.

## Tests
Round-trip replay, deterministic divergence detection, checkpoint recovery, corrupted artifact and version mismatch.

## Invariants
- Replay cannot execute privileged commands without authorization.
- Replay artifacts identify the exact compatibility context.
- Playback does not mutate production worlds by default.
