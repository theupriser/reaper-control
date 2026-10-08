# Reaper Control v2 — Master Plan

Companion to `SPEC.md` (what/how). This file is the *plan*: phases, work packages, order, gates, risks. IDs (WP-x.y) are referenced by the issue tracker.

Decisions in force: D1 public release (macOS + Windows; D6) · D2 setlists in REAPER project · D3 companion component required, no Basic mode · D4 remote control = nice-to-have (stretch) · ~~D5 Lua-once~~ superseded by **D7: native Rust REAPER extension** (targets macOS Apple Silicon and Windows x64 only, D8; Lua + web interface kept only as fallback, SPEC §2.5) · D6 v2.0 = macOS + Windows only; Linux, updater, website, NL, MIDI learn, hold-to-confirm → v2.1 (§12b).

---
**Progress legend (keep current in every PR that touches a work package):** ✅ done and merged · 🟡 partly done (the row says what is missing) · ⬜ not started. Last checked 2026-10-08.

## 1. Guiding principles
1. Parity first, no new features (except D4 stretch, after parity).
2. Stage-safety beats everything: every WP is reviewed against SPEC §6.
3. Walking skeleton early: thinnest end-to-end slice (REAPER ↔ extension ↔ app ↔ screen) by end of Phase 1, then thicken.
4. Spikes before commitments: unknowns (S1–S6) are time-boxed and decided via ADRs.
5. Test the pure core heavily; keep adapters thin.
6. Every phase ends with a gate (demo + checklist); no starting the next phase's risky work before the gate.

## 2. Team assumptions & effort
Solo developer (+ Claude Code). Estimates are focused-work ideal days (d); multiply ×1.5 for real calendar. **Sum of work packages ≈ 205 d for v2.0 (Phases 0–8, re-summed by script after D7 and D8) + 11.5 d stretch (Phase 9). That is about 14 d more than the Lua + web-interface plan (193 d): more spikes (+5.5), the extension adapter/server and 3-target builds (+9.5 in Phase 3), the crash-containment campaign (+3), partly offset by a simpler installer, wizard and no `mlua` host. At ×1.5 ≈ 310 calendar working days; Claude Design work is not included. See REVIEW-1 for scope-reduction options.

## 2b. DDD framing (see SPEC §14)
Core domain = Performance; supporting = Setlists/Catalogue/Control/Link; generic = Installation/Settings/Diagnostics (kept simple). Work packages below are grouped by phase but delivered **per bounded context**; each context gets: event storm, glossary entries, invariant table, aggregate + tests, application services, projections.

## 3. Repository & environment (Phase 0)
```
reaper-control-app-v2/
  docs/            SPEC.md PLAN.md REVIEW-1.md language.md adr/ domain/ design/ user/
  crates/          domain crates per bounded context, `protocol`, `reaper-port`, `reaper-extension` (cdylib), infrastructure, app (SPEC §14.5); `reaper-script/` exists only if the fallback is activated
  src-tauri/       Tauri shell (thin; uses `app` composition root), event bridge
  ui/              Svelte 5 + TS + Vite
  testing/         fake-reaper (mock server), soak harness, shared vectors/scenarios
  tools/           release scripts, codegen
  .github/workflows/
```
| WP | Task | d |
|----|------|---|
| ✅ 0.1 | Git repo, Cargo workspace, pnpm workspace, rustfmt/clippy/eslint/prettier/svelte-check configs | 1 |
| ✅ 0.2 | CI: build+lint+test on macOS (arm64) and Windows x64; cross-build matrix for the extension (2 targets); dependency check (cargo-deny, audit) | 2 |
| ✅ 0.3 | Layer/context dependency check in CI (SPEC §14.5) — merged with 0.9 | 0.5 |
| ✅ 0.4 | Type generation pipeline Rust → TS (specta/ts-rs) with CI drift check | 1 |
| 🟡 0.5 | ADR template + ADR-001 (Tauri), 002 (extension-owned playback), 003 (local socket link), 004 (hexagonal + DDD contexts) **Missing: ADR-001, 003 and 004 still to write.** | 1 |
| ⬜ 0.6 | Licence decision (v1 is proprietary source-available; v2 public → choose) | 0.5 |
| ⬜ 0.8 | **Domain discovery**: event storming per context (timeline of events, commands, aggregates, invariants), glossary `docs/language.md`, context map ADR-007, invariant tables `docs/domain/*.md` | 4 |
| 🟡 0.9 | Crate-per-context workspace skeleton + CI dependency rules (§14.5), banned-synonym lint **Missing: banned-synonym lint not found in the architecture tests.** | 1.5 |
| 🟡 0.7 | Dev REAPER portable install + sample project with regions/markers/special markers for testing **Missing: more sample projects.** | 0.5 |

