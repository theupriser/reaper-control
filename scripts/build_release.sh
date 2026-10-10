#!/usr/bin/env bash
# Builds everything a runnable app needs, in the right order, so the extension in dist/ is never older
# than the app: the UI (the app embeds ui/dist), the extension, its copy in dist/, then the app with
# custom-protocol (without it the window loads the dev address and stays white).
# usage: scripts/build_release.sh   (from the repository root)
set -euo pipefail

pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build
cargo build --release -p reaper-extension
scripts/package_extension.sh
cargo build --release -p app --features custom-protocol
