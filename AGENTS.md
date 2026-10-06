# AGENTS.md — Reaper Control v2

Working notes for AI agents (and humans) on this repo. Read this first, then the docs it points to. Keep it short and accurate: update it when a decision changes.

## What this is
Reaper Control v2: a stage app that controls the REAPER DAW and plays a setlist live. It is a rewrite of v1 (Electron/Svelte/TS, at `~/Projects/Reaper-Control-App`, **read-only reference, never edit it**; if the local copy is not available, the same code is at https://github.com/theupriser/Reaper-Control-App, `main`, read it with `gh api repos/theupriser/Reaper-Control-App/contents/<path> --jq .content | base64 -d` or `gh repo clone` into the scratchpad). Owner: Rick Peters (rickpeters@upriser.nl). Used on stage, so **stability beats features**.

## Source of truth (read in this order)
0. `docs/STATUS.md` — where we are and what is next.
1. `docs/SPEC.md` — what/how (architecture, protocol, state machine, safety, installer, DDD). Draft 2.
2. `docs/PLAN.md` — phases, work packages (WP ids), gates, risks, estimates (~205 focused days).
3. `docs/REVIEW-1.md` — consistency review and why some decisions changed.
4. Design canvas (Claude Design, private): https://claude.ai/artifact/X3zo5xvtRT58fPWjRPtXsH — screens to implement. SPEC §10b–10e lists what is designed and what must be updated for D7.
5. v1 code is the behavioural reference for parity (F1–F18 in SPEC §1). When in doubt about v1 behaviour, read the v1 source, do not guess.

If code and docs disagree, stop and reconcile (fix the doc or the code, and say which).

## Decisions in force (do not re-litigate without the owner)
- **D1** Public release. **D6** v2.0 ships on macOS + Windows only; Linux, auto-updater, website, Dutch, MIDI learn, hold-to-confirm are v2.1 (PLAN §12b).
- **D8** macOS Apple Silicon (arm64) + Windows x64 only. No Intel Macs. Intel REAPER under Rosetta is detected and guided, not supported.
- **D7** Primary companion = **native Rust REAPER extension** (`reaper-rs`), pushing state over a **loopback socket** (127.0.0.1 + token). **No REAPER web interface and no Lua script** in the primary design. Fallback (Lua + web interface + `mlua`) only if Gate 1 rejects the extension: SPEC §2.5.
- **D2** Setlists live in the REAPER project (project ExtState), edited via `SaveSetlist(expectedRev)`; app keeps a restore-only mirror.
- **D3** Companion component required, no "basic mode". **D4** Remote control (tablet) is a stretch (PLAN Phase 9).
- Parity first: **no new performance features** until parity is reached (SPEC §0 lists the allowed additions).
- The **Performer screen keeps the v1 look**; other screens follow the new canvas design.

## Architecture in one screen
```
REAPER ── reaper-extension (Rust cdylib) ──loopback socket (push)── App (Tauri 2: Rust core + Svelte 5 UI)
          • performance core (shared crate)                         • link client, command queue, MIDI, config
          • timer/audio hook, journal, setlists in ExtState         • UI renders state, never infers it
```
- The extension runs the show (hand-overs, hard stops, count-in). The app is a remote control + display. App crash must never interrupt playback.
- One implementation of the performance rules: the Rust `performance` crate, used by the extension (production) and the app (simulator, dry-run, tests).
- Everything REAPER-specific hides behind the `ReaperPort` trait (`ReaperRsAdapter` real, `FakeReaper` for tests).
- Every mutation is a `Command` through one `dispatch()`; UI, keyboard and MIDI all use it (one path per action).

## Vocabulary (binding for code, UI copy, docs; SPEC §14.2)
Song (= region) · Setlist · Entry · Performance · Cue (= marker for navigation) · Directive (`!1008` hard stop, `!length:N`, `!bpm:N`) · Hand-over (never "transition" in UI) · Count-in · Intent vs Command · Link · Extension.
Phases (only vocabulary): `Idle, Playing, Paused, CountingIn, HardStopped, HandingOver, Finished`.
User-facing copy follows v1 wording where it exists (e.g. "Count-in when pressing marker", "Auto-resume playback").

## Code rules
- **Layout:** crate per bounded context under `crates/` (SPEC §14.5). `domain` crates depend only on `shared-kernel`; contexts talk via `protocol`/events, never each other's domain. `reaper-extension` must not depend on Tauri or tokio. `app` is the composition root only. CI enforces this once Phase 0 lands.
- **Pure core:** domain/performance code has no I/O, no REAPER calls, no real clock (inject `Clock`).
- **Extension safety (SPEC S-9):** every FFI entry wrapped in `catch_unwind`; `panic = "unwind"`; `#![deny(unsafe_code)]` except one FFI module; deny `unwrap_used`, `expect_used`, `indexing_slicing`; no blocking on REAPER's main/audio thread; audio hook only touches atomics and a lock-free queue; fuzz the protocol parser; safe-mode marker file.
- **Make illegal states unrepresentable:** enums with data, newtypes (`SongId`, `Seconds`, `Bpm`). Errors are values (`thiserror`), no log-and-continue.
- **Small units:** files < ~300 lines, functions < ~40, Svelte components < ~200 lines of script. Presentational components take props only; no business logic in `.svelte`.
- **Types are generated** from Rust to TS (no hand-copied interfaces). Marker test vectors live in `testing/vectors/`.
- Match surrounding style; no comment noise; no abstractions without a second implementation or a test double (SPEC anti-goals). No event sourcing, no plugin system, no DI container.

## Git workflow (owner's rule)
- Never commit to `main` directly. Every change lives on a branch: `feature/<short-name>` for features and chores, `bugfix/<short-name>` for bugs.
- A feature or complete piece of functionality is **reviewed by the owner before it is merged**. When it is done: tests written and green, clippy/fmt clean, docs updated, PR open with CI green. Then stop and hand over for review: PR link, a short summary, what changed and where to look first, and the visible test evidence. Merge (merge commit, delete the branch) **only after the owner approves**. Never merge on my own.
- Commit messages: plain, imperative, no `Co-Authored-By` lines and no tool/AI attribution (owner's explicit instruction).
- The repo is public (`github.com/theupriser/reaper-control`): no secrets, tokens, personal data, or private paths in commits.

## Working agreements
- **Test visibly.** Show the real terminal output of builds, linters and tests (not filtered through grep/tail, not summarised). For anything with a screen (UI, app window, REAPER), run it and show the screen (screenshot or opened window). Say what was run and what was seen.
- **Spikes before commitments** (PLAN Phase 1): S1 extension loads, S2 timing, S3 socket, S4 installer, S5 webview, S6 identity, S7 crash containment. Record outcomes as ADRs in `docs/adr/`. Gate 1 is the go/no-go on the extension.
- Work packages are referenced by id (e.g. WP 3.4). Definition of done: PLAN §14.
- Each WP: tests with it; docs/ADR updated if a decision was made; parity checklist updated when a feature is reached.
- Report faithfully: failing tests, skipped steps, measured numbers (with how they were measured).
- **Cheap screenshots:** capture one window only (find its CGWindowID with a small swift `CGWindowListCopyWindowInfo` script filtered by pid, then `screencapture -x -o -l <id>`), downscale (`sips -Z 800 -s format jpeg -s formatOptions 60`), and only when it adds evidence; prefer log output for numbers. Screen Recording permission is granted to the terminal.
- **Isolated REAPER for tests:** `open -n -a /Applications/REAPER.app --args -cfgfile <dir>/reaper.ini <project>` runs a second instance whose resource path is `<dir>` (verified). Put test extensions in `<dir>/UserPlugins/`. Real projects are only ever used as copies of the `.RPP` file under the git-ignored `.dev/`; never commit them or their song names.
- Do not touch the v1 repo, the user's real REAPER config, or the user's real REAPER projects for experiments. Use a **portable REAPER copy and a sample project** (WP 0.7). If a step must touch the real REAPER install, ask first.
- Confirm before installing system software, signing, publishing, or anything outward-facing.

## Environment (as of 2026-10-06)
- macOS, Apple Silicon (arm64). Node 22, pnpm 10, git 2.54, **Rust 1.99 stable (installed via rustup on 2026-10-06; run `source ~/.cargo/env` in fresh shells)**. No `lua`.
- REAPER is installed at `/Applications/REAPER.app`; resource dir `~/Library/Application Support/REAPER` (contains a `Helgoboss` folder, i.e. ReaLearn/reaper-rs ecosystem is already present). v1 app: `/Applications/Reaper Control.app`, its data in `~/Library/Application Support/electron-reaper-control` and `reaper-control`.
- **This MacBook is the homebase** for all development and testing. Windows x64 is covered by CI only for now; real Windows testing needs another machine later.

## Status and next steps
See **`docs/STATUS.md`** (single source: done, in progress, next, Gate 1 checklist, open questions). Update it in the same PR whenever a feature, spike or bug completes. Do not duplicate status here.

## Commands
```
source ~/.cargo/env
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace      # includes the architecture rules
```

## Project commands (`.claude/commands/`)
- `/start`: orient at the start of a session (reads AGENTS.md and docs/STATUS.md, verifies against git/PRs/ADRs, proposes the next step, waits for confirmation).
- `/next`: after finishing something: sync `main`, list open PRs, update docs/STATUS.md, propose the next work package.
- Remind the owner to `/compact` whenever a feature, spike or bug is complete.

## Pointers
- v1 key files for parity: `src/main/services/{reaperConnector,regionService,projectService,midiService}.ts`, `src/main/utils/{bpmUtils,config}.ts`, `src/renderer/src/lib/utils/markerUtils.ts`, `src/renderer/src/components/{PerformerMode,TransportControls,RegionList,SetlistEditor,Settings,Help}.svelte` (all under `~/Projects/Reaper-Control-App`).
- v1 REAPER commands used: transport via web interface (`_/TRANSPORT`, `_/BEATPOS`, `_/REGION`, `_/MARKER`, actions 1007 play, 1008 pause, 40046, 40317 play from edit cursor, 40363 count-in toggle, 40667). v2 calls the REAPER API directly instead.
