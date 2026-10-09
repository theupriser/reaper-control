# ADR-006: Installer strategy

Status: **Proposed** (the owner decided the four points on 2026-10-09; nothing is measured yet). Date: 2026-10-09. Related: SPEC installer section, PLAN WP 1.4, 3.10, 4.14, 4.15, 8.1, risks R3, R16, R17.

## Question
How does the app put the extension into REAPER, on macOS arm64 and Windows x64, without an Apple Developer ID or a Windows certificate?

## Decision
1. **macOS: ad-hoc signing.** The extension and the app are signed ad-hoc. Developer ID and notarization wait until the owner has an account (WP 8.1). The install guide explains the one-time Gatekeeper step (right-click Open, or `xattr -dr com.apple.quarantine`).
2. **Windows: unsigned.** The install guide explains the SmartScreen warning. A certificate waits for WP 8.1.
3. **Folder choice, never silent.** The installer lists the REAPER resource folders it finds (standard and the ones it is given) and asks which one may receive the extension; the user can choose another folder (portable installs). Nothing is written without that yes. The app also takes `--reaper-folder <path>` so one shortcut per REAPER works: a `.lnk` with the argument on Windows; on macOS a shortcut cannot carry arguments, so a small `.command` file or launcher runs `open -a "Reaper Control" --args --reaper-folder <path>`.
4. **Update with REAPER closed, only when the user asks.** No staged update. If REAPER is running, the app asks the user to close it first and waits for it to quit (Windows locks the DLL). It never updates by itself, and never offers an update during a show (R17).

## Cost
- Users see one OS warning on each system until signing exists.
- A user with two REAPERs has to pick the folder (or use the shortcut argument) for each.
- No swap-at-next-start code to write or test; the update flow is "close REAPER, copy, start REAPER".

## Measured on macOS (2026-10-09, isolated REAPER, Apple Silicon)
The extension dylib was copied into `UserPlugins` with the quarantine attribute set, as a browser download would leave it.

| Case | Result |
|---|---|
| Quarantined, linker-signed (ad-hoc) | Gatekeeper dialog ("not opened, may harm your Mac"); in the repeat run the extension did not load |
| Quarantined, re-signed with `codesign --force -s -` | Same dialog, extension did not load |
| Signature removed, quarantined | Did not load |
| Quarantine removed by the installer (`xattr -d com.apple.quarantine`), ad-hoc signed | Loads, link listens, ticks run |

Decision: the macOS installer copies the dylib, ensures an ad-hoc signature (`codesign --force -s -`), and removes `com.apple.quarantine` from the copy. The first run on a quarantined file once appeared to load before the dialog showed; do not rely on it.

## The launcher argument (2026-10-09)
The app reads `--reaper-folder <path>` (or `--reaper-folder=<path>`) at start and looks for that REAPER's `RC2/endpoint.json` (`RC2_DIRECTORY` still wins, for isolated test setups). Tried live: the app started with only `--reaper-folder .dev/reaper-test` connected to the isolated REAPER ("Connected, extension 0.0.0"). The macOS shortcut form `open -a "Reaper Control" --args --reaper-folder <path>` relies on `open` passing what follows `--args` to the program; the same mechanism starts the isolated REAPER in our tests (`open -n -a REAPER.app --args -cfgfile ...`). The `.command` file itself is not produced yet: there is no app bundle to open until the installer build (WP 8.1).

## Not yet measured
What SmartScreen and antivirus tools say about the unsigned DLL (needs a real Windows PC, owner). WP 1.4 stays 🟡 until that is tried.

## Undo if
A Developer ID or certificate becomes available (then sign and notarize, WP 8.1), or users cannot get past the OS warning without help.
