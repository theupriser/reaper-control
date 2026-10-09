#!/usr/bin/env bash
# Copies the release build of the extension to dist/ under the name REAPER loads and writes
# SHA256SUMS next to it. Run `cargo build --release -p reaper-extension` first.
set -euo pipefail

case "$(uname -s)" in
  Darwin) built=target/release/libreaper_extension.dylib; packaged=reaper_rc2-arm64.dylib ;;
  MINGW*|MSYS*|CYGWIN*) built=target/release/reaper_extension.dll; packaged=reaper_rc2-x64.dll ;;
  *) echo "unsupported platform $(uname -s)" >&2; exit 1 ;;
esac

mkdir -p dist
cp "$built" "dist/$packaged"
cd dist
if command -v sha256sum >/dev/null; then sha256sum "$packaged" > "$packaged.sha256"; else shasum -a 256 "$packaged" > "$packaged.sha256"; fi
cat "$packaged.sha256"
