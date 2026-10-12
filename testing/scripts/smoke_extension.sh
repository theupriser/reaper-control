#!/usr/bin/env bash
# Loads a packaged extension in an isolated REAPER (macOS or Windows) and checks that it starts and listens.
# usage: smoke_extension.sh <REAPER executable> <packaged extension> [check script...]
# Each check script runs while REAPER is up, with REAPER_RESOURCE set to the resource directory.
# SMOKE_SECONDS (default 60) is how long to wait for the extension to listen.
# REAPER runs with a throwaway resource directory under $RUNNER_TEMP or /tmp; your own REAPER is never touched.
# On a CI runner REAPER is installed first by the workflow; locally pass /Applications/REAPER.app/Contents/MacOS/REAPER.
set -euo pipefail
cd "$(dirname "$0")/../.."

reaper=$1
extension=$2
seconds=${SMOKE_SECONDS:-60}
shift 2
version=$(sed -n 's/^version = "\(.*\)"/\1/p' crates/reaper-extension/Cargo.toml | head -1)

resource=$(mktemp -d "${RUNNER_TEMP:-/tmp}/reaper-smoke.XXXXXX")
mkdir -p "$resource/UserPlugins"
cp "$extension" "$resource/UserPlugins/$(basename "$extension")"
: > "$resource/reaper.ini"

configuration=$resource/reaper.ini
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) configuration=$(cygpath -w "$configuration") ;; esac

"$reaper" -cfgfile "$configuration" -newinst -nosplash > "$resource/reaper.out" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; sleep 1; kill -9 "$pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 $((seconds * 2))); do
  [ -f "$resource/RC2/endpoint.json" ] && break
  kill -0 "$pid" 2>/dev/null || break
  sleep 0.5
done

report() {
  echo "--- resource directory"; ls -la "$resource" "$resource/RC2" 2>&1 || true
  echo "--- extension.log"; cat "$resource/RC2/extension.log" 2>&1 || true
  echo "--- REAPER output"; cat "$resource/reaper.out" 2>&1 || true
}

if [ ! -f "$resource/RC2/endpoint.json" ]; then
  echo "FAIL: no endpoint.json after $seconds s (REAPER running: $(kill -0 "$pid" 2>/dev/null && echo yes || echo no))"
  report
  exit 1
fi
report
grep -q "started, version $version," "$resource/RC2/extension.log" || { echo "FAIL: the log does not name version $version"; exit 1; }
echo "OK: extension $version started and is listening"

for check in "$@"; do
  echo "== $check"
  REAPER_RESOURCE=$resource "$check"
done
