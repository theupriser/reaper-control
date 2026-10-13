#!/usr/bin/env bash
# Checks that a project that asks for it gets the app started by the extension, once, and that a
# project that does not ask starts nothing. A stand-in program takes the app's place and writes
# one line to a marker file each time it is started.
# Run by smoke_extension.sh while REAPER is up (REAPER_RESOURCE is set):
#   SMOKE_PROJECT=testing/data/projects/autostart-sample.rpp \
#     testing/scripts/smoke_extension.sh <REAPER> <extension> testing/scripts/autostart_check.sh
# AUTOSTART_EXPECT=started (default) is for a project with the switch on, AUTOSTART_EXPECT=none for
# one with it off (use s1-sample.rpp).
set -euo pipefail

expect=${AUTOSTART_EXPECT:-started}
directory=$REAPER_RESOURCE/RC2
marker=$directory/autostart-marker
rm -f "$marker"

case "$(uname -s)" in
  Darwin)
    application=$directory/StandIn.app
    mkdir -p "$application/Contents/MacOS"
    printf '#!/bin/sh\necho started >> "%s"\n' "$marker" > "$application/Contents/MacOS/StandIn"
    chmod +x "$application/Contents/MacOS/StandIn"
    cat > "$application/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>StandIn</string>
<key>CFBundleIdentifier</key><string>nl.upriser.reaper-control.standin</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>
PLIST
    location=$application ;;
  MINGW*|MSYS*|CYGWIN*)
    program=$directory/stand-in.cmd
    printf '@echo started>>"%s"\r\n' "$(cygpath -w "$marker")" > "$program"
    location=$(cygpath -w "$program") ;;
  *) echo "autostart_check.sh runs on macOS and Windows"; exit 1 ;;
esac
printf '%s\n' "$location" > "$directory/application"
begun=$(date +%s)

started() { [ -f "$marker" ] && wc -l < "$marker" | tr -d ' ' || echo 0; }

if [ "$expect" = started ]; then
  for _ in $(seq 1 60); do [ "$(started)" -ge 1 ] && break; sleep 0.5; done
  [ "$(started)" -ge 1 ] || { echo "FAIL: the app was not started"; cat "$directory/extension.log"; exit 1; }
  echo "the app was started $(( $(date +%s) - begun )) s after the location was written"
  sleep 12
  [ "$(started)" -eq 1 ] || { echo "FAIL: the app was started $(started) times, not once"; exit 1; }
  grep -q "autostart: started" "$directory/extension.log" || { echo "FAIL: the log does not say the app was started"; exit 1; }
  echo "OK: the project asked for the app, it was started once"
else
  sleep 20
  [ "$(started)" -eq 0 ] || { echo "FAIL: the app was started by a project that did not ask"; exit 1; }
  echo "OK: a project that does not ask starts nothing"
fi
