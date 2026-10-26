#!/usr/bin/env bash
# The extension update against a real REAPER (macOS): with an older extension loaded, the update asks
# REAPER to quit, puts the new extension in place and starts REAPER again with the same isolated
# settings. Checks that the first REAPER is gone, that the installed file changed, and that a new REAPER
# started and wrote a fresh `endpoint.json` (a new token, so the new extension is the one answering).
#
# Run it through smoke_extension.sh, which starts the isolated REAPER with the OLD extension:
#   UPDATE_REAPER=<REAPER program> UPDATE_BUNDLED=<the new extension> \
#     testing/scripts/smoke_extension.sh <REAPER program> <old extension> testing/scripts/update_live_check.sh
# Needs: cargo build --release -p installer --examples
# The update quits REAPER by its name, which would also close a REAPER you are using, so this refuses to
# run while any other REAPER is open. The new REAPER is closed again at the end.
set -euo pipefail
cd "$(dirname "$0")/../.."

resource=${REAPER_RESOURCE:?run this through smoke_extension.sh}
old_pid=${REAPER_PID:?run this through smoke_extension.sh}
reaper=${UPDATE_REAPER:?set UPDATE_REAPER to the REAPER program}
bundled=${UPDATE_BUNDLED:?set UPDATE_BUNDLED to the new extension}
example=target/release/examples/update_live
[ -x "$example" ] || { echo "build it first: cargo build --release -p installer --examples"; exit 1; }

others=$(pgrep -x REAPER | grep -v -x "$old_pid" || true)
[ -z "$others" ] || { echo "REFUSED: another REAPER is running (pid $others); quit it first, the update would close it too"; exit 1; }

installed=$(ls "$resource"/UserPlugins/reaper_rc2-* | grep -v '\.stamp$' | head -1)
endpoint=$resource/RC2/endpoint.json
before=$(cat "$endpoint")
cp "$installed" "$resource/old-extension"

echo "== update (REAPER pid $old_pid is running with the old extension)"
"$example" "$reaper" "$resource" "$bundled"

new_pid=""
for _ in $(seq 1 120); do
  candidate=$(pgrep -x REAPER | head -1 || true)
  if [ -n "$candidate" ] && [ "$candidate" != "$old_pid" ] && [ -f "$endpoint" ] && [ "$(cat "$endpoint")" != "$before" ]; then
    new_pid=$candidate
    break
  fi
  sleep 0.5
done
trap '[ -z "$new_pid" ] || kill "$new_pid" 2>/dev/null || true' EXIT

kill -0 "$old_pid" 2>/dev/null && { echo "FAIL: the first REAPER (pid $old_pid) is still running"; exit 1; }
cmp -s "$installed" "$resource/old-extension" && { echo "FAIL: the installed extension did not change"; exit 1; }
[ -n "$new_pid" ] || { echo "FAIL: REAPER did not start again with a new endpoint.json"; exit 1; }
ps -p "$new_pid" -o command= | grep -q -- "-cfgfile $resource/reaper.ini" || { echo "FAIL: the new REAPER does not use the isolated settings"; exit 1; }
echo "OK: the first REAPER quit, the extension was replaced, REAPER (pid $new_pid) started again and its extension answers"
