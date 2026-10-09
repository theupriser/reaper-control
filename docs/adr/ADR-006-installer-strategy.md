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

## Not yet measured
Whether a downloaded ad-hoc signed dylib loads in REAPER after the quarantine step; what SmartScreen and antivirus tools say about the unsigned DLL; the macOS launcher form. WP 1.4 stays 🟡 until these are tried.

## Undo if
A Developer ID or certificate becomes available (then sign and notarize, WP 8.1), or users cannot get past the OS warning without help.
