#!/usr/bin/env bash
# Chaos test (WP 7.2): REAPER restarted under a connected client, and a client that disappears
# while REAPER plays. Uses only the isolated REAPER in .dev/reaper-test, never the real one.
# Needs: cargo build --release -p reaper-extension && scripts/package_extension.sh
set -euo pipefail
cd "$(dirname "$0")/.."
source ~/.cargo/env
cycles="${1:-3}"
R="$PWD/.dev/reaper-test"
PROJECT="$PWD/.dev/projects/large-set-sample.rpp"
ENDPOINT="$R/RC2/endpoint.json"
[ -d "$R" ] && [ -f "$PROJECT" ] || { echo "missing $R or $PROJECT"; exit 1; }
cp dist/reaper_rc2-arm64.dylib "$R/UserPlugins/reaper_rc2-arm64.dylib"
cargo build --release -p link --examples
example() { ./target/release/examples/"$1" "${@:2}"; }

isolated_pid() { pgrep -f "REAPER.*-cfgfile $R/reaper.ini" | head -1 || true; }
start_reaper() {
  rm -f "$ENDPOINT" "$R/RC2/running" "$R/RC2/faulted"
  open -n -a /Applications/REAPER.app --args -cfgfile "$R/reaper.ini" "$PROJECT"
  for _ in $(seq 1 60); do [ -f "$ENDPOINT" ] && return 0; sleep 0.5; done
  echo "REAPER did not write $ENDPOINT"; exit 1
}
stop_reaper() {
  local pid; pid=$(isolated_pid)
  [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
  sleep 1
}

echo "== Part 1: REAPER restarted $cycles times under one connected client =="
stop_reaper; start_reaper
watch_log=$(mktemp)
example link_watch "$ENDPOINT" $((cycles * 25 + 15)) >"$watch_log" 2>&1 &
watcher=$!
sleep 5
for cycle in $(seq 1 "$cycles"); do
  echo "-- cycle $cycle: quit REAPER"; stop_reaper
  sleep 4
  echo "-- cycle $cycle: start REAPER"; start_reaper
  sleep 8
done
wait "$watcher" || true
cat "$watch_log"
connections=$(tail -1 "$watch_log" | cut -d' ' -f1)
[ "$connections" -ge $((cycles + 1)) ] || { echo "FAIL: expected ${cycles}+1 connections, got $connections"; exit 1; }

echo "== Part 2: client killed while REAPER plays; the hand-over must still happen =="
for cycle in $(seq 1 "$cycles"); do
  stop_reaper; start_reaper; sleep 3
  : >"$R/RC2/journal.log"
  # Song 1 is 0 to 30 s: start it, jump to 26 s (a seek while playing always continues), then kill the client at once.
  ./target/release/examples/send_command "$ENDPOINT" play "seek:26" >/dev/null 2>&1 &
  client=$!
  sleep 4; kill -9 "$client" 2>/dev/null || true
  echo "-- cycle $cycle: client killed, REAPER plays alone"
  sleep 8
  grep -q HandOver "$R/RC2/journal.log" || { echo "FAIL: no hand-over in the journal"; cat "$R/RC2/journal.log"; exit 1; }
  grep HandOver "$R/RC2/journal.log"
done
stop_reaper
echo "chaos restarts: ok"
