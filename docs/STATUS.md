# STATUS

Single source for "where are we". `/start` and `/next` read this file first. Update it in the same PR whenever a feature, spike or bug completes (keep it short and true).

Last updated: 2026-10-07, after PR #19 (S6 second round); WP 0.5b protocol framing on `feature/protocol-framing` (this PR).

## Done (merged to main)
- Planning: `docs/SPEC.md` (draft 2), `docs/PLAN.md`, designs in `docs/design/` (canvas link in `docs/design/README.md`).
- Phase 0: Cargo workspace with one crate per context and the lint wall (WP 0.1 Rust part); CI on macOS arm64 + Windows x64 (WP 0.2); executable dependency rules in `crates/architecture-tests` (WP 0.3).
- Spike S1, macOS Apple Silicon half (WP 1.1): `docs/adr/ADR-002-extension-owned-playback.md`. Extension loads in REAPER 7.78/7.82, main-thread tick 30.0 ms, regions identical to v1's data path.
- Spike S2 (WP 1.2): `docs/adr/ADR-005-handover-strategy.md` (Proposed): seek-while-playing with about 15 ms lead, cut within 11 ms and never late. Menu and modal dialog check done (ticks keep running, one 95–135 ms gap when it opens). Recording of REAPER's output done: no gap or overlap at lead 15 ms or 0 ms (sound card path not covered).
- Spike S3 (WP 1.3): `docs/adr/ADR-003-local-link.md` (Proposed): loopback TCP, all socket work off the main thread; push latency mean 0.29 ms, slow client dropped without affecting others. Not tested: Windows, firewall prompts, reconnect with replay, menus/modals.
- Spike S7 (WP 1.7, first half): `docs/adr/ADR-009-crash-containment.md` (Proposed): panic contained with `panic = "unwind"`, abort build kills REAPER, safe-mode marker needs an `atexit` hook. Decided: only the newest REAPER build is supported (7.82 so far). Second round: panic in the audio hook and during project load are contained, a real SIGSEGV kills REAPER and the next start is safe mode, `is_playing` is correct. Open: Windows, safe-mode false positives, long fuzz campaign (WP 7.7).
- Spike S6 (WP 1.6), PRs #17 and #19: `docs/adr/ADR-008-song-identity.md` (Proposed): identity = region GUID, stable across rename/move/insert/delete/save/reload and undo/redo; projects without GUIDs get random ones on every load until saved. ExtState ProjectId survives, copies share id and GUIDs, v1 import mapping proposed. Not tested: Region Manager UI, duplicate GUIDs in one project, a real v1 setlist file, Windows.
- App shell (WP 0.1 UI half), PR #13: `crates/app` (Tauri 2, one `dispatch` over a fake performance) and `ui/` (Svelte 5, Vite, Vitest, svelte-check); CI builds, checks and tests the UI before Rust. UI types are generated since WP 0.4. App icon is the canvas logo, source `crates/app/app-icon.svg`. The screens do not look like the canvas designs yet (stub only).
- Spike S5 (WP 1.5, macOS half): `docs/adr/ADR-010-webview-performance.md` (Proposed): 30 Hz push into WKWebView, 59.8 fps, no missed events, handler 1 ms, `invoke` round trip about 2 ms, only start-up frames over 25 ms. Not tested: Windows/WebView2, Svelte on top, hidden window, touch.
- Performer screen, PR #16: `PerformerScreen` and parts in the v1 look, hand-written `PerformerView` (`ui/src/lib/performer.ts`) with fixtures per phase, `?phase=` override; only Play/Pause go through `dispatch`. Not yet: click-to-seek, record dot, system stats, keyboard controls, v1 hard-stop flash timing check.
- WP 0.4 type generation, PR #18: `Phase`, `Command`, `AppState` live in `crates/protocol` (ts-rs); `ui/src/lib/generated/protocol.ts` is generated. `cargo test -p protocol` fails with a diff when the file is stale; `UPDATE_TYPES=1 cargo test -p protocol` rewrites it. The app crate may now depend on `protocol` (architecture rule, no ADR yet). `PerformerView` in `ui/src/lib/performer.ts` is still a hand-written view model.
- Canvas shell, PR #15: `Sidebar`, screen list, Performer mode toggle, placeholder screens for Setlists, Pre-show check, Settings, Help.
- Project commands `/start`, `/next`; `AGENTS.md`; v1 GitHub fallback pointer; `docs/STATUS.md`.

## In progress
- WP 0.5b protocol framing (this PR, a feature, waits for review): `crates/protocol` has length-prefixed framing (`frame.rs`, 1 MiB limit, errors as values), the message types and the Hello handshake (`message.rs`), test vectors in `testing/vectors/frames.json`, and two `cargo-fuzz` targets (`crates/protocol/fuzz`, nightly). 60 s per target locally: 6.9 M and 5.8 M runs, no crash. CI fuzzes 30 s per target. Not yet: a socket server or client, reconnect/replay, the real message set (Live, Catalog, Event).
- Owner to-do: the hosted canvas still needs the two placement sentences from SPEC §4. Owner decision open: how aggressive safe mode is.

## Next (proposed, confirm with the owner)
- Next: the Performer extras (click-to-seek, record dot, system stats, keyboard controls, v1 hard-stop flash timing). After that the link server/client over `protocol`. S4 installer/signing needs questions first; Windows runs of S1/S2/S3/S6/S7 need a Windows machine.
- After parity (owner's wish): `!hardstop` (and maybe `!stop`) alias for the hard stop if no other default marker has that key (SPEC §4). SWS interplay for `!1008` still to verify.
- Spike S4 (installer/signing).
- Remaining Phase 0: ADRs 001/003/004 (0.5), licence decision (0.6), more sample projects (0.7), domain discovery (0.8).
- Update the designs for D7 (SPEC §10e): wizard, settings connection card, connection states, pre-show check.

## Gate 1 checklist (extension go/no-go)
S1 macOS: done. S1 Windows: open (needs a Windows machine; CI cannot run REAPER). S2 timing: measured, menu/modal checked, output recorded, macOS done (ADR-005). S3 socket: measured on macOS, Windows not run (ADR-003). S4 installer/signing: open. S5 webview: macOS measured, 59.8 fps at 30 Hz push, Windows/WebView2 not run (ADR-010). S6 identity: region GUID is stable across rename/move/insert/delete/save/reload and undo/redo, GUID-less projects get new GUIDs on every load until saved, copied project files share id and GUIDs; Region Manager UI and Windows not tested (ADR-008). S7 crash containment: macOS measured incl. audio-hook panic, load-time panic and real SIGSEGV, load-time API check works on 7.82, only the newest REAPER is supported, Windows open, frame decoder and message parser fuzzed 60 s each on macOS with no crash, long campaign waits for WP 7.7 (ADR-009).
After Gate 1: remove `spikes/` on `feature/remove-spikes`, tag the last commit with spikes first and point the ADRs at the tag.

## Open questions for the owner
PLAN §16: app name and licence, crash reporting, ReaPack, Windows x64 only?

## How to resume
Start Claude Code in `~/Projects/reaper-control-app-v2`, run `/start` (new session) or `/next` (after finishing something).
