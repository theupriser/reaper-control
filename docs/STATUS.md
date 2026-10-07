# STATUS

Single source for "where are we". `/start` and `/next` read this file first. Update it in the same PR whenever a feature, spike or bug completes (keep it short and true).

Last updated: 2026-10-07, after PR #11; Spike S6 on `feature/spike-s6-identity` (this PR).

## Done (merged to main)
- Planning: `docs/SPEC.md` (draft 2), `docs/PLAN.md`, designs in `docs/design/` (canvas link in `docs/design/README.md`).
- Phase 0: Cargo workspace with one crate per context and the lint wall (WP 0.1 Rust part); CI on macOS arm64 + Windows x64 (WP 0.2); executable dependency rules in `crates/architecture-tests` (WP 0.3).
- Spike S1, macOS Apple Silicon half (WP 1.1): `docs/adr/ADR-002-extension-owned-playback.md`. Extension loads in REAPER 7.78/7.82, main-thread tick 30.0 ms, regions identical to v1's data path.
- Spike S2 (WP 1.2): `docs/adr/ADR-005-handover-strategy.md` (Proposed): seek-while-playing with about 15 ms lead, cut within 11 ms and never late. Menu and modal dialog check done (ticks keep running, one 95–135 ms gap when it opens). Recording of REAPER's output done: no gap or overlap at lead 15 ms or 0 ms (sound card path not covered).
- Spike S3 (WP 1.3): `docs/adr/ADR-003-local-link.md` (Proposed): loopback TCP, all socket work off the main thread; push latency mean 0.29 ms, slow client dropped without affecting others. Not tested: Windows, firewall prompts, reconnect with replay, menus/modals.
- Spike S7 (WP 1.7, first half): `docs/adr/ADR-009-crash-containment.md` (Proposed): panic contained with `panic = "unwind"`, abort build kills REAPER, safe-mode marker needs an `atexit` hook. Decided: only the newest REAPER build is supported (7.82 so far). Open: Windows, safe-mode false positives, `is_playing` reads false.
- Spike S6 (WP 1.6, first half): `docs/adr/ADR-008-song-identity.md` (Proposed): identity = region GUID, stable across rename/move/insert/delete/save/reload; projects without GUIDs get random ones on every load until saved. Not tested: hand edits and undo, ProjectId, v1 import mapping.
- Project commands `/start`, `/next`; `AGENTS.md`; v1 GitHub fallback pointer; `docs/STATUS.md`.

## In progress
- Nothing open. Owner to-do: the hosted canvas still needs the two placement sentences from SPEC §4.

## Next (proposed, confirm with the owner)
- Next spike in my own order (S4 installer/signing needs questions first, S5 webview needs the Tauri shell); Windows runs of S1/S2/S3/S7 need a Windows machine.
- After parity (owner's wish): `!hardstop` (and maybe `!stop`) alias for the hard stop if no other default marker has that key (SPEC §4). SWS interplay for `!1008` still to verify.
- Spike S4 (installer/signing), S5 (webview). Rest of S6: stable ProjectId, v1 setlist import mapping, hand edits and undo.
- Remaining Phase 0: Svelte/Tauri app shell (0.1 UI), Rust to TS type generation (0.4), ADRs 001/003/004 (0.5), licence decision (0.6), more sample projects (0.7), domain discovery (0.8).
- Update the designs for D7 (SPEC §10e): wizard, settings connection card, connection states, pre-show check.

## Gate 1 checklist (extension go/no-go)
S1 macOS: done. S1 Windows: open (needs a Windows machine; CI cannot run REAPER). S2 timing: measured, menu/modal checked, output recorded, macOS done (ADR-005). S3 socket: measured on macOS, Windows not run (ADR-003). S4 installer/signing: open. S5 webview: open. S6 identity: region GUID is stable across rename/move/insert/delete/save/reload, GUID-less projects get new GUIDs on every load until saved; UI edits and undo not tested (ADR-008). S7 crash containment: macOS measured, load-time API check works on 7.82, only the newest REAPER is supported, Windows open (ADR-009).
After Gate 1: remove `spikes/` on `feature/remove-spikes`, tag the last commit with spikes first and point the ADRs at the tag.

## Open questions for the owner
PLAN §16: app name and licence, crash reporting, ReaPack, Windows x64 only?

## How to resume
Start Claude Code in `~/Projects/reaper-control-app-v2`, run `/start` (new session) or `/next` (after finishing something).
