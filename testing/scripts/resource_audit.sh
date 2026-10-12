#!/usr/bin/env bash
# macOS: idle CPU and memory of the app, the isolated REAPER and its extension (PLAN WP 7.5).
# usage: resource_audit.sh <resource directory with reaper.ini> <project> [seconds, default 180]
# Build first: pnpm --dir ui build && cargo build --release -p app -p reaper-extension --features app/custom-protocol (without custom-protocol the window stays white).
# The resource directory is a throwaway copy (git-ignored .dev/); this never touches the real REAPER.
set -euo pipefail

resource=$(cd "$1" && pwd)
project=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
seconds=${3:-180}
root=$(pwd)
config=$(mktemp -d)

mkdir -p "$resource/UserPlugins"
cp "$root/target/release/libreaper_extension.dylib" "$resource/UserPlugins/reaper_rc2-arm64.dylib"
rm -f "$resource/RC2/running" "$resource/RC2/faulted" "$resource/RC2/endpoint.json"

open -n -a /Applications/REAPER.app --args -cfgfile "$resource/reaper.ini" "$project"
RC2_DIRECTORY="$resource/RC2" RC2_CONFIG_DIRECTORY="$config" "$root/target/release/app" >"$config/app.log" 2>&1 &
app=$!
cleanup() { kill "$app" 2>/dev/null || true; pkill -f "$resource/reaper.ini" || true; rm -f "$resource/RC2/running" "$resource/RC2/faulted"; rm -rf "$config"; }
trap cleanup EXIT

for _ in $(seq 1 60); do [ -f "$resource/RC2/endpoint.json" ] && break; sleep 0.5; done
[ -f "$resource/RC2/endpoint.json" ] || { echo "FAIL: no endpoint.json after 30 s"; exit 1; }
sleep 20   # settle: window, connection, first catalog

reaper=$(pgrep -f "$resource/reaper.ini" | head -1)
echo "app pid $app, reaper pid $reaper, sampling $seconds s every 5 s"
echo "time   app_cpu% app_rss_MB  reaper_cpu% reaper_rss_MB"
end=$((SECONDS + seconds))
while [ $SECONDS -lt $end ]; do
  a=$(ps -o %cpu=,rss= -p "$app"); r=$(ps -o %cpu=,rss= -p "$reaper")
  echo "$a $r" | awk -v t="$(date +%T)" '{printf "%s %8.1f %10.1f %12.1f %13.1f\n", t, $1, $2/1024, $3, $4/1024}'
  sleep 5
done