**Gate 0:** green CI on macOS + Windows, empty Tauri app launches, codegen works, glossary + context map + invariant tables approved.

## 4. Phase 1 — Spikes & walking skeleton
| WP | Task | d | Output |
|----|------|---|--------|
| 🟡 1.1 | **S1** Extension hello-world with `reaper-rs` on macOS arm64 (native Apple Silicon REAPER) and Windows x64: loads from `UserPlugins`, timer callback, reads position/regions/markers/tempo, runs actions, ExtState. Minimum REAPER version. Confirm what happens with an Intel REAPER under Rosetta (expected: cannot load) **Missing: Windows half.** | 3.5 | ADR-001 confirmed or fallback (SPEC §2.5) |
| 🟡 1.2 | **S2** Transport/timing: seek-while-playing vs native region playlist vs stop/seek/play; timer-driven vs audio-hook-triggered hand-over; click-track measurement of audible gaps and trigger jitter; behaviour while REAPER shows menus/modal dialogs/renders **Missing: sound card output path not measured.** | 3 | ADR-005 hand-over strategy and `lead` |
| 🟡 1.3 | **S3** Local link: loopback TCP vs named pipe/Unix socket, latency/throughput, endpoint file + token, reconnect with replay, firewall/AV prompts **Missing: Windows, firewall prompts, menus/modals.** | 2 | ADR-003 final |
| ⬜ 1.4 | **S4 (priority)** Installer feasibility: `UserPlugins` copy by the app, REAPER architecture detection, macOS signing/quarantine/Gatekeeper/library validation (ad-hoc vs Developer ID, Apple Silicon), Windows SmartScreen/AV and DLL file locking, staged update, portable installs | 3.5 | ADR-006 installer strategy + list of "manual with guidance" cases |
| 🟡 1.5 | **S5** Tauri webview differences (WKWebView macOS, WebView2 Windows): timers, CSS, fullscreen, wake-lock, MIDI access **Missing: Windows/WebView2, Svelte on top, hidden window, touch.** | 1 | Compat notes |
| ✅ 1.6 | **S6** Stable identity: region ids (index numbers vs GUID) surviving renumbering/edits; deriving a stable ProjectId; v1 setlist import mapping | 1.5 | ADR-008 SongId/ProjectId |
| 🟡 1.7 | **S7** Crash containment: `catch_unwind` at every FFI entry, panic hook → Faulted state, safe-mode marker, behaviour when REAPER crashes; `reaper-rs` API coverage vs REAPER versions **Missing: Windows, safe-mode false positives, long fuzz campaign (7.7).** | 2 | ADR-009 (S-9 design) |
| 🟡 1.8 | Walking skeleton: extension pushes `{position, status}` over the socket; app shows a ticking number; Play button works; built and run on both targets **Missing: not run on Windows.** | 4 | Demo |

**Gate 1:** measured numbers recorded; installer feasibility documented (ADR-006); **go/no-go on the extension** (otherwise activate fallback SPEC §2.5, about +10 d); hand-over strategy chosen (ADR-005).

