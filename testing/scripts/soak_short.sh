#!/usr/bin/env bash
# Short soak: the song set from soak_short_project.sh (8 songs of 6 s) plays from start to end in the
# running isolated REAPER with a client connected. A hand-over skips the 5 s gap, so hand-over N must
# complete 6 s * N after the play command. About a minute. The full 40-song soak is soak.sh.
# Runs after smoke_extension.sh (REAPER_RESOURCE is set) with SMOKE_PROJECT pointing at the project.
# Needs: cargo build --release -p link --examples
set -euo pipefail
cd "$(dirname "$0")/../.."

resource=${REAPER_RESOURCE:?run this through smoke_extension.sh}
songs=${SOAK_SONGS:-8}
length=${SOAK_SECONDS_PER_SONG:-6}
tolerance=${SOAK_TOLERANCE:-0.5}
endpoint=$resource/RC2/endpoint.json
journal=$resource/RC2/journal.log

extension=""
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) extension=.exe; python=python ;; *) python=python3 ;; esac
watch=target/release/examples/link_watch$extension
send=target/release/examples/send_command$extension

endpoint_argument=$endpoint
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) endpoint_argument=$(cygpath -w "$endpoint") ;; esac

"$watch" "$endpoint_argument" $((songs * length + 25)) > "$resource/link_watch.log" 2>&1 &
watcher=$!
sleep 2
"$send" "$endpoint_argument" play
wait "$watcher" || true

echo "== link_watch saw"; cat "$resource/link_watch.log"
echo "== journal"; ls -l "$journal"

"$python" -I - "$journal" "$songs" "$length" "$tolerance" <<'PY'
import json, sys
path, songs, length, tolerance = sys.argv[1], int(sys.argv[2]), float(sys.argv[3]), float(sys.argv[4])
events = [json.loads(line) for line in open(path)]
start = next(e["timestamp"] for e in events if e["event"]["kind"] == "PerformanceStarted")
completed = [e["timestamp"] - start for e in events if e["event"]["kind"] == "HandOverCompleted"]
started = sum(1 for e in events if e["event"]["kind"] == "HandOverStarted")
expected = songs - 1
errors = [t - length * (n + 1) for n, t in enumerate(completed)]
print(f"hand-overs: {started} started, {len(completed)} completed (expected {expected})")
if errors:
    ordered = sorted(errors)
    print(f"timing error against the plan: min {ordered[0]:+.3f} s, median {ordered[len(ordered)//2]:+.3f} s, max {ordered[-1]:+.3f} s")
bad = len(completed) != expected or started != expected or any(abs(e) > tolerance for e in errors)
sys.exit("FAIL: missed or late hand-over" if bad else 0)
PY
echo "short soak: ok"
