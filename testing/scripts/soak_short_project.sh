#!/usr/bin/env bash
# Writes the project for soak_short.sh: regions "Song 01".. of the given length, 5 s apart, no audio.
# usage: soak_short_project.sh <output .rpp> [songs, default 8] [seconds per song, default 6]
set -euo pipefail

output=$1
songs=${2:-8}
length=${3:-6}

{
  echo '<REAPER_PROJECT 0.1 "7.82/CI" 1759780000 0'
  echo '  RIPPLE 0'
  echo '  TEMPO 120 4 4 0'
  for number in $(seq 1 "$songs"); do
    start=$(( (number - 1) * (length + 5) ))
    id=$(printf '{C2000000-0000-0000-0000-%012d}' "$number")
    printf '  MARKER %d %d "Song %02d" 1 0 1 B %s\n' "$number" "$start" "$number" "$id"
    printf '  MARKER %d %d "" 1 0 1 B %s\n' "$number" "$((start + length))" "$id"
  done
  echo '>'
} > "$output"