## 5. Phase 2 — Performance core and domain (pure Rust, no I/O)
| WP | Task | d |
|----|------|---|
| ✅ 2.1 | Cargo workspace; shared kernel + Catalogue domain (Song, Cue, Directives value objects), domain events | 2.5 |
| ✅ 2.2 | Directive grammar + shared vectors (all v1 combinations + invalid input) | 1.5 |
| ✅ 2.3 | **Performance aggregate** (Phase enum, behaviour methods, domain events, HandOverPolicy, EffectiveEnd, specifications); each invariant ↔ named test; scenarios for hard stop, `!length`, count-in on cues only, finish, debounce, pause, seek | 5 |
| ✅ 2.4 | Count-in/tempo maths (bars→seconds from a tempo-map snapshot), BPM from `!bpm`/tempo | 1 |
| ✅ 2.5 | Property tests (proptest): never skips an entry, never double-fires, never leaves song bounds without hand-over, never panics | 1.5 |
| ✅ 2.6 | Setlists aggregate (validation, events, `expectedRev`) + in-memory repository | 2 |
| ✅ 2.7 | `reaper-port` crate: `ReaperPort` trait + `FakeReaper` (deterministic playhead and clock) + scenario-file format and runner | 3.5 |
| ✅ 2.8 | Projections/read models (PerformanceView, PlayerView, SetlistView) + stale/sequence handling | 2 |
| ✅ 2.9 | `protocol` crate: messages, framing, versioning, serde, type generation to TS | 2.5 |

**Gate 2:** `performance`, `catalogue`, `setlists` ≥ 95% line coverage; all v1 behaviours expressed as scenario files that pass on `FakeReaper`.

## 6. Phase 3 — REAPER extension (`reaper-extension`, Rust cdylib)
| WP | Task | d |
|----|------|---|
| ✅ 3.1 | Crate skeleton: `reaper-rs` plugin entry, FFI guard (`catch_unwind`, panic hook → Faulted), lint wall (S-9.2), safe-mode marker (S-9.4) | 3 |
| ✅ 3.2 | `ReaperRsAdapter` implementing `ReaperPort` (position, transport, regions/markers, tempo map, ExtState, actions) | 4 |
| ✅ 3.3 | Main-thread timer loop driving the performance core (snapshot → step → execute effects), tick budget, watchdog | 2.5 |
| ✅ 3.4 | Local server: listener, endpoint file, token, framing, handshake, back-pressure, multi-client, resume-from-event-id | 3.5 |
| 🟡 3.5 | Publishing: Live/Catalog/Event streams, event ring, on-disk journal **Missing: on-disk journal.** | 2.5 |
| 🟡 3.6 | Command intake/ack (ids, idempotency, unknown commands, version mismatch) **Missing: a refused command is acked Done first; ids and idempotency not checked.** | 2 |
| ✅ 3.7 | Setlist persistence in project ExtState (versioned schema, `expectedRev`, corruption recovery, re-validation on load) | 2.5 |
| 🟡 3.8 | Hand-over execution per ADR-005, hard stops, count-in (audio-hook trigger only if S2 shows it is needed and safe) **Missing: count-in effect in REAPER (R4), Windows runs.** | 3.5 |
| 🟡 3.9 | Project change/tab handling, cheap change detection, stable ids per ADR-008 **Missing: project switch and tab handling, stable ids not verified live.** | 2 |
| ⬜ 3.10 | Build and packaging: CI cross-builds for the 2 targets, signing, version constant, checksum, in-REAPER smoke test | 3 |

**Gate 3:** scenarios pass on `FakeReaper` **and** in real REAPER on both targets; 30 min unattended run, 40 hand-overs, zero misses; kill-the-app test keeps playing; panic injection leaves REAPER alive and reports `ExtensionFaulted`; journal matches events.

