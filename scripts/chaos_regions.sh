#!/usr/bin/env bash
# Chaos test (WP 7.2): regions edited while a song plays, and the tick time with 1000 regions.
# Uses only the isolated REAPER in .dev/reaper-test and the probe build of the extension, which
# moves and renames a region the way the Region Manager does (`region-edit` in probe-command).
# Needs .dev/projects/regions-1000.rpp (written by scripts/chaos_project.sh) and the screen unlocked.
set -euo pipefail
cd "$(dirname "$0")/.."
source ~/.cargo/env
R="$PWD/.dev/reaper-test"
SAMPLE="$PWD/.dev/projects/large-set-sample.rpp"
HUGE="$PWD/.dev/projects/regions-1000.rpp"
ENDPOINT="$R/RC2/endpoint.json"
[ -d "$R" ] && [ -f "$SAMPLE" ] && [ -f "$HUGE" ] || { echo "missing $R, $SAMPLE or $HUGE"; exit 1; }
cargo build --release -p reaper-extension --features probe
cargo build --release -p link --examples
cp target/release/libreaper_extension.dylib "$R/UserPlugins/reaper_rc2-arm64.dylib"
example() { ./target/release/examples/"$1" "${@:2}"; }

isolated_pid() { pgrep -f "REAPER.*-cfgfile $R/reaper.ini" | head -1 || true; }
start_reaper() {
  rm -f "$ENDPOINT" "$R/RC2/running" "$R/RC2/faulted"
  : >"$R/RC2/extension.log"
  open -n -a /Applications/REAPER.app --args -cfgfile "$R/reaper.ini" "$1"
  for _ in $(seq 1 120); do [ -f "$ENDPOINT" ] && break; sleep 0.5; done
  [ -f "$ENDPOINT" ] || { echo "REAPER did not write $ENDPOINT"; exit 1; }
  sleep 4
}
stop_reaper() {
  local pid; pid=$(isolated_pid)
  [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
  sleep 1
}
probe() { echo "$*" >"$R/RC2/probe-command"; sleep 1; }
trap stop_reaper EXIT

echo "== Region 2 moved and renamed while song 1 plays =="
stop_reaper; start_reaper "$SAMPLE"
: >"$R/RC2/journal.log"
edit() {
  example send_command "$ENDPOINT" play seek:20 >/dev/null 2>&1; sleep 2
  probe region-edit 2 40 70 Moved and renamed
  sleep 12
}
log=$(mktemp)
example link_watch "$ENDPOINT" 20 >"$log" 2>&1 &
watcher=$!
sleep 2; edit; wait "$watcher" || true
cat "$log"
echo "-- journal"; cat "$R/RC2/journal.log"
grep -q "Moved and renamed" "$log" || { echo "FAIL: the catalog never showed the new name"; exit 1; }
if grep -q HandOverCompleted "$R/RC2/journal.log"; then
  echo "region edit: the hand-over still happened"
else
  echo "FINDING: no hand-over after the region edit; the performance started over (phase Idle) while REAPER kept playing"
fi

echo "== Tick time with 1000 regions =="
stop_reaper; start_reaper "$HUGE"
example send_command "$ENDPOINT" play seek:26 >/dev/null 2>&1
sleep 25
grep "slowest" "$R/RC2/extension.log" | tail -3
grep -q "over budget 0" <(grep "slowest" "$R/RC2/extension.log" | tail -1) || echo "NOTE: ticks went over the 5 ms budget"
echo "chaos regions: done"
