# catalog-digest

`content/first-generation/CATALOG.md` records, for every albedo, its
resolution, its format, its size and the **FNV-1a 64 of its PNG's bytes**. This
tool computes those columns for any file, in **Python, standard library only**,
and answers over a versioned line contract on stdin and stdout.

It is the language gate's **one cross-language tool call**
(`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17,
[ADR-0034](../../docs/adr/ADR-0034-the-freeze-gates-the-cores-language-not-every-boundarys.md)):
a tool in the language map's research and automation language, called by the
Rust benchmark through the boundary layer the map gives tools — **IPC** — and
timed. `nexora_benchmark::tool` starts it, checks every answer against Rust's
own computation (`nexora_foundation::hashing`), and only then times it; the
numbers are in Appendix P of
[`docs/benchmarks/PHASE-0-BASELINE.md`](../../docs/benchmarks/PHASE-0-BASELINE.md).

## Running it

```sh
python3 -I -S -B tools/catalog-digest/catalog_digest.py
```

It writes its banner, then answers one request per line until its input
ends. By hand:

```sh
printf '1 digest %s\n' "$PWD/albedo.png" | python3 -I -S -B tools/catalog-digest/catalog_digest.py
```

The flags are part of the contract's invocation: `-I` isolates it from
`PYTHON*` variables and the user's site directory, `-S` leaves site-packages
off the path — so "standard library only" is structural, and a third-party
import fails everywhere rather than on the machines without the package — and
`-B` writes no bytecode next to the source. Python 3.8 or later.

## Contract: NEXORA tool IPC, protocol version 1

Bytes are UTF-8. Every line ends in `\n`; the tool also accepts `\r\n` on
requests. Fields are separated by **one** space.

### Session

1. The client starts the tool with stdin and stdout piped.
2. The tool writes its **banner**, one line, before reading anything:

   ```text
   nexora-tool/1 catalog-digest/1
   ```

   `nexora-tool/1` is the protocol and its version; `catalog-digest/1` the
   tool and its version. A client refuses any other protocol version.
3. The client writes requests; the tool writes **exactly one response per
   request, in order**, flushing after each.
4. The client ends the session by **closing stdin**. The tool exits with
   status 0, having written nothing more. stderr carries nothing in a
   correct session; a traceback there is a bug in the tool.

### Request

```text
<id> digest <path>
```

- `<id>`: decimal, ASCII digits only, at most 20 of them. Echoed in the
  response, so a client can tell a desynchronised stream from an answer.
- `digest`: the one verb of version 1.
- `<path>`: everything after the second space, to the end of the line: a file
  path, absolute or relative to the tool's working directory, which may hold
  spaces but no line break.

### Response

A digest:

```text
<id> ok fnv1a64=0x<16 lowercase hex> bytes=<decimal> ihdr=<w>,<h>,<depth>,<colour> work_ns=<decimal>
```

| field | meaning | `CATALOG.md` column |
| --- | --- | --- |
| `fnv1a64` | FNV-1a 64 of every byte of the file | albedo FNV-1a 64 |
| `bytes` | the file's size | size |
| `ihdr` | width, height, bit depth and PNG colour type from the IHDR chunk; `none` unless the file starts with the PNG signature and a 13-byte IHDR whose CRC matches | resolution, format (`16,16,8,6` is 16×16 RGBA8) |
| `work_ns` | the tool's own time for reading and digesting the file, by `time.perf_counter_ns` | — |

All four fields are required, each once, in any order. A field version 1 does
not define is a different protocol version, and a version 1 client refuses it.

A refusal:

```text
<id> err <code> <message>
```

`<id>` is `-` when the request was too malformed to carry one. `<message>` is
one line for a person; `<code>` is for a program:

| code | when |
| --- | --- |
| `bad-request` | no decimal id, no verb, no path, or the line is not UTF-8 |
| `unknown-verb` | a verb other than `digest` |
| `unreadable` | the path could not be opened or read |

Errors cross the boundary as these answers, never as an exception: the rule
`NEXORA LANGUAGE AND FFI BOUNDARY.md` sets for every boundary. A refused
request does not end the session.

## What it is not

Not a hot path. On the machine Appendix P measured, a round trip costs about
0.2 ms and a tool run once 40 to 230 ms; an FFI call costs about a
nanosecond. That gap is the evidence for the language map's rule that
per-frame and per-tick work stays inside the owning runtime, and the reason a
tool like this answers a batch, not a voxel.
