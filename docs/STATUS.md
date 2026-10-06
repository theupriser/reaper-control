# STATUS

Single source for "where are we". `/start` and `/next` read this file first. Update it in the same PR whenever a feature, spike or bug completes (keep it short and true).

Last updated: 2026-10-06 (evening), after PR #3; session paused, continues the next day.

## Done (merged to main)
- Planning: `docs/SPEC.md` (draft 2), `docs/PLAN.md`, designs in `docs/design/` (canvas link in `docs/design/README.md`).
- Phase 0: Cargo workspace with one crate per context and the lint wall (WP 0.1 Rust part); CI on macOS arm64 + Windows x64 (WP 0.2); executable dependency rules in `crates/architecture-tests` (WP 0.3).
- Spike S1, macOS Apple Silicon half (WP 1.1): `docs/adr/ADR-002-extension-owned-playback.md`. Extension loads in REAPER 7.78/7.82, main-thread tick 30.0 ms, regions identical to v1's data path.
- Project commands `/start`, `/next`; `AGENTS.md`; v1 GitHub fallback pointer.

## In progress
- PR `feature/status-file` (this file, `/start` and `/next` reading it) waiting for the owner's review. Nothing else.

## Next (proposed, confirm with the owner)
**Spike S2 (WP 1.2): timing and seek-while-playing**, branch `feature/spike-s2-timing`.
- Extend `spikes/s1-hello-extension` (or a sibling spike crate) to trigger a seek at the end of a song with three methods: seek-while-playing, native region playlist, stop/seek/play.
- Synthetic click-track project in `testing/projects/`; measure from REAPER's own play position and timestamps: gap, jitter, trigger lead. Also test while a menu/modal dialog is open and during project load (S1 saw a 3.6 s main-thread stall).
- Isolated test instance (`-cfgfile .dev/reaper-test/reaper.ini`), low volume / no audio device. Show logs; cheap window-only screenshots.
- Output: ADR for hand-over strategy (ADR-005). Gate 1 depends on it.

Other small items that can go first or in between:
- `bugfix/directive-placement-docs`: `!1008` may sit anywhere inside the song, `!bpm` sits at the start edge (+0.00002 s); fix SPEC §4, `docs/design/canvas/Help.dc.html` and the hosted canvas.
- Remaining Phase 0: Svelte/Tauri app shell (0.1 UI), Rust→TS type generation (0.4), ADRs 001/003/004 (0.5), licence decision (0.6), more sample projects (0.7), domain discovery (0.8).
- Delete the remote branch `feature/project-commands` if it still exists.
- Update the designs for D7 (SPEC §10e): wizard, settings connection card, connection states, pre-show check.

## Gate 1 checklist (extension go/no-go)
S1 macOS: done. S1 Windows: open (needs a Windows machine; CI cannot run REAPER). S2 timing: open. S3 socket: open. S4 installer/signing: open. S5 webview: open. S6 identity (GUIDs exist in the project file, see ADR-002): open. S7 crash containment: open.
After Gate 1: remove `spikes/` on `feature/remove-spikes`, tag the last commit with spikes first and point the ADRs at the tag.

## Open questions for the owner
PLAN §16: minimum REAPER version, app name and licence, crash reporting, ReaPack, Windows x64 only?

## How to resume
Start Claude Code in `~/Projects/reaper-control-app-v2`, run `/start` (new session) or `/next` (after finishing something).
