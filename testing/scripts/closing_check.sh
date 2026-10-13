#!/usr/bin/env bash
# A connected client must hear "closing" when REAPER quits the way a person quits it, and must hear
# nothing of the kind when REAPER is killed. It also checks that the quit removes the `running`
# file and that the kill leaves it.
# Runs after smoke_extension.sh starts REAPER (it sets REAPER_RESOURCE and REAPER_PID). It ends
# REAPER, so it needs a smoke run of its own. Needs the link_watch example (cargo build --release
# -p link --examples). CLOSING_MODE=quit (default) or kill.
set -euo pipefail
cd "$(dirname "$0")/../.."

resource=${REAPER_RESOURCE:?start REAPER with smoke_extension.sh}
pid=${REAPER_PID:?start REAPER with smoke_extension.sh}
mode=${CLOSING_MODE:-quit}
endpoint=$resource/RC2/endpoint.json
watcher=target/release/examples/link_watch
[ -f "$endpoint" ] || { echo "no $endpoint"; exit 1; }
[ -x "$watcher" ] || [ -x "$watcher.exe" ] || { echo "build the examples first: cargo build --release -p link --examples"; exit 1; }
[ -f "$resource/RC2/running" ] || { echo "FAIL: no running file while REAPER runs"; exit 1; }

output=$resource/closing_watch.out
"$watcher" "$endpoint" 20 > "$output" 2>&1 &
watching=$!
for _ in $(seq 1 40); do grep -q "connected" "$output" && break; sleep 0.25; done
grep -q "connected" "$output" || { echo "FAIL: the client never connected"; cat "$output"; exit 1; }

echo "== $mode REAPER (pid $pid)"
case "$mode:$(uname -s)" in
  kill:MINGW*|kill:MSYS*|kill:CYGWIN*) taskkill //F //PID "$(cat "/proc/$pid/winpid")" > /dev/null ;;
  kill:*) kill -9 "$pid" ;;
  quit:MINGW*|quit:MSYS*|quit:CYGWIN*)
    # without /F, taskkill closes the main window the way the close button does
    taskkill //PID "$(cat "/proc/$pid/winpid")" > /dev/null ;;
  quit:Darwin)
    script=$(mktemp "${RUNNER_TEMP:-/tmp}/quit.XXXXXX")
    mv "$script" "$script.swift"
    printf 'import AppKit\nif let app = NSRunningApplication(processIdentifier: %s) { _ = app.terminate() }\n' "$pid" > "$script.swift"
    swift "$script.swift" ;;
esac

for _ in $(seq 1 40); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
sleep 2
wait "$watching" || true
echo "--- what the client heard"; cat "$output"
echo "--- exit.diag"; cat "$resource/RC2/exit.diag" 2>&1 || true
echo "--- running file: $(ls "$resource/RC2/running" 2>&1)"

if [ "$mode" = quit ]; then
  grep -q "REAPER says it is closing" "$output" || { echo "FAIL: no goodbye after a clean quit"; exit 1; }
  [ ! -f "$resource/RC2/running" ] || { echo "FAIL: the running file is still there after a clean quit"; exit 1; }
  echo "OK: a clean quit said goodbye and removed the running file"
else
  ! grep -q "REAPER says it is closing" "$output" || { echo "FAIL: a killed REAPER said goodbye"; exit 1; }
  [ -f "$resource/RC2/running" ] || { echo "FAIL: a kill removed the running file"; exit 1; }
  echo "OK: a kill said nothing and left the running file"
fi