## 7. Phase 4 — App core (Rust)
| WP | Task | d |
|----|------|---|
| 🟡 4.1 | Application layer per context: use-case services, ports (`Driver`, `Clock`, repositories, `MidiSource`), in-process domain event bus, command bus **Missing: Clock, MidiSource and repository ports, use-case services.** | 3 |
| ✅ 4.16 | Reaper Link ACL: `MessageTranslator`/`CommandTranslator`, LinkSession aggregate + health events (with cause classification) `ExtensionFaulted` arrives as the `faulted` file (4.6). No `CommandTranslator`, by design: the command is already the wire type, so one is added only when a second form exists. | 2.5 |
| 🟡 4.17 | Control context: `ControlBinding`, `IntentTranslator` (one validation path for UI/MIDI/keyboard/remote) **Missing: `ControlBinding` is the MIDI note map in the config for now; keyboard and remote bindings come with the UI (6.x, 9).** | 2 |
| 🟡 4.2 | Link client over `protocol`: connect via endpoint file, handshake, reconnect with back-off, resume from last event id; decorators Timeout/Metrics/Logging Timeout is `CommandBus` expiry, Logging is `event_logger`, Metrics is `MeteredDriver` (send-to-answer delay); replay after a cut connection and the heartbeat are tested on real sockets. **Missing: Windows run (CI compiles only).** | 3.5 |
| ✅ 4.3 | `ExtensionDriver`: state decode, version check, command send/ack | 2.5 |
| ✅ 4.4 | Command queue: ordered, id'd, de-duplication of rapid repeats, per-command timeout, back-pressure; the UI shows refused, dropped, timed-out and full-queue commands as notices (limits come from config, 4.7) | 2.5 |
| ✅ 4.5 | Command bus + handlers, single dispatch entry. By design there is no handler per command in the app: the extension judges state, `command_check` only refuses what can never be right, and MIDI goes through the 4.17 intent translator (the UI sends commands straight to the same bus) | 2 |
| ✅ 4.6 | Link health model: Connected/Degraded/Lost/Dead + causes ReaperNotRunning/ExtensionNotLoaded/ExtensionOutdated/ExtensionFaulted; events The extension writes a `faulted` file when it disables itself (safe mode, REAPER too old) and clears it at a normal start; the app reads it and the UI shows every cause as a notice. | 1.5 |
| ✅ 4.7 | Config: typed struct, defaults, validation, atomic write, schema version + migrations; Settings screen for the MIDI and queue settings (queue limits apply at once, MIDI after a restart; the note table is read only until MIDI learn, v2.1) | 2 |
| 🟡 4.8 | Setlist backup mirror + v1 import (config.json, setlists/*.json) (mirror follows the project, restore and import in Settings, v1 MIDI settings taken over on the first start) **Missing: region number not matched (the catalog does not carry it; names only), no Setlists screen yet (the card moves there with WP 5.x), not tried on a real v1 project.** | 2.5 |
| 🟡 4.9 | MIDI (midir): device list/hotplug, channel filter, note→Command registry (user-editable mapping), global debounce **Missing: no screen to pick the device or edit the mapping (6.5), settings need a restart, key-up and other message types ignored.** | 2.5 |
| ✅ 4.10 | Logging (tracing): levels, rolling file, journal import from the extension, "export diagnostics" bundle. The level is set in Settings (`config.json`, applies after a restart; `RC2_LOG` overrides). The extension writes `journal.log` (one JSON line per event; a journal over 1 MB is set aside at the next start); the app copies new lines into its log at start, on connect, on loss and before an export. "Export diagnostics" in Settings writes one zip (about, config, app logs, extension log and journal, `faulted`; never `endpoint.json`, it holds the token) | 2 |
| 🟡 4.11 | FakeDriver + simulator (the same `performance` crate over `FakeReaper`, deterministic clock) **Missing: simulator not built, only FakeDriver.** | 1.5 |
| ⬜ 4.12 | In-process fake extension server (the real server code over `FakeReaper`) for conformance and chaos tests (drops, latency, reordering, garbage) | 2.5 |
| ⬜ 4.18 | System stats (`sysinfo`: CPU/memory) for F14 | 1 |
| ⬜ 4.14 | **Installer module** (`ExtensionInstaller` port + OS adapters): locate REAPER, detect its architecture, `InstallReport`, copy to `UserPlugins`, signature/quarantine handling, rollback copy, staged update, verify-by-handshake, repair, uninstall, dry-run | 5 |
| ⬜ 4.15 | Installer test suite: temp fake REAPER resource dirs (clean / existing extension / read-only / wrong architecture / portable / running-REAPER lock), golden-file tests | 2.5 |
| ⬜ 4.13 | Composition root, graceful shutdown, panic hook → log + safe UI state | 1.5 |

**Gate 4:** installer suite green on all temp-dir scenarios; conformance + chaos suite green incl. link loss, replay and restart-reattach; CLI harness can run a whole set against `FakeReaper` and the real extension.

## 8. Phase 5 — UI foundation & design (Claude Design runs in parallel from here)
| WP | Task | d |
|----|------|---|
| ⬜ 5.1 | Design system: tokens (colour, type, spacing, motion), light/dark/stage-dark, density, large-touch variant | 3 |
| ⬜ 5.2 | Component kit: Button, Toggle, Panel, Timeline(markers, hard-stop), Clock, StatusBadge, ListRow, Toast(non-modal), Dialog | 5 |
| ⬜ 5.3 | State bridge: single `appState` store from one event; command helper `send()`; pending/ack indicators | 2 |
| ⬜ 5.4 | Navigation shell, keyboard layer (space, ←/→, a — via command bus), focus management | 2 |
| ⬜ 5.5 | UI strings kept in one module (no hard-coded copy in components) so translation can be added in v2.1 | 0.5 |
| ⬜ 5.6 | Accessibility baseline (contrast, focus, ARIA, reduced motion) | 1.5 |
| ⬜ 5.7 | Frame-budget tooling: input→paint measurement, performance budget in CI (Playwright trace) | 1.5 |

**Gate 5:** designs for all 8 screens approved (§10 of SPEC); kit storybook-style page complete.

## 9. Phase 6 — Screens
| WP | Screen | Parity | d |
|----|--------|--------|---|
| ⬜ 6.1 | Player: transport, timeline w/ markers, region/setlist list, toggles, BPM, record arm | F2–F4, F7, F9 | 5 |
| ⬜ 6.2 | Setlist editor: create/rename/delete, add/remove/reorder (drag), select, validation (missing regions) | F5 | 5 |
| ⬜ 6.3 | **Performer**: title, song/total time, next song, hard-stop prompt, clock, toggles, exit; stage-lock | F11 | 5 |
| ⬜ 6.4 | Connection/health: header status, popovers, banners (Lost/NotLoaded/Outdated/Faulted), diagnostics | F1, F14 | 3 |
| ⬜ 6.5 | Settings: connection status + repair, MIDI (devices, channel, mapping), behaviour, appearance | F12, F13 | 4 |
| ⬜ 6.6 | First-run wizard + Repair screen driven by `InstallReport`: find REAPER → install extension → restart REAPER → connect; per-step status (Ok/Fixed/Manual), instruction cards with copy/reveal buttons, "Check again", REAPER-running handling, safe-mode re-enable, success only on live handshake | new (D3, R-INST) | 5 |
| ⬜ 6.7 | Pre-show checklist | S-7 | 2 |
| ⬜ 6.8 | Help + marker guide (+ optional marker helper) | F15, F8 | 3 |
| ⬜ 6.9 | Perf pass: virtualised lists, batch updates, no layout thrash; verify budgets | goal 5 | 3 |

**Gate 6 (feature complete):** parity checklist F1–F18 all ticked with evidence (test or manual script).

## 10. Phase 7 — Hardening
| WP | Task | d |
|----|------|---|
| ⬜ 7.1 | Soak: 2 h/40-song simulated set, random link drops/app kills; assert zero missed hand-overs | 3 |
| ⬜ 7.2 | Chaos tests: REAPER paused/unfocused, project switch mid-song, regions edited live, huge projects (1000 regions) | 3 |
| ⬜ 7.3 | Real-rig rehearsals ×3 (mac + windows) with audio interface and MIDI foot controller | 3 |
| ⬜ 7.4 | Security review: localhost-only, input validation, Tauri capability allow-list, CSP | 1.5 |
| ⬜ 7.5 | Resource audit: idle CPU/RAM budget (target < 1% CPU, < 120 MB) | 1 |
| ⬜ 7.7 | **Crash-containment campaign (S-9)**: panic/fault injection, fuzzing the protocol parser, safe-mode flows, 8 h soak inside real REAPER on both targets, REAPER-crash recovery | 3 |
| ⬜ 7.6 | Beta programme: 5–10 musicians, feedback form, crash/diagnostic export | calendar 3 wks |

**Gate 7:** zero known P0/P1; crash-containment campaign passed (no REAPER crash attributable to the extension); beta sign-off.

## 11. Phase 8 — Public release
| WP | Task | d |
|----|------|---|
| ⬜ 8.1 | Code signing + notarisation (Apple Developer ID for the app **and** the extension binary, Windows code-signing cert), macOS dmg (Apple Silicon) + Windows installer, bundling both extension builds | 3.5 |
| ⬜ 8.2 | Release pipeline (tags → signed artefacts → draft GitHub release; manual publish) | 2 |

| ⬜ 8.9 | Installer QA matrix: clean/existing REAPER setups × (macOS arm64 REAPER, Intel REAPER under Rosetta → guidance only, Windows x64) × running/closed REAPER × admin/standard user; manual-install guide verified by a non-developer | 2.5 |
| ⬜ 8.4 | User docs site (install, first-run, markers, MIDI, troubleshooting), manual extension install guide | 4 |
| ⬜ 8.6 | Support process: issue templates, diagnostics bundle, FAQ | 1 |
| ⬜ 8.7 | Licence + third-party notices, privacy statement (no telemetry by default) | 1 |
| ⬜ 8.8 | v1→v2 migration guide and import test | 1 |

**Gate 8:** v2.0.0 tagged.

## 12. Stretch — Phase 9 (D4) remote control
| WP | Task | d |
|----|------|---|
| ⬜ 9.1 | Local web server in Rust exposing State stream (WebSocket) + `dispatch` | 3 |
| ⬜ 9.2 | Pairing/auth: one-time code/QR, token, LAN-only default, off by default | 3 |
| ⬜ 9.3 | Touch-first remote UI (reuses Performer components) | 4 |
| ⬜ 9.4 | Multi-client conflict rules (commands ordered by bus, last-write visible) | 1.5 |
Architecture impact already accounted: command bus + event stream are the only interfaces, so remote is another adapter.

## 12b. v2.1 backlog (deferred by D6)
| Item | Est. d | Notes |
|------|--------|-------|
| Linux support (Tauri webview QA, AppImage/deb, REAPER-on-Linux installer paths, CI) | 6 | Code already portable; mainly QA/packaging |
| Auto-updater (opt-in, only when idle, signed feed) | 3 | Never during a show |
| Website + download page | 2 | |
| Dutch translation + i18n framework | 2 | Strings already centralised (WP 5.5) |
| MIDI learn | 1 | |
| Hold-to-confirm option for Next/Previous | 1.5 | |

## 13. Critical path & parallelism
`0 → 1 (S1,S2) → 2 → 3 ∥ 4 → 5 (design in parallel from Phase 2) → 6 → 7 → 8`
- Phases 3 and 4 can overlap after 2.3 (state machine), 2.7 (`ReaperPort`/`FakeReaper`) and 2.9 (`protocol`) are stable.
- Claude Design work (SPEC §10) runs in parallel to Phases 2–4, feeding Phase 5 gate.
- The only hard blockers for everything: **S1/S2/S3/S4 results** (Gate 1: extension go/no-go).

## 14. Definition of Done (every WP)
Code + tests + docs updated, one type per file (SPEC §14), clippy/eslint/svelte-check clean, layer check passes, reviewed against SPEC §6 stage-safety, ADR if a decision was made, parity checklist updated.

## 15. Risk register
| # | Risk | P | I | Mitigation | Owner trigger |
|---|------|---|---|-----------|---------------|
| R1 | Extension cannot be loaded/run reliably on one of the 2 targets (signing, API coverage) | M | H | S1/S4 first; fallback SPEC §2.5 | Gate 1 |
| R2 | Seek-while-playing glitches | M | H | S2; region playlist or pre-roll strategy | Gate 1 |
| R3 | Auto-install not possible on some setups (permissions, AV, managed machines, portable) | M | M | S4; installer reports Manual steps with exact guidance; QA matrix 8.9 | Gate 1 / 6.6 |
| R4 | `FakeReaper` diverges from real REAPER behaviour (simulator trusted too much) | M | M | In-REAPER click-track tests (WP 3.10), rehearsals, scenarios replayed in real REAPER | Gate 3 |
| R15 | **Extension crash takes REAPER down mid-show** | M | H | S-9 (catch_unwind, lint wall, minimal unsafe, fuzzing, safe mode), WP 7.7 campaign, release gate | Gate 3 / 7 |
| R5 | Webview differences between macOS and Windows break stage UI | L | M | S5, test on both OS each phase | Phase 5 |
| R6 | REAPER API changes between versions | L | M | Declare min REAPER version; test 2 versions in CI manually per release | Phase 7 |
| R7 | Scope creep before parity | H | M | Parity-only rule; ideas parked in backlog | Always |
| R8 | Signing/notarisation delays release | M | M | Start accounts in Phase 5 | Phase 8 |
| R11 | DDD over-engineering (ceremony in simple contexts) | M | M | Anemic CRUD allowed for generic subdomains; model only where invariants exist; review at each gate | Always |
| R12 | Hand-over depends on the main thread (blocked by modal UI) | M | H | S2 verdict; audio-hook trigger or native region playlist if needed; stage guidance | Gate 1 |
| R13 | Unstable region ids break setlists | M | H | S6; identity by GUID/fallback matching; validation + repair UI | Gate 1 |
| R14 | Schedule: ~205 d solo (~308 calendar days) | H | M | v2.1 backlog (§12b) already deferred; further cuts only by owner decision | Gate 0 |
| R16 | Intel REAPER under Rosetta on an Apple Silicon Mac (extension cannot load), or wrong build installed | M | M | Installer inspects the REAPER binary, not the machine; guidance card to install the Apple Silicon REAPER; QA matrix | Gate 4 |
| R17 | REAPER must be restarted to load/update the extension; users forget or update at the wrong time | M | M | Clear wizard step, staged updates, never offered mid-show | Phase 6 |
| R18 | `reaper-rs`/REAPER API drift between versions | L | M | Declare min REAPER version (S7), CI on two REAPER versions per release | Phase 7 |
| R9 | Solo bus-factor/time | M | M | ADRs, docs, small modules, automation | Always |
| R10 | Extension work on REAPER's main thread starves REAPER/UI under heavy projects | L | H | Tick budget + watchdog (WP 3.3), no blocking calls, soak tests | Gate 3 |

## 16. Open questions (to resolve before/at Gate 0–1)
0. ~~Targets~~ — decided (D8): macOS Apple Silicon + Windows x64; no Intel Macs. Still to confirm: Windows is x64 only (no Windows on ARM).
1. ~~Minimum supported REAPER version?~~ Decided 2026-10-07: only the newest REAPER build is supported (ADR-009).
2. App name / branding / licence for the public release.
3. ~~Languages at launch~~ — decided: English only in 2.0, NL in v2.1 (D6).
4. Do you want anonymous opt-in crash reporting, or diagnostics export only (proposal: export only)?
5. ~~Distribution~~ — decided: GitHub releases for 2.0; website/updater in v2.1 (D6).
6. Extension distribution: also via ReaPack in addition to the app installer?
