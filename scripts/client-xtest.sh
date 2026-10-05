#!/usr/bin/env bash
# Run `nexora-client` on an X server, hold W for a second through the X
# server's test extension (XTEST), then press Escape until it exits
# (ADR-0032). Meant to run inside `xvfb-run`; needs `xdotool`.
#
# Between the two, the player's hands (ADR-0036): the down arrow held for a
# second turns the view straight down, the primary button breaks the block
# under the feet and the player falls into the hole; the secondary button
# then asks for a block in the cell the feet stand in, which the authority
# refuses. The pointer is put over the window first: a button goes to the
# window under it.
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
window=$(timeout "$timeout" xdotool search --sync --name '^NEXORA$' 2> /dev/null \
  | head -n 1 || true)
if [ -n "$window" ]; then
  xdotool windowfocus --sync "$window" > /dev/null 2>&1 || true
  xdotool mousemove --window "$window" 40 40 2> /dev/null || true
fi
sleep 1
xdotool keydown w 2> /dev/null || true
sleep 1
xdotool keyup w 2> /dev/null || true
xdotool keydown Down 2> /dev/null || true
sleep 1
xdotool keyup Down 2> /dev/null || true
xdotool click 1 2> /dev/null || true
sleep 1
xdotool click 3 2> /dev/null || true
sleep 1
while kill -0 "$pid" 2> /dev/null; do
  xdotool key Escape 2> /dev/null || true
  sleep 1
done
wait "$pid"
