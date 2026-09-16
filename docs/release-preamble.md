## What is in the archive

Three command-line programs, the content they read, and both licences.

| | |
| --- | --- |
| `nexora-headless` | the vertical slice: builds a world from a seed, generates chunks across a worker pool, edits voxels, drops a character and simulates it until it settles, streams an observer away and back, saves, reopens and verifies every probe |
| `nexora-benchmark` | the measurement harness behind `docs/benchmarks/PHASE-0-BASELINE.md` |
| `nexora-texture-forge` | the procedural material generator (`TEXTURE FORGE.md`) |

**There is no renderer and no window.** That is the state of the project, not a
packaging mistake — see "Where the project is" in the README. These programs
report to a terminal and exit non-zero if the engine does not hold up.

## Running it

```bash
./nexora-headless                 # the slice, with its defaults
./nexora-headless --help          # seed, radius, worker threads, save path
./nexora-benchmark --markdown     # the measurements, as a table
```

Every binary in this archive was run on its own platform by the workflow that
built it, before it was packaged: a build that compiles and cannot run does not
get this far.

## Verifying the download

```bash
shasum -a 256 -c SHA256SUMS       # or: sha256sum -c SHA256SUMS
```

`SHA256SUMS` covers every archive in this release.
