#!/usr/bin/env bash
# Run `nexora-client` on an X server, hold W for a second through the X
# server's test extension (XTEST), then press Escape until it exits
# (ADR-0032). Meant to run inside `xvfb-run`; needs `xdotool`.
#
# Escape is pressed again every second until the client exits: a focus
# change landing between a press and its release correctly releases the key
# (ADR-0031), and the next press is still a real key through the same path.
#
# usage: client-xtest.sh <path to nexora-client> [timeout seconds]
set -euo pipefail

client="$1"
timeout="${2:-60}"

"$client" --timeout "$timeout" &
pid=$!
# Bounded: a window that never opens is the client's failure to report.
timeout "$timeout" xdotool search --sync --name '^NEXORA$' windowfocus --sync \
  > /dev/null 2>&1 || true
sleep 1
xdotool keydown w 2> /dev/null || true
sleep 1
xdotool keyup w 2> /dev/null || true
while kill -0 "$pid" 2> /dev/null; do
  xdotool key Escape 2> /dev/null || true
  sleep 1
done
wait "$pid"
