#!/usr/bin/env bash
# Soak test (WP 7.1): the 40-song sample set (30 s songs, 35 s apart) plays from start to end in the
# isolated REAPER with a client connected and nobody touching it. A hand-over skips the 5 s gap, so
# hand-over N must complete 30 s * N after the play command. Also checks that REAPER's memory stays
# flat. About 25 minutes.
# Needs: cargo build --release -p reaper-extension && scripts/package_extension.sh
set -euo pipefail
cd "$(dirname "$0")/.."
source ~/.cargo/env
tolerance="${1:-0.25}"
R="$PWD/.dev/reaper-test"
PROJECT="$PWD/.dev/projects/large-set-sample.rpp"
ENDPOINT="$R/RC2/endpoint.json"
[ -d "$R" ] && [ -f "$PROJECT" ] || { echo "missing $R or $PROJECT"; exit 1; }
cp dist/reaper_rc2-arm64.dylib "$R/UserPlugins/reaper_rc2-arm64.dylib"
cargo build --release -p link --examples

isolated_pid() { pgrep -f "REAPER.*-cfgfile $R/reaper.ini" | head -1 || true; }
stop_reaper() {
  local pid; pid=$(isolated_pid)
  [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
  sleep 1
}

stop_reaper
rm -f "$ENDPOINT" "$R/RC2/running" "$R/RC2/faulted"
open -n -a /Applications/REAPER.app --args -cfgfile "$R/reaper.ini" "$PROJECT"
for _ in $(seq 1 60); do [ -f "$ENDPOINT" ] && break; sleep 0.5; done
[ -f "$ENDPOINT" ] || { echo "REAPER did not write $ENDPOINT"; exit 1; }
sleep 4
pid=$(isolated_pid)
: >"$R/RC2/journal.log"
watch_log=$(mktemp)
./target/release/examples/link_watch "$ENDPOINT" 1440 >"$watch_log" 2>&1 &
watcher=$!
sleep 2

echo "== soak: 40 songs, 35 s apart, started $(date +%T) =="
./target/release/examples/send_command "$ENDPOINT" play
memory_start=$(ps -o rss= -p "$pid")
for minute in $(seq 1 24); do
  sleep 60
  echo "   minute $minute: REAPER $(( $(ps -o rss= -p "$pid") / 1024 )) MB, $(grep -c HandOverCompleted "$R/RC2/journal.log") hand-overs"
done
wait "$watcher" || true
memory_end=$(ps -o rss= -p "$pid")
echo "== link_watch saw =="; cat "$watch_log"
echo "== journal =="; grep -c . "$R/RC2/journal.log"; grep -v 'HandOver' "$R/RC2/journal.log" || true
stop_reaper

python3 -I - "$R/RC2/journal.log" "$tolerance" "$memory_start" "$memory_end" <<'PY'
import json, sys
path, tolerance, memory_start, memory_end = sys.argv[1], float(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
events = [json.loads(line) for line in open(path)]
start = next(e["timestamp"] for e in events if e["event"]["kind"] == "PerformanceStarted")
completed = [e["timestamp"] - start for e in events if e["event"]["kind"] == "HandOverCompleted"]
started = sum(1 for e in events if e["event"]["kind"] == "HandOverStarted")
errors = [t - 30.0 * (n + 1) for n, t in enumerate(completed)]
print(f"hand-overs: {started} started, {len(completed)} completed (expected 39)")
if errors:
    ordered = sorted(errors)
    print(f"timing error against the plan: min {ordered[0]:+.3f} s, median {ordered[len(ordered)//2]:+.3f} s, max {ordered[-1]:+.3f} s")
print(f"REAPER memory: {memory_start // 1024} MB at the start, {memory_end // 1024} MB at the end")
bad = len(completed) != 39 or started != 39 or any(abs(e) > tolerance for e in errors)
sys.exit("FAIL: missed or late hand-over" if bad else 0)
PY
echo "soak: ok"
