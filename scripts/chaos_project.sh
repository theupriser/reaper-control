#!/usr/bin/env bash
# Chaos test (WP 7.2): project switch mid-song, markers inserted live, and a 1000-region project.
# Uses only the isolated REAPER in .dev/reaper-test, driven through its own menus (System Events).
# Needs: cargo build --release -p reaper-extension && scripts/package_extension.sh, and the screen unlocked.
set -euo pipefail
cd "$(dirname "$0")/.."
source ~/.cargo/env
R="$PWD/.dev/reaper-test"
SAMPLE="$PWD/.dev/projects/large-set-sample.rpp"
HUGE="$PWD/.dev/projects/regions-1000.rpp"
ENDPOINT="$R/RC2/endpoint.json"
[ -d "$R" ] && [ -f "$SAMPLE" ] || { echo "missing $R or $SAMPLE"; exit 1; }
cp dist/reaper_rc2-arm64.dylib "$R/UserPlugins/reaper_rc2-arm64.dylib"
cargo build --release -p link --examples
example() { ./target/release/examples/"$1" "${@:2}"; }

python3 -I - "$HUGE" <<'PY'
import sys
lines = ['<REAPER_PROJECT 0.1 "7.82/macOS-arm64" 1759780000 0', "  RIPPLE 0", "  TEMPO 120 4 4 0"]
for n in range(1, 1001):
    start = (n - 1) * 35
    guid = "{C3000000-0000-0000-0000-%012d}" % n
    lines.append(f'  MARKER {n} {start} "Song {n:04d}" 1 0 1 B {guid}')
    lines.append(f'  MARKER {n} {start + 30} "" 1 0 1 B {guid}')
lines.append(">")
open(sys.argv[1], "w").write("\n".join(lines) + "\n")
PY

isolated_pid() { pgrep -f "REAPER.*-cfgfile $R/reaper.ini" | head -1 || true; }
start_reaper() {
  rm -f "$ENDPOINT" "$R/RC2/running" "$R/RC2/faulted"
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
menu() {
  osascript -e "tell application \"System Events\" to tell (first process whose unix id is $(isolated_pid)) to click menu item \"$2\" of menu 1 of menu bar item \"$1\" of menu bar 1" >/dev/null
}
watch_during() { # seconds, then commands run in the foreground while the watcher prints
  local log; log=$(mktemp)
  : >"$R/RC2/journal.log"
  example link_watch "$ENDPOINT" "$1" >"$log" 2>&1 &
  local watcher=$!
  sleep 2
  "${@:2}"
  wait "$watcher" || true
  cat "$log"
}

echo "== Project switch mid-song =="
stop_reaper; start_reaper "$SAMPLE"
switch() { example send_command "$ENDPOINT" play seek:20 >/dev/null 2>&1; sleep 1; menu File "New project tab"; sleep 5; }
watch_during 14 switch | tee /tmp/chaos_switch.log
grep -q "ProjectChanged" /tmp/chaos_switch.log || { echo "FAIL: no ProjectChanged"; exit 1; }
grep -q "0 songs" /tmp/chaos_switch.log || { echo "FAIL: the new tab still shows the old songs"; exit 1; }
tail -2 /tmp/chaos_switch.log | head -1 | grep -q "Idle" || { echo "FAIL: not Idle after the switch"; exit 1; }

echo "== Markers inserted live while playing =="
stop_reaper; start_reaper "$SAMPLE"
insert() {
  example send_command "$ENDPOINT" play seek:20 >/dev/null 2>&1; sleep 2
  for _ in 1 2 3; do menu Insert Marker; sleep 1; done
  sleep 8
}
watch_during 16 insert | tee /tmp/chaos_markers.log
grep -q "3 cues" /tmp/chaos_markers.log || { echo "FAIL: the catalog never showed 3 cues"; exit 1; }
grep -q HandOverCompleted "$R/RC2/journal.log" || { echo "FAIL: no hand-over after the edits"; exit 1; }

echo "== 1000 regions =="
stop_reaper
started=$(date +%s)
start_reaper "$HUGE"
echo "REAPER up with 1000 songs after $(( $(date +%s) - started )) s"
play() { example send_command "$ENDPOINT" play seek:26 >/dev/null 2>&1; sleep 7; }
watch_during 12 play | tee /tmp/chaos_huge.log
grep -q "1000 songs" /tmp/chaos_huge.log || { echo "FAIL: the catalog does not hold 1000 songs"; exit 1; }
grep -q HandOverCompleted "$R/RC2/journal.log" || { echo "FAIL: no hand-over with 1000 regions"; exit 1; }
stop_reaper
echo "chaos project: ok"
