# NEXORA — TIME / CALENDAR SYSTEM

> Time is an authoritative world service used by simulation, scheduling, climate, agriculture, civilization, research and events. It must not be duplicated independently by each subsystem.

## Model
```text
World Clock
→ Calendar
→ Time Scales
→ Schedulers
→ Systems
```

## Responsibilities
- authoritative world time;
- tick/time-scale conversion;
- day/night and calendar rules;
- seasons, months and years;
- time dilation policies where supported;
- simulation scheduling windows;
- persistent timestamps and historical ordering.

## Time scales
`FRAME`, `SIMULATION_TICK`, `SECOND`, `MINUTE`, `HOUR`, `DAY`, `WEEK`, `MONTH`, `SEASON`, `YEAR`, `DECADE`, `CENTURY`.

## Calendar
Dimensions or civilizations may expose custom calendars, but the engine retains a canonical world-time representation for ordering and persistence.

## API
```ts
interface ITimeSystem {
  now(): WorldTime;
  advance(delta: WorldDuration): void;
  calendar(world: WorldID): CalendarState;
  schedule(task: TimeTask): ScheduleHandle;
  convert(time: WorldTime, scale: TimeScale): number;
}
```

## Determinism
Authoritative simulation uses world time, not wall-clock time, except for explicitly local/presentation services.

## Persistence
Save current world time, calendar configuration and persistent scheduled tasks. Reconstruct derived sun/season state after load.

## Tests
Pause/resume, long-term simulation, leap/calendar rules, scheduled task ordering, save/reload and deterministic replay.

## Invariants
- One authoritative world clock per simulation context.
- Domain systems do not create competing authoritative clocks.
- Presentation time cannot silently change simulation time.
