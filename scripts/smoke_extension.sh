#!/usr/bin/env bash
# macOS: loads a packaged extension in an isolated REAPER and checks that it starts and listens.
# usage: smoke_extension.sh <packaged .dylib> <resource directory with reaper.ini> <project>
# The resource directory is a throwaway copy (git-ignored .dev/); this never touches the real REAPER.
set -euo pipefail

extension=$1; resource=$2; project=$3
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

mkdir -p "$resource/UserPlugins"
cp "$extension" "$resource/UserPlugins/reaper_rc2-arm64.dylib"
rm -f "$resource/RC2/running" "$resource/RC2/faulted" "$resource/RC2/endpoint.json" "$resource/RC2/extension.log"

open -n -a /Applications/REAPER.app --args -cfgfile "$resource/reaper.ini" "$project"
trap 'pkill -f "$resource/reaper.ini" || true; rm -f "$resource/RC2/running" "$resource/RC2/faulted"' EXIT

for _ in $(seq 1 60); do
  [ -f "$resource/RC2/endpoint.json" ] && break
  sleep 0.5
done
[ -f "$resource/RC2/endpoint.json" ] || { echo "FAIL: no endpoint.json after 30 s"; cat "$resource/RC2/extension.log" 2>/dev/null; exit 1; }

cat "$resource/RC2/extension.log"
grep -q "started, version $version," "$resource/RC2/extension.log" || { echo "FAIL: log does not name version $version"; exit 1; }
echo "OK: extension $version started and is listening"
