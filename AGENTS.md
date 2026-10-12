# AGENTS.md — Reaper Control v2

Working notes for AI agents (and humans) on this repo. Read this first, then the docs it points to. Keep it short and accurate: update it when a decision changes.

## What this is
Reaper Control v2: a stage app that controls the REAPER DAW and plays a setlist live. It is a rewrite of v1 (Electron/Svelte/TS, at `~/Projects/Reaper-Control-App`, **read-only reference, never edit it**; if the local copy is not available, the same code is at https://github.com/theupriser/Reaper-Control-App, `main`, read it with `gh api repos/theupriser/Reaper-Control-App/contents/<path> --jq .content | base64 -d` or `gh repo clone` into the scratchpad). Owner: Rick Peters (rickpeters@upriser.nl). Used on stage, so **stability beats features**.

## Source of truth (read in this order)
The `docs/` and `spikes/` folders are kept on the owner's machine only (git-ignored since 2026-10-11, backup copies sit next to the repo); it is not in the repository, so STATUS and PLAN updates are edited in place and are not part of any PR.
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

## Repository layout
```
crates/                 one crate per bounded context; `installer` holds the extension install and the setup wizard checks, `app-core` the shared app types (commands and the bus, events, config, intents, clock), `midi` the MIDI input (see "Architecture" and "Code rules")
ui/                     Svelte 5 UI; copy in ui/src/lib/strings.ts, generated types in ui/src/lib/generated/
testing/
  data/scenarios/       performance scenarios (JSON), read by the reaper-port tests
  data/vectors/         protocol and marker test vectors (JSON)
  data/projects/        sample REAPER projects (markers and regions, no audio)
  data/reaper/          starting reaper.ini for the isolated REAPER in tests
  scripts/              scripts that run against an isolated REAPER (soak, security, resource audit, chaos/)
.github/
  workflows/            CI and the release workflow
  scripts/              packaging and the release build, used by the workflows
  release-notes/        one file per release, vX.Y.Z.md (see "Releases")
docs/, spikes/, .claude/  local only, git-ignored
```
Where to look in `crates/app/src` (one folder per topic; a name with an underscore becomes a sub-folder, e.g. `input/midi/router.rs`; the crate root still re-exports the public API flat, so `app::Type` paths do not change):
```
(command, event, config, intent and clock now live in the `app-core` crate, same folder names)
legacy/    import of v1 config, setlists and journals
mirror/    the restore-only mirror of setlists
link/      the connection to the extension: session, pipeline, health, chaos proxy
health/    faults and the health monitor
system/    clock, stats, diagnostics bundle, logging setup, panic hook, periodic thread, shutdown
fake/      test doubles for the traits above
start/     starting the installation service and the link
shell.rs   the composition root; simulator/ the app without REAPER
```
Other crates follow the same shape: `protocol/src` groups its types by topic (`installation/`, `configuration/`, `link/`, `state/`, `song/`, `setlist/`; `frame/` and `message/` are the wire format), `performance/src` keeps the state machine at the top with `tempo/` and `song/` beside it.
`testing/README.md` explains the testing folder; an architecture test (`testing_layout.rs`) keeps it in that shape. Put new fixtures in `testing/data/`, new manual scripts in `testing/scripts/`, and anything a workflow calls in `.github/scripts/`. The checks on a pull request have a target of 5 minutes and a limit of 10; longer runs belong in a nightly or manual job.

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
- **One type per file (owner's rule, 2026-10-07):** every `struct`, `enum` and `trait` has its own file, named after it in snake_case (`LinkServer` in `link_server.rs` or the module's `server.rs`); its `impl` blocks live in the same file. The parent module only declares the modules and re-exports (`pub use`), so public paths do not change. Free functions go with the type they serve, or in one file named for what they do. Test modules (`tests.rs`) are exempt. One responsibility per type: if a type does connecting and timing and reporting, split it. Namespaces follow responsibility: a type with helpers is a file `x.rs` plus a folder `x/` holding its private helper types (`server/connection.rs` with `server/connection/handshake.rs`); crate-internal groups get their own module (`wire`), and the crate root re-exports only the public API. Check: no file under `crates/*/src` declares two types.
- **No abbreviations (owner's rule, 2026-10-08):** write names out in full in identifiers, file names, modules, fields and wire names: `directory`, `sequence`, `timestamp`, `revision`, `error`, `foreign_interface`. Exceptions: fixed domain terms (`Bpm`, `Id`, and the marker syntax `!bpm`, `!length`, `!1008`) and acronyms that are the normal name (`Json`, `Tcp`, `Uuid`). Single letters only for a closure over one obvious value.
- **Memory and layout (owner's rule, 2026-10-09; enforced from Phase 4b, PLAN §7b):** structs and enums stay under 128 bytes, ideally 64 or less; no large arrays or heavy values by value; a heavy type or variant goes behind `Box`/`Vec`/`Rc` with a one-line comment; heed `clippy::large_enum_variant`; check `size_of` before adding fields. Module and file names are `lower_snake_case` and singular; use `x.rs` next to `x/`, never `mod.rs`; keep internals private (`mod` without `pub`) and expose the API with `pub use`.
- **Small units:** files < ~300 lines, functions < ~40, Svelte components < ~200 lines of script. Presentational components take props only; no business logic in `.svelte`.
- **Types are generated** from Rust to TS (no hand-copied interfaces): types in `crates/protocol`, output `ui/src/lib/generated/protocol.ts`, refresh with `UPDATE_TYPES=1 cargo test -p protocol`. Marker test vectors live in `testing/data/vectors/`.
- Match surrounding style; no comment noise; no abstractions without a second implementation or a test double (SPEC anti-goals). No event sourcing, no plugin system, no DI container.

## Git workflow (owner's rule)
- Never commit to `main` directly. Every change lives on a branch: `feature/<short-name>` for features and chores, `bugfix/<short-name>` for bugs.
- A finished **feature** is a checkpoint: it is **reviewed by the owner before it is merged**. **Spike and bugfix PRs** are merged by the agent itself once CI is green (owner's rule, 2026-10-07). When it is done: tests written and green, clippy/fmt clean, docs updated, PR open with CI green. Then stop and hand over for review: PR link, a short summary, what changed and where to look first, and the visible test evidence. Merge (merge commit, delete the branch) a feature **only after the owner approves**; spikes and bugfixes need no approval.
- Commit messages: plain, imperative, no `Co-Authored-By` lines and no tool/AI attribution (owner's explicit instruction).
- The repo is public (`github.com/theupriser/reaper-control`): no secrets, tokens, personal data, or private paths in commits.

## Releases
- A release is made by pushing a tag `vX.Y.Z` that equals the workspace version. The workflow builds both installers and publishes the release by itself.
- The run **fails at the start** unless `.github/release-notes/vX.Y.Z.md` exists on `main`. Write it in the release PR, in the style of `.github/release-notes/v2.0.0.md` (sections such as Playing a set, Control, Known issues), in plain words for performers, not developers.
- Put in it: new features, and bugfixes that are important and big enough to matter on stage. Leave out code cleanup, refactoring, tests, CI, docs and other internal changes. Keep the known issues list current.
- Never publish by hand and never tag without the owner's go-ahead.

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
See **`docs/STATUS.md`** (single source: done, in progress, next, Gate 1 checklist, open questions). Update it in the same PR whenever a feature, spike or bug completes, and set the ✅/🟡/⬜ mark of the work package in `docs/PLAN.md`. Do not duplicate status here.

## Commands
```
source ~/.cargo/env
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace      # includes the architecture rules
pnpm --dir ui install && pnpm --dir ui build   # first: the app embeds ui/dist at compile time
pnpm --dir ui check && pnpm --dir ui test
cd crates/app && ../../ui/node_modules/.bin/tauri dev   # run the app window
.github/scripts/build_release.sh    # UI + extension + dist/ + app, in order (a runnable release build)
```

## Project commands (`.claude/commands/`, local only, git-ignored)
- `/start`: orient at the start of a session (reads AGENTS.md and docs/STATUS.md, verifies against git/PRs/ADRs, proposes the next step, waits for confirmation).
- `/next`: after finishing something: sync `main`, list open PRs, update docs/STATUS.md, propose the next work package.
- Remind the owner to `/compact` whenever a feature, spike or bug is complete.

## Project agents (`.claude/agents/`, local only, git-ignored)
- `coder`: builds one work package on a branch, with tests, and opens the PR. Does not merge or touch STATUS/PLAN.
- `reviewer`: reads a PR with `gh` against the diff, docs and these rules; reports findings, changes nothing.
- `tester`: runs the checks and the app with the isolated REAPER, reports what it saw, changes nothing.
- `docs-keeper`: the `/next` routine: syncs `main`, updates STATUS and the PLAN marks, names the next package.
- Agents report to the main session, not to each other; the main session passes findings on. Use them when asked, they start without context.

## Pointers
- v1 key files for parity: `src/main/services/{reaperConnector,regionService,projectService,midiService}.ts`, `src/main/utils/{bpmUtils,config}.ts`, `src/renderer/src/lib/utils/markerUtils.ts`, `src/renderer/src/components/{PerformerMode,TransportControls,RegionList,SetlistEditor,Settings,Help}.svelte` (all under `~/Projects/Reaper-Control-App`).
- v1 REAPER commands used: transport via web interface (`_/TRANSPORT`, `_/BEATPOS`, `_/REGION`, `_/MARKER`, actions 1007 play, 1008 pause, 40046, 40317 play from edit cursor, 40363 count-in toggle, 40667). v2 calls the REAPER API directly instead.
