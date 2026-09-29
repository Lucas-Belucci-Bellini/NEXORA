#!/usr/bin/env bash
# Run `nexora-window-probe --input` on an X server and press W through the
# X server's test extension (XTEST), which reaches the window the way a
# keyboard does (ADR-0031). Meant to run inside `xvfb-run`; needs `xdotool`.
#
# W is pressed again every second until the probe exits: a focus change
# landing between one press and its release correctly releases the key, and
# the next press is still a real key through the same path.
#
# usage: input-probe-xtest.sh <path to nexora-window-probe> [timeout seconds]
set -euo pipefail

probe="$1"
timeout="${2:-30}"

"$probe" --input --timeout "$timeout" &
pid=$!
# Bounded: a window that never opens is the probe's failure to report.
timeout "$timeout" xdotool search --sync --name "NEXORA input probe" windowfocus --sync \
  > /dev/null 2>&1 || true
while kill -0 "$pid" 2> /dev/null; do
  xdotool key w 2> /dev/null || true
  sleep 1
done
wait "$pid"
