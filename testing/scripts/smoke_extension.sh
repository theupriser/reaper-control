#!/usr/bin/env bash
# Loads a packaged extension in an isolated REAPER (macOS or Windows) and checks that it starts and listens.
# usage: smoke_extension.sh <REAPER executable> <packaged extension> [check script...]
# Each check script runs while REAPER is up, with REAPER_RESOURCE set to the resource directory.
# SMOKE_SECONDS (default 60) is how long to wait for the extension to listen; SMOKE_PROJECT is a
# project to open.
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
cp testing/data/reaper/reaper.ini "$resource/reaper.ini"
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) cat testing/data/reaper/windows.ini >> "$resource/reaper.ini" ;; esac

configuration=$resource/reaper.ini
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) configuration=$(cygpath -w "$configuration") ;; esac

project=()
if [ -n "${SMOKE_PROJECT:-}" ]; then
  project=("$SMOKE_PROJECT")
  case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) project=("$(cygpath -w "$SMOKE_PROJECT")") ;; esac
fi

"$reaper" -cfgfile "$configuration" -newinst -nosplash ${project[@]+"${project[@]}"} > "$resource/reaper.out" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; sleep 1; kill -9 "$pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 $((seconds * 2))); do
  [ -f "$resource/RC2/endpoint.json" ] && break
  kill -0 "$pid" 2>/dev/null || break
  sleep 0.5
done

report() {
  echo "--- resource directory"; ls -la "$resource" "$resource/RC2" 2>&1 || true
  echo "--- reaper.ini"; cat "$resource/reaper.ini" 2>&1 || true
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

# On a failure, show what is on the screen (a dialog that blocks REAPER, for instance).
show_screen() {
  echo "--- reaper.ini after the run"; cat "$resource/reaper.ini" 2>&1 || true
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*)
      powershell.exe -NoProfile -Command "Get-Process | Where-Object { \$_.MainWindowTitle } | Select-Object ProcessName, MainWindowTitle | Format-Table -AutoSize | Out-String -Width 200" 2>&1 || true
      powershell.exe -NoProfile -Command "Add-Type -AssemblyName System.Windows.Forms,System.Drawing; \$b=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds; \$i=New-Object System.Drawing.Bitmap \$b.Width,\$b.Height; [System.Drawing.Graphics]::FromImage(\$i).CopyFromScreen(\$b.Location,[System.Drawing.Point]::Empty,\$b.Size); \$i.Save('$(cygpath -w "$PWD")\\screen.png')" 2>&1 || true ;;
    Darwin) screencapture -x "$PWD/screen.png" 2>&1 || true ;;
  esac
}

for check in "$@"; do
  echo "== $check"
  REAPER_RESOURCE=$resource "$check" || { show_screen; exit 1; }
done
