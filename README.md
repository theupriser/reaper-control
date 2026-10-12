# Reaper Control

A stage controller for the [REAPER](https://www.reaper.fm) DAW: it plays a setlist live, hands over from song to song on time, and stays out of the way.

A native REAPER extension runs the show inside REAPER. The app is a remote control and display, so if the app crashes or is restarted, playback carries on.

## What it does

- **Setlists** stored in the REAPER project, edited in the app.
- **Hand-over** from song to song at the end of the song, with no gap.
- **Hard stop** with the `!1008` marker: the set stops at the end of the song. `!length:N` and `!bpm:N` markers are understood too.
- **Count-in** when you jump to a cue, using REAPER's own count-in.
- **Performer screen** with song title, time left, set time, next song, clock and toggles.
- **Keyboard and MIDI** control (pads and controllers, with hot-plug).
- **Setup wizard** that finds REAPER, installs the extension and waits until it is connected.
- **Diagnostics**: logs, a journal of every performance event and an export for bug reports.

The full list is in the [release notes](.github/release-notes/v2.0.0.md).

## Install

Platforms: macOS on Apple Silicon (arm64) and Windows x64. Tested with REAPER 7.82. Intel Macs and Linux are not supported.

1. Download the installer for your system from the [latest release](../../releases/latest): the `.dmg` for macOS or the `-setup.exe` for Windows.
2. The app is **not code-signed yet**. On macOS, right-click the app and choose Open the first time. On Windows, SmartScreen may warn: choose More info, then Run anyway.
3. Start Reaper Control. The setup wizard finds REAPER, installs the extension into its `UserPlugins` folder and asks you to restart REAPER. When REAPER is back, the app connects by itself.

If REAPER did not shut down cleanly (a crash), the extension starts disabled ("safe mode") and the app says so. For now, remove the `faulted` and `running` files in the `RC2` folder of the REAPER resource path and restart REAPER.

## Build from source

You need Rust (stable), Node 22 and pnpm 10.

```
pnpm --dir ui install && pnpm --dir ui build
cargo test --workspace
scripts/build_release.sh
```

`scripts/build_release.sh` builds the UI, the extension and the app in order. See `AGENTS.md` for the code rules and the project layout.

## Contributing

Working notes for contributors and AI agents are in `AGENTS.md`.

## Licence

Free to use, change and share; selling it is not allowed. See `LICENSE` (MIT with the Commons Clause).
