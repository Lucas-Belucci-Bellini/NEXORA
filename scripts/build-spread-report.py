#!/usr/bin/env python3
"""Report the per-row spread across several builds of identical source.

Reads the `--markdown` output of `nexora-benchmark`, one file per build, and
prints the range each timed row covered. Companion to `scripts/build-spread.sh`;
see `DEBT-0039` for why the number it produces is needed at all.
"""

import re
import sys

UNITS = {"ns": 1.0, "µs": 1e3, "us": 1e3, "ms": 1e6, "s": 1e9}
ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*([0-9.]+)\s*(ns|µs|us|ms|s)\s*\|")


def read(path):
    """Row name to nanoseconds, for the rows that report a time."""
    out = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            match = ROW.match(line)
            if match:
                out[match.group(1)] = float(match.group(2)) * UNITS[match.group(3)]
    return out


def show(nanos):
    for unit, scale in (("ms", 1e6), ("µs", 1e3), ("ns", 1.0)):
        if nanos >= scale:
            return f"{nanos / scale:.2f} {unit}"
    return f"{nanos:.2f} ns"


def main(paths):
    runs = [read(path) for path in paths]
    shared = sorted(set.intersection(*(set(run) for run in runs)))
    if not shared:
        sys.exit("no timed rows in common; did the benchmark output change shape?")

    rows = []
    for name in shared:
        values = [run[name] for run in runs]
        low, high = min(values), max(values)
        rows.append((100.0 * (high - low) / low if low else 0.0, name, low, high))
    rows.sort(reverse=True)

    print()
    print(f"| row | lowest build | highest build | spread across {len(runs)} builds |")
    print("| --- | ---: | ---: | ---: |")
    for spread, name, low, high in rows:
        print(f"| `{name}` | {show(low)} | {show(high)} | **{spread:.1f}%** |")

    print()
    print(
        f"{len(shared)} timed rows, {len(runs)} builds of identical source "
        "separated by neutral edits."
    )
    under20 = [row for row in rows if row[2] < 20.0]
    if under20:
        worst = max(row[0] for row in under20)
        print(
            f"Rows under 20 ns: **{len(under20)}, worst spread {worst:.1f}%**. "
            f"Every row: worst spread **{rows[0][0]:.1f}%** (`{rows[0][1]}`)."
        )
    else:
        print(f"No row under 20 ns. Worst spread **{rows[0][0]:.1f}%** (`{rows[0][1]}`).")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        sys.exit("usage: build-spread-report.py RUN.md RUN.md [RUN.md ...]")
    main(sys.argv[1:])
