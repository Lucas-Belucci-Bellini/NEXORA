#!/usr/bin/env bash
# Run `nexora-client` on an X server and play it through the X server's test
# extension (XTEST), then press Escape until it exits (ADR-0032, ADR-0035,
# ADR-0036). Meant to run inside `xvfb-run`; needs `xdotool`.
#
# The play, in an order whose outcome the terrain cannot change:
#
#   1. hold W for a second: the player walks;
#   2. hold the down arrow until the view is straight down (89.9 degrees
#      clamps it; 1.5 s turns 135);
#   3. click the primary button: the block under the feet is mined, and the
#      player falls into the hole;
#   4. tap Space, and click the secondary button (X11's button 3) near the
#      top of the jump: the cell under the feet is free there, the block is
#      built, and the player lands on it. The feet clear that cell from about
#      0.3 s to 0.7 s after the jump; the click is at 0.45 s.
#
# Escape is pressed again every second until the client exits: a focus
# change landing between a press and its release correctly releases the key
# (ADR-0031), and the next press is still a real key through the same path.
#
# usage: client-xtest.sh <path to nexora-client> [timeout seconds] [client arguments...]
set -euo pipefail

client="$1"
timeout="${2:-60}"
shift $(( $# > 1 ? 2 : 1 ))

"$client" --timeout "$timeout" "$@" &
pid=$!
# Bounded: a window that never opens is the client's failure to report.
timeout "$timeout" xdotool search --sync --name '^NEXORA$' windowfocus --sync \
  > /dev/null 2>&1 || true
sleep 1
xdotool keydown w 2> /dev/null || true
sleep 1
xdotool keyup w 2> /dev/null || true
xdotool keydown Down 2> /dev/null || true
sleep 1.5
xdotool keyup Down 2> /dev/null || true
sleep 0.5
xdotool click 1 2> /dev/null || true
sleep 1
xdotool key space 2> /dev/null || true
sleep 0.45
xdotool click 3 2> /dev/null || true
sleep 1
while kill -0 "$pid" 2> /dev/null; do
  xdotool key Escape 2> /dev/null || true
  sleep 1
done
wait "$pid"
