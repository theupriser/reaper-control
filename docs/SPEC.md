# Reaper Control v2 — Specification (draft 2, after decision D7: native REAPER extension)

## 0. Goals (from the owner)
1. **Feature parity with v1** — no new features until parity is reached.
2. **Stage-stable** — the show keeps playing if the app crashes or the link drops.
3. **One path per action** — every action (play, next, seek, toggle…) has exactly one code path, whether triggered by UI, keyboard or MIDI.
4. **No duplicated logic** — one state model, one transport state machine, one marker parser.
5. **Responsive UI** — input-to-visual feedback < 50 ms, REAPER state reflected < 100 ms.

**Parity definition:** no new *performance* features. The following additions are explicitly allowed because decisions D1/D3 or stage-safety require them: extension installer/repair (§13), link-health states, pre-show checklist, diagnostics export + extension-side journal, simulator/dry-run. (Hold-to-confirm, MIDI learn and Dutch are v2.1.) Anything else goes to the backlog.

Non-goals for v2.0: cloud, telemetry, event sourcing; remote clients are a *stretch* (D4, PLAN Phase 9), updates never prompt during a show.

## 1. v1 feature inventory (the parity checklist)
| # | Feature | v1 source |
|---|---------|-----------|
| F1 | Connect to REAPER, auto-reconnect, link latency (v1: host/port/protocol; v2: automatic, see §2.2) | reaperConnector, ConnectionStatus |
| F2 | Transport: play/pause toggle, pause, play with count-in, seek, restart region | reaperConnector, TransportControls |
| F3 | Region list, current region by playhead, next/previous (honours setlist) | regionService, RegionList |
| F4 | Markers list/timeline display; special markers hidden; **hover tooltip with the cue name (and "Hard stop point" on the red hard-stop marker), on Player and Performer** | markerService, markerUtils, marker-tooltip in PerformerMode/TransportControls |
| F5 | Setlists per project: create/rename/delete, add/remove/move items, select | projectService, SetlistEditor |
| F6 | Playlist mode: auto-switch to next song before region end; pause at end of list | regionService.checkEndOfRegion |
| F7 | Autoplay (auto-resume) toggle; count-in toggle; record-arm toggle | playbackStore |
| F8 | Special markers `!1008` hard stop, `!length:N`, `!bpm:N` (combinable) | markerUtils, bpmUtils |
| F9 | BPM display (from REAPER tempo map / `!bpm`) + time signature | bpmUtils |
| F10 | Count-in: seek 2 bars before the target and play with count-in. **v1 semantics: count-in applies only when jumping to a marker/cue (toggle is labelled "Count-in when pressing marker"); song-to-song navigation never counts in** | regionService |
| F11 | Performer mode: song title, song time + remaining, total set elapsed/remaining, next song + duration, hard-stop prompt, clock, record dot, toggles, exit | PerformerMode |
| F18 | Timeline click-to-seek with time popover (player and performer) | handleProgressBarClick |
| F12 | MIDI input: device select/hotplug, channel filter, note→action mapping, debounce | midiService |
| F13 | Settings: connection status/repair, MIDI. (Host, port, protocol, polling interval, reconnect settings and transition offset disappear: the extension runs the timing in-process. An advanced `handOverLeadMs`, default 0, may remain, see §3) | Settings |
| F14 | System stats popover (machine CPU/memory, link latency; Electron/Node figures dropped) | SystemStats |
| F15 | Help screen + first-run help | Help |
| F16 | Keyboard: space, ←/→, `a` | transportService |
| F17 | Logging (file/level) | logger |

Known v1 gaps to *not* carry over: duplicated stores, 2 s button lockout workaround, latency-guess transitions, app-side local timer, `any`-typed config merge.

## 2. Architecture

```
┌──────────────── REAPER ─────────────────┐          ┌─────────── App (Tauri 2) ───────────┐
│ Companion extension (Rust, reaper-rs)    │  local   │ Rust core                           │
│  • Performance core (shared Rust crate)  │  socket  │  • Link client (reconnect, replay)  │
│  • REAPER timer / audio hook             │◄────────►│  • Command queue (ordered, acked)   │
│  • local server 127.0.0.1 + token        │  push    │  • MIDI input (midir)               │
│  • setlists in project ExtState          │          │  • Config + setlist mirror (serde)  │
│  • hand-over journal                     │          │ Svelte 5 UI (thin, read-only state) │
└──────────────────────────────────────────┘          └─────────────────────────────────────┘
```
No REAPER web interface and no Lua script are used (decision D7). Fallback design if the spikes reject the extension: §2.5.

### 2.1 Roles
- **Extension = source of truth for playback.** Song-end detection, hard stops, count-in seek and auto-advance run inside REAPER, driven by REAPER's timer (and, if spike S2 proves it safe and useful, an audio-hook trigger). No network in the timing path.
- **App = remote control + display.** Sends commands, renders the latest state. Holds no playback logic and no timer. Position between pushes is *interpolated for drawing only* (never for decisions).
- **If the app dies:** the extension keeps running the set. App restart reconnects and resumes from the last event it saw.
- **If the extension is absent or not loaded:** the app cannot control playback. It shows the install wizard (§13) and stays read-only. There is no second playback path. The app's `Driver` port has two implementations: `ExtensionDriver` and `FakeDriver` (simulator/tests).

### 2.2 Link protocol (local socket, push-based)
- **Transport:** TCP on `127.0.0.1` (named pipe / Unix socket if spike S3 shows an advantage). The extension picks a port at start and writes `<REAPER resource>/RC2/endpoint.json` = `{protocol, port, token, extensionVersion, pid}` readable only by the user. The app reads it to connect. Non-loopback connections are refused.
- **Framing:** length-prefixed JSON messages (versioned; binary encoding later only if measurements demand it). A frame is a 4-byte big-endian length, then that many bytes of one JSON document; empty frames and frames above 1 MiB are protocol errors and the connection is closed (`crates/protocol`, `frame.rs`). The first message must be `Hello{protocol, token}` with the right version and token, else the extension closes without a reply.
- **Messages:** `Hello{protocol, token, resumeFromEventId}` → `Welcome{extensionVersion, catalogRev, setlistRev, lastEventId}`; `Live` (pushed on change, at most ~30 Hz while playing, always at least 1 Hz as heartbeat); `Catalog` (sent when `catalogRev`/`setlistRev` change or on request: songs, cues, parsed directives, tempo, setlists); `Event{id,…}` (hand-over completed, hard stop reached, finished, command rejected…); `Command{id,type,args}` → `Ack{id, ok|error}`; `Ping/Pong`.
- **No lost events:** the extension keeps a ring of the last 1000 events plus the on-disk journal; after reconnect the app asks to resume from its last event id and gets the missed ones. Event ids start at 1 and rise by one. If the app is further behind than the ring (or ahead of a restarted extension) it gets `EventsLost{oldestAvailable}` instead, asks for the `Catalog` and treats its state as new. Implemented as `EventLog::since` in `crates/link` (WP 2.9); the wire types are in `crates/protocol` (protocol version 2: `Hello.resume_from_event_id`, `Welcome` with the revisions and the last event id, `Live`, `Catalog`, `Event`, `EventsLost`, `GetCatalog`).
- **Commands** are idempotent (id-deduplicated), processed in order, acknowledged individually. The app queue de-dupes rapid repeats; there is no global button lockout.
- **Health:** states `Connected / Degraded / Lost / Dead` (heartbeat missed 1 s / 5 s) with cause classification (§14.3): `ReaperNotRunning` (process check), `ExtensionNotLoaded` (REAPER running, no endpoint file or no connection), `ExtensionOutdated` (protocol/version mismatch), `ExtensionFaulted` (extension reports it disabled itself after an internal error).
- **Security:** loopback only plus token; the extension never opens a network-reachable port. Remote control (D4) is an adapter in the *app* with its own authentication, never a direct line to REAPER.
- **Open spikes:** S1 (extension loads and runs on both targets), S2 (timing/transport), S3 (socket latency/throughput/reconnect).

### 2.3 State model (single definition in Rust, generated to TS; field names follow the glossary §14.2)
```
Live    { sequence, timestamp, extensionVersion, status: Stopped|Playing|Paused|Recording, position,
          performance: { setlistId|null, phase, currentEntry|null, nextEntry|null },
          flags { autoplay, countIn, recordArmed }, catalogRev, setlistRev }
Catalog { project{id,name}, songs[{id,name,start,end,colour}], cues[{id,name,pos}],
          directives { hardStops:[songId], lengths:{songId:sec}, bpms:{songId:bpm} },
          tempo{bpm,timeSig}, setlists[{id,name,entries:[{id,songId}]}] }
phase = Idle | Playing | Paused | CountingIn | HardStopped | HandingOver | Finished   // the ONLY phase vocabulary
```
`phase` is produced by the Performance state machine (§3). The UI renders it; it never infers it.

### 2.4 Commands (the *only* mutation surface)
`Play, Pause, TogglePlay, Next, Previous, RestartSong, SeekSong(id), SeekCue(id), SeekPosition(sec), SetAutoplay(b), SetCountIn(b), SetRecordArm(b), SelectSetlist(id|null), SaveSetlist(setlist, expectedRev), DeleteSetlist, RefreshCatalog`. (Entry add/remove/move are edits inside `SaveSetlist`, validated by the Setlists context; one write path.)
UI buttons, keyboard, MIDI and (future) remote all call `dispatch(Command)` in the Rust core. Nothing else talks to REAPER.

### 2.5 Fallback design (only if Gate 1 rejects the extension)
Lua companion script (`reaper-script/`, pure `core/` + thin `shell/`) with the app talking to REAPER's built-in web interface through ExtState (live/catalog/events keys, command mailbox), password-protected web interface, and the performance rules hosted in Rust through `mlua` (the former decision D5). Everything else in this spec (state model, commands, DDD contexts, UI) is unchanged. The fallback is documented in `docs/adr/ADR-002-fallback.md` and costs about +10 days if activated.

## 3. Performance state machine (single implementation: Rust `performance` crate, decision D7)
The same crate is compiled into the extension (production) and into the app (simulator, dry-run, projections, tests).
Phases: `Idle, Playing, Paused, CountingIn, HardStopped, HandingOver, Finished` (same vocabulary everywhere).
Rules (ported from v1, now deterministic):
1. Playing in a setlist and `time_to_end ≤ lead` → hand-over. `lead` is derived from the trigger's measured granularity (REAPER timer tick ≈ 33 ms, or the audio block if an audio-hook trigger is used) plus seek execution time (measured in S2). `handOverLeadMs` (default 0) stays as an advanced offset for parity with v1's transition offset. If S2 proves native region-playlist playback, hand-overs become native and the lead disappears.
2. Next entry exists → seek-while-playing to `next.start` (no pause/resume; falls back to stop/seek/play if REAPER requires it). If the next song is physically contiguous, do nothing.
3. Song has `!1008` → stop at the hard stop (song end or `!length`) → `HardStopped`; Play resumes into the next entry.
4. No next entry → pause → `Finished`.
5. Count-in (cue jumps only, per F10): seek to `target − 2 bars` (tempo map), enable count-in, play → `CountingIn` → `Playing`.
6. Manual Next/Previous: honour setlist; autoplay flag decides whether to resume playing; never counts in.
7. Debounce: ignore a hand-over if one fired < 1 s ago (kept from v1).
**Threading rule:** REAPER API calls happen on REAPER's main thread (timer callback). An audio-hook may only read atomics and enqueue a request on a lock-free queue for the main thread; it never allocates, locks or calls the REAPER API. **Open spike S2:** timer-driven vs audio-hook-triggered vs native region playlist hand-over.

## 4. Special markers — one parser
One grammar, implemented once in Rust (`performance`/`catalogue` shared crate). The extension parses marker and region names and publishes the parsed directives in the catalog; the app uses the same crate for the marker helper (Help screen) and tests. Tokens `!1008` (or its alias `!hardstop`), `!length:<float>`, `!bpm:<float>` in any order, whitespace-separated, in a marker or region name. v1's six-way regex cascade is replaced by tokenise-and-classify. A marker consisting only of tokens is hidden from the timeline. **Placement:** `!1008` counts when the marker lies anywhere inside the song's window; `!bpm` is read at the song's start edge (the tempo at start + 0.00002 s), so put it at the very start of the song. **SWS compatibility (owner requirement):** `!1008` is the SWS marker-action syntax (`!<action id>` runs that REAPER action when playback passes the marker), so it must stay compatible with SWS and projects must behave the same with or without SWS installed. `!length:N` and `!bpm:N` are Reaper Control's own directives, not SWS; they must never collide with an SWS marker-action name. **Alias (owner):** `!hardstop` does the same as `!1008` and does not depend on SWS, so it is the preferred spelling in new projects. (`!stop` is not added.) Open question for a later spike or WP: with SWS installed, SWS itself pauses at the marker, so the extension must not also act on it twice; the exact interplay is not yet verified.
Test vectors live in `testing/vectors/markers.json` (done, WP 2.2). Tokens are whole whitespace-separated words; v1 matched substrings.

## 5. App (Tauri 2 + Rust + Svelte 5)
- **Why Tauri:** ~10 MB installer, native webview (WKWebView on macOS, WebView2 on Windows), Rust for the link/MIDI/queue, memory-safe core.
- **Crates:** `tokio`, `serde`, `midir`, `tracing`, `tauri`, `specta`/`ts-rs` (type generation); extension side: `reaper-rs` (+ `std` threads and `std::net`, deliberately no async runtime inside REAPER's process).
- **Modules:** organised per bounded context, see §14.5 (this list is superseded).
- **Frontend:** one Svelte `appState` store fed by a single `state` event; no per-feature stores duplicating it. UI-only state (selected view, form drafts) separate.
- **Responsiveness:** optimistic *pending* indicators (button pressed state) within 1 frame; real state replaces them on the next State. No global button lockout; the queue de-dupes and orders instead.
- **Config:** `serde` struct with defaults and validation; atomic write (temp + rename) — replaces 150 lines of fsync/verify.
- **Setlists:** runtime copy stored in the REAPER project (project ExtState, written and read by the extension), edited only via `SaveSetlist` with an expected revision (optimistic concurrency). The app-side JSON mirror is for *restore offers only* (e.g. project opened without setlists), never auto-overwrites.
- **Migration:** import v1 `config.json` and `setlists/<projectId>.json` on first run.

## 6. Stage-safety requirements
- S-1 App crash does not interrupt playback (hand-overs, hard stops, count-ins all run inside the extension). S-2 App restart reconnects and resumes without interrupting.
- S-3 Link loss shows a persistent banner, never stale numbers presented as live.
- S-4 No modals or sleep during playback; wake-lock while connected. (2.0 has no auto-updater; an extension update never happens while REAPER is running a show.)
- S-5 Debounce double taps; hold-to-confirm for Next/Previous mid-song is a v2.1 option (default off for parity).
- S-6 Every *performance* action is operable by MIDI alone (settings/wizard excluded).
- S-7 Pre-show checklist (extension version, regions found, setlist valid, MIDI present).
- S-8 Hand-over journal written **by the extension** (file in REAPER resource dir), because the app may be dead during a show; the app imports it for review.
- **S-9 The extension must never take REAPER down** (the price of a native plug-in):
  1. Every entry point REAPER calls is wrapped in `catch_unwind`; the crate builds with `panic = "unwind"`, a global panic hook records the fault and switches the extension to *Faulted* (stops acting, keeps REAPER alive, reports `ExtensionFaulted`).
  2. `#![deny(unsafe_code)]` except in one small FFI module; `clippy::unwrap_used`, `expect_used`, `indexing_slicing` denied in the extension crate; no blocking calls on REAPER's main or audio thread.
  3. The protocol parser is fuzzed; all inputs are length- and range-checked.
  4. **Safe mode:** the extension writes a `running` marker at start and removes it at clean shutdown. If a marker is found at startup (REAPER crashed), the extension starts *disabled* and the app tells the user, who can re-enable it with one click.
  5. Soak and fault-injection tests (PLAN Phase 7) are release gates.

## 7. Testing
- Rust unit and property tests for the `performance`, `catalogue`, `setlists` domain crates (state machine, directives, count-in maths, hand-over invariants).
- **`ReaperPort` trait** abstracts everything the extension needs from REAPER (position, transport actions, markers/regions, tempo map, ExtState). Two adapters: `ReaperRsAdapter` (real) and `FakeReaper` (deterministic playhead and clock, scenario files). Scenarios replay identically in the extension crate's tests, the app's simulator and the dry-run.
- **Protocol conformance tests:** app Link client against the real extension server code running over `FakeReaper` (reconnect, replay, duplicate/out-of-order commands, garbage input, slow client).
- **In-REAPER tests:** a scripted REAPER project with a click track measures real hand-over gaps and timing on both targets.
- Fault injection: kill app mid-song, kill extension thread, panic injection, corrupt endpoint file, REAPER crash then restart (safe mode).
- Soak test: 2 h simulated set, 40 songs, random link drops and app kills, no missed hand-overs.
- Manual checklist per release on macOS (Apple Silicon) and Windows x64.

## 8. Milestones
Superseded: the schedule, gates and work packages live only in `PLAN.md` (single source).

## 9. Risks
Full register: PLAN §15. Headlines: the extension can crash REAPER (S-9 mitigations, safe mode, soak tests); REAPER must be restarted to load or update the extension; architecture mismatch (REAPER installed as the Intel build and running under Rosetta on an Apple Silicon Mac); macOS signing/quarantine behaviour; `reaper-rs`/REAPER API drift; seek-while-playing glitches (S2 → region playlist or pre-roll); unstable song identity (S6); webview differences macOS/Windows (S5).

## 10. Screens to design (Claude Design)
1. Player (transport, region/setlist list, timeline, toggles)  2. Setlist editor  3. **Performer (stage)**  4. Settings (Connection, MIDI, Behaviour)  5. First-run wizard  6. Connection/health states (Connected, Lost, Extension missing/outdated/faulted)  7. Pre-show checklist  8. Marker help/editor.

## 11. Design patterns & code-structure rules (maintainability)

Rule of thumb: a pattern is used only where it removes a concrete v1 problem. Each row names the problem it solves.

| Pattern | Where | v1 problem it removes |
|---|---|---|
| **Command** | Every mutation is a `Command` value (serde enum), carrying an id; handled by one `CommandHandler` per variant | Actions triggered through several routes (UI, MIDI, keyboard, IPC handler, services) |
| **Mediator / Command bus** | `dispatch(cmd)` is the single entry; UI, MIDI and keyboard know only the bus, not each other or REAPER | `ipcHandler.ts` 958 lines wired to every service |
| **State machine (State pattern)** | Performance `phase` with a pure `transition(state, event) -> (state, effects)` function in the shared Rust `performance` crate (extension + app) | Playing/transitioning/hard-stop logic scattered over 4 files and 2 timers |
| **Reducer / Unidirectional data flow** | App side only: read-model projections `reduce(View, Event) -> View` from extension pushes/events; UI renders them, never mutates | Duplicate stores and local timers drifting from REAPER |
| **Strategy** | `Driver` trait with `ExtensionDriver` and `FakeDriver` (simulator/tests); a `HandOverStrategy` only if S2 proves two strategies are both needed (otherwise a plain function; anti-goal below) — chosen by ADR-005, never a user setting | Hard-wired HTTP polling; latency-guess hack |
| **Port / Adapter / Anti-corruption layer** | `ReaperPort` trait (real `ReaperRsAdapter`, `FakeReaper`); the app's `link/` ACL translates protocol messages to domain events; nothing outside sees the wire format | String parsing (`TRANSPORT\t…`) mixed into business logic |
| **Observer / Pub-Sub** | One typed event stream (`StateChanged`, `LinkChanged`, `MidiActivity`, `Log`) to the UI and logger | 35 ad-hoc IPC channels + duplicate channel tables |
| **Repository** | `SetlistRepository`, `ConfigRepository` traits (REAPER-project impl, JSON-file impl, in-memory test impl) | Storage logic inside `ProjectService` |
| **Factory / Builder** | `StateFactory` for tests and simulator; `ConfigBuilder` with validation | Duplicated `playbackStateFactory` (main + renderer) |
| **Parser (Interpreter-lite)** | One tokenising marker grammar returning `Vec<MarkerToken>` | Regex cascade in `markerUtils.ts` |
| **Decorator** | Link client middleware: `Timeout`, `Metrics(latency)`, `Logging` around the socket client | Retry/latency/logging tangled inside `makeRequest` |
| **Circuit breaker / backoff** | In the link client: reconnect with back-off, health state machine, no retry stacking | Retries (3×1 s) layered on a 150 ms poll |
| **Dependency Injection** | Constructors take traits; one composition root (`main.rs` / `bootstrap`) wires everything | Global singletons (`config`, `logger`) and `if (service)` null checks |
| **Null Object** | `NoopDriver`, `NoopMidi` when absent so callers never branch on null | `x \| null` service fields in `index.ts` |
| **Facade** | UI talks to one `ReaperClient` API (`state`, `send`) | Components importing `ipcService`, stores and services directly |
| **Registry** | MIDI action table: `action name → Command` registry, user mapping is data not code | `switch` on action strings |
| **Specification** | Reusable predicates (`canGoNext`, `isHardStopRegion`) tested once | Same checks repeated in region/setlist/performer code |

### Structure rules
1. **Hexagonal layering inside each bounded context** (see §14.5 for the crate layout): `domain` ← `application` ← infrastructure. Dependencies point inward only; enforced with a CI check.
2. **Pure core, thin shell.** All decisions live in pure functions testable without REAPER, a clock or the OS. Time is injected (`Clock` trait).
3. **One source for each fact.** Shared types generated from Rust (`specta`/`ts-rs`); marker test vectors shared by the extension and app crates. No hand-copied interfaces.
4. **Make illegal states unrepresentable.** Enums with data instead of boolean flag clusters (`Playing{..} | HardStopped{..}`), newtypes for `RegionId`, `Seconds`, `Bpm`.
5. **Errors are values.** One `AppError` enum (`thiserror`), mapped once to user messages; no `catch` that logs and continues.
6. **Small units.** One type per file (each `struct`/`enum`/`trait` with its `impl`s in its own file, the parent module only declares and re-exports; one responsibility per type; a type's private helpers live in a folder named after it; owner's rule). Soft limits: files < 300 lines, functions < 40, no component > 200 lines of script; UI built from a small shared component kit (Button, Toggle, Panel, Timeline, StatusBadge) — v1's 700–900-line components are the counter-example.
7. **Frontend:** presentational components get props only; one container per screen subscribes to the state store. No business logic in `.svelte` files.
8. **Tooling:** `clippy -D warnings`, `rustfmt`, `eslint` + `svelte-check` strict, `cargo-deny`, pre-commit and CI gates; ADRs in `docs/adr/` for each non-obvious decision.
9. **Documentation:** each module has a header explaining responsibility; the state machine has a generated diagram that CI keeps in sync.

### Anti-goals (to avoid over-engineering)
No plugin system, no generic event sourcing, no DI container framework (plain constructor injection), no abstraction without two real implementations or a test double need.

## 12. Decisions log (owner, 2026-10-06)
| ID | Decision | Consequence |
|----|----------|-------------|
| D1 | **Public release** | macOS + Windows (Linux deferred, D6), code signing/notarisation, installers, user docs, issue process, licence review; update mechanism deferred to v2.1 (when added: never while a show is running) |
| D2 | **Setlists live in the REAPER project** (extension-owned); app keeps a backup mirror | Extension must persist via project ExtState; app edits via commands; import v1 files |
| D3 | **Companion component required, no Basic mode** (the component is the Rust extension per D7) | One playback path; first-run wizard must install/verify it; read-only mode when absent |
| D6 | **v2.0 ships on macOS + Windows only** (macOS is the development platform); Linux, auto-updater, website, Dutch translation, MIDI learn and hold-to-confirm move to the v2.1 backlog | Smaller QA matrix; manual GitHub releases for 2.0; code stays portable (no OS-specific shortcuts in domain/application) |
| D5 | ~~**Performance rules are written once, in Lua**~~ **Superseded by D7; kept as the fallback design (§2.5).** Original: rules written once, in Lua (pure `core/` in the REAPER script) and hosted in Rust via `mlua` for simulator/tests/dry-run (REVIEW-1 A1) | No Rust re-implementation of the state machine, marker parser or count-in maths; no cross-implementation tests |
| D7 | **Native REAPER extension in Rust is the primary companion** (`reaper-rs`), replacing Lua script + REAPER web interface. Targets: macOS Apple Silicon (arm64) and Windows x64 only; Intel Macs are not supported (D8). Performance rules written once in Rust, shared by extension and app. Gate 1 confirms; fallback §2.5 | No web interface setup or password; local push socket; installer = copy one file + restart REAPER; crash-containment requirements S-9; extension update needs REAPER restart |
| D8 | **Intel Macs are not supported** (macOS Apple Silicon only; Windows x64) | Two extension builds instead of three; Intel REAPER under Rosetta is detected and guided, not supported |
| D4 | **Remote control from other devices: nice-to-have** | Planned as stretch milestone M7 (local web server + touch UI) on the same command bus; not in v2.0 scope |

## 13. Extension installer (requirement R-INST, owner priority)
**Requirement:** the app installs the companion extension into REAPER itself, verifies it works, and — where automation is impossible — tells the user exactly what to do. The user never has to hunt for files.

### 13.1 Flow (`installation` context, port `ExtensionInstaller`)
1. **Locate REAPER:** find the resource path (macOS `~/Library/Application Support/REAPER`, Windows `%APPDATA%\REAPER`, portable installs via user-chosen folder; several installs → the user picks) and the REAPER executable. Detect if REAPER is running.
2. **Detect REAPER's architecture** (not the machine's): inspect the REAPER binary (Mach-O arm64, PE x64). If REAPER on an Apple Silicon Mac is the Intel build (running under Rosetta), the extension cannot load into it: the wizard shows a guidance card ("install the Apple Silicon version of REAPER", with the download link) instead of installing. Bundled builds: `reaper_rc2-arm64.dylib` and `reaper_rc2-x64.dll`.
3. **Inspect:** is an extension installed? which version? Result = `InstallReport` with each item `Ok | Fixable | Manual | Unknown`.
4. **Install** (idempotent, reversible): copy the extension into `<resource>/UserPlugins/`; on macOS verify the code signature and remove the quarantine attribute from the copy; keep the previous version in `RC2-backups/` for rollback. **No REAPER settings files are edited.**
5. **Activate:** REAPER loads extensions only at start. REAPER closed → offer to launch it. REAPER running → show a *Restart REAPER* instruction card (the app never closes or restarts REAPER itself). On Windows a loaded DLL cannot be replaced: updates are staged and swapped in when REAPER is not running.
6. **Verify:** success only when the extension answers the handshake with the expected version (`Welcome`). Copying the file does not count.
7. **Report / fallback:** anything that fails or can't be automated → plain-language instruction card with the exact path (copy button, "reveal in Finder/Explorer") and a "Check again" button. Specific advice for permission errors, antivirus quarantine, REAPER architecture mismatch, and *safe mode* (the extension started disabled after a REAPER crash, S-9.4).

### 13.2 Lifecycle
- **Update:** the app bundles the extension; if the installed version is older it offers "Update extension", never while REAPER is running a show; applies at the next REAPER start.
- **Repair:** Settings ▸ "Repair installation" re-runs the inspection and fixes drift (REAPER reinstalled, file removed, wrong architecture).
- **Uninstall:** removes the extension file (and optionally RC2 data); restores the previous version on request.
- **Compatibility:** minimum REAPER version check; incompatible → clear message.
- **Safety:** never overwrites files we don't own; dry-run shows what would change; every change is logged in the diagnostics bundle.

### 13.3 Risks / unknowns (resolved by spike S4, see PLAN)
- macOS: whether an ad-hoc-signed or Developer-ID-signed dylib copied by the app loads in REAPER without prompts (Gatekeeper, library validation, quarantine), on Apple Silicon.
- Windows: SmartScreen or antivirus reactions to an unsigned or signed DLL; file locking while REAPER runs.
- REAPER versions: which REAPER versions load `reaper-rs`-based extensions and expose the needed API functions.
- Portable installs and non-default resource paths.

## 14. Domain-Driven Design (the layout in §5 and §11 rule 1 is superseded by §14.5)

### 14.1 Strategic design
**Core domain:** *Performing a set live* — turning a setlist into a gapless, stage-safe sequence of songs. This is where the modelling effort and the best tests go.
**Supporting subdomains:** Setlist Management, Song Catalogue (regions/markers), Control Surfaces (MIDI/keyboard/remote), Link Health.
**Generic subdomains:** Installation, Configuration, Diagnostics/Logging, Updates (kept simple, buy-not-build where possible).

**Bounded contexts** (each is a Rust crate with its own model; no shared "god" types):
| Context | Responsibility | Owns |
|---|---|---|
| **Performance** (core) | Plays a setlist: transitions, hard stops, count-in, finish | `Performance` aggregate, state machine |
| **Setlists** | Create/edit/validate setlists | `Setlist` aggregate |
| **Catalogue** | Songs as seen in the REAPER project: regions, markers, special-marker meaning | `Song`, `Marker`, `SpecialMarkers` |
| **Control** | Turning intent from any device into commands | `ControlBinding`, `Controller`, device registry |
| **Reaper Link** | Talking to the extension; connection health | `LinkSession` aggregate |
| **Installation** | Getting the extension into REAPER and keeping it healthy | `Installation` aggregate |
| **Settings** | User preferences | `Preferences` aggregate |
| **Diagnostics** | Logs, transition journal, support bundle | `Journal` |

**Context map**
- Performance ⟵ **Customer/Supplier** ⟵ Catalogue (Performance consumes songs) and Setlists (supplies ordered entries).
- Setlists ⟵ **Conformist** to Catalogue's `SongId` (a setlist entry references a song by identity only).
- Reaper Link ⟶ **Anti-Corruption Layer** to every context: the wire protocol (§2.2) never leaks; the ACL translates messages to domain events/types.
- Control ⟶ **Open Host Service**: publishes `Intent`s; Performance and Setlists handle them via the command bus.
- Installation is **Separate Ways** from Performance (no model sharing), integrating only via the Link handshake.
- A **Shared Kernel** is deliberately tiny: `Seconds`, `Bpm`, `SongId`, `SetlistId`, `ProjectId` newtypes.

### 14.2 Ubiquitous language (glossary, binding for code, UI copy, docs)
| Term | Meaning | Not to be called |
|---|---|---|
| **Song** | A region in the REAPER project that can be performed | "track", "item" |
| **Setlist** | Ordered list of **Entries** referencing songs, belonging to a project | "playlist" (that is the *mode*) |
| **Entry** | One position in a setlist (song reference + optional per-entry settings) | "item" |
| **Performance** | A running pass through a setlist | "session" |
| **Cue** | A marker the performer can jump to | "marker" when meant as navigation |
| **Directive** | A special marker instruction (`!1008`, `!length`, `!bpm`) | "command" |
| **Hard stop** | Directive: performance halts at song end (or `!length`) until the performer resumes | "pause" |
| **Hand-over** | The moment control passes from one song to the next | "transition" in UI copy |
| **Count-in** | Two bars of click before a song starts | |
| **Intent** | What a person wants (from UI/MIDI/key/remote) before it's validated | "command" |
| **Command** | A validated instruction on the bus | |
| **Link** | Connection to REAPER; **Extension** = companion extension | "script", "plugin" |
Glossary is a living doc (`docs/language.md`) and CI-linted for banned synonyms in identifiers where practical.

### 14.3 Tactical design (per context; core shown in detail)
**Performance context** *(single implementation: Rust `performance` crate, compiled into the extension and the app)*
- **Aggregate root `Performance`** — invariants: at most one current song; position never leaves the current song's bounds except via a hand-over; a hard stop always halts; no hand-over fires twice for one song end; finishing is terminal until restart.
- **Value objects:** `Phase` (`Idle | Playing | Paused | CountingIn | HardStopped | HandingOver | Finished`), `Position`, `SongWindow { start, effective_end }`, `CountIn`, `PlaybackFlags`.
- **Domain events** (published on the extension's event stream and journal): `PerformanceStarted`, `HandOverStarted`, `HandOverCompleted`, `HardStopReached`, `PerformanceFinished`, `FlagChanged`, `SeekPerformed`, `CommandRejected`.
- **Domain services:** `HandOverPolicy`, `EffectiveEnd` (`!length`/hard stop), `CountInPlanner` (bars→seconds from the tempo-map snapshot). **Specifications:** `CanAdvance`, `IsAtHardStop`, `IsSongContiguous`.
- **Purity rule:** the crate has no I/O and no REAPER calls. It receives a `Snapshot` + `Clock` + `Command` and returns `(state, effects, events)`. The extension's `ReaperPort` adapter executes the effects.
- **In the app** the same crate provides the `PerformanceView` projection, `IntentValidator` (is `Next` allowed now?) and the simulator/dry-run (with `FakeReaper`). The *authoritative running instance* is always the one in the extension.

**Setlists context** *(editing rules are authoritative in the shared Rust crate; the extension stores the saved setlist in project ExtState, plays it, and re-validates on load)* — aggregate `Setlist` (invariants: unique entry ids, no entry for unknown song when *activated*, order is contiguous; editing is allowed with dangling entries but `validate()` reports them). Events: `SetlistCreated/Renamed/EntryAdded/EntryRemoved/EntryMoved/Deleted`. Repository `SetlistRepository` (impl: Reaper project ExtState via Link ACL; in-memory for tests).

**Catalogue context** — read model of the project: `Song` (id, name, window, colour), `Cue`, `Directives` value object **parsed by the shared Rust grammar** (§4) and delivered in the catalog. Rebuilt from Link snapshots (event-driven projection, no mutation by UI).

**Reaper Link context** — aggregate `LinkSession` (invariants: one connection to the extension; health transitions follow `Connected → Degraded → Lost → Dead`; reconnect with back-off and event replay from the last seen event id). Events: `LinkEstablished/Degraded/Lost/Recovered`, `ExtensionNotLoaded/ExtensionOutdated/ExtensionFaulted`. Owns the **ACL** (translator classes `MessageTranslator`, `CommandTranslator`).

**Installation context** — aggregate `Installation` (the *desired vs actual* installation of one REAPER install). Value objects: `ReaperHome`, `InstallStep`, `StepOutcome(Ok|Fixed|Manual{instructions}|Failed)`. Domain events: `InstallInspected`, `StepApplied`, `InstallVerified`, `ManualActionRequired`. Domain service `InstallPlanner` (inspection → ordered plan). Ports for file system/process (pure planner is testable without disk).

**Control context** — `ControlBinding` (source→`Intent`), `Controller` (keyboard, MIDI device, remote client). Anti-duplication: all controllers produce the same `Intent`; one `IntentTranslator` validates against current Performance/Setlist state and yields `Command`s.

**Settings / Diagnostics** — simple CRUD-style, deliberately *not* over-modelled (anemic is acceptable for generic subdomains).

### 14.4 Application layer & integration
- **Application services** orchestrate one use case each. Where the app is authoritative (Setlists, Installation, Link, Settings): load aggregate → behaviour → publish events. For **Performance** the service only validates the intent against the projection and sends the command to the extension; the resulting events come back over the event stream. They contain no business rules.
- **Domain events** are the *only* cross-context signal (in-process event bus); handlers in other contexts translate to their own models (no cross-context method calls).
- **CQRS-lite:** commands mutate aggregates (in the extension for Performance, in the app elsewhere); the UI reads **read models/projections** (`PlayerView`, `PerformerView`, `SetlistView`) built from events. This is also what makes the UI fast and keeps Svelte free of business logic.
- **Event sourcing is NOT used** (anti-goal). Aggregates persist as state; the transition journal is an append-only *log*, not the source of truth.
- **Transactions/consistency:** one aggregate per command; cross-aggregate effects via events (eventual consistency inside one process, in-order).
- **One implementation of the performance rules** (decision D7): the Rust `performance` crate. The extension runs it in production; the app embeds the very same crate for the simulator, dry-run and tests, so nothing is copied or ported.

### 14.5 Code layout (replaces §5/§11 layout)
```
crates/
  shared-kernel/            Seconds, Bpm, SongId, SetlistId, ProjectId
  performance/  {domain, application(host-agnostic), projections}   -- shared by extension and app
  setlists/     {domain,application}
  catalogue/    {domain,application}                  -- directive grammar, song/cue read model
  protocol/                 wire messages + framing (shared by extension server and app client)
  reaper-port/              ReaperPort trait + FakeReaper
  reaper-extension/         cdylib: ReaperRsAdapter, server, timer/audio hooks, journal, FFI guard (S-9)
  control/      {domain,application}
  reaper-link/  {domain,application,acl}              -- app-side client of `protocol`
  installation/ {domain,application}
  settings/ diagnostics/
  infrastructure/           adapters: socket client, midi, fs, tauri, repositories, event-bus impl
  app/                      composition root only
```
(Fallback §2.5 would add `reaper-script/` and an `mlua` host crate.)
Rules enforced in CI: `domain` crates depend only on `shared-kernel`; no context depends on another context's `domain` (only on published **events/contracts** crates, i.e. `protocol`); `reaper-extension` depends only on `shared-kernel`, `performance`, `catalogue`, `setlists`, `protocol`, `reaper-port` (no Tauri, no tokio); infrastructure depends inward; `app` wires everything.

### 14.6 Modelling process
Event Storming workshop (solo + Claude) per context → timeline of domain events → aggregates/invariants → glossary → context map ADR. Models live in `docs/domain/*.md` with an invariant table per aggregate, each invariant mapped to a named test.

### 10b. Design status (2026-10-06)
Canvas (private, Claude Design): https://claude.ai/artifact/X3zo5xvtRT58fPWjRPtXsH
Designed: Performer (unchanged from v1 look, plus hard-stop state), Player, Setlists, Settings, First-run setup (manual step card), Connection/script states, Pre-show check, shared Sidebar.
Owner feedback: approved ("awesome, better usable"). Performer must stay as in v1.
Not yet designed: marker help/editor, Help screen, Settings extras (logging/diagnostics, appearance, About, shortcuts), remaining wizard steps (1, 2, 4), setlist create/rename dialogs, empty states.

### 10c. Design gaps found in review 2 (v1 details missing from the canvas)
**Must add (v1 behaviour or parity):**
1. **Timeline click-to-seek time popover** (F18) — small time bubble above the click point; also **clicking near a cue snaps to that cue** (v1 click threshold) and then follows the count-in-on-cue rule.
2. **Settings has no Save pattern** — v1 had Save Changes + saving/success/error messages and a port-change hint. Decide: autosave with a "Saved" indicator (proposed) or explicit Save.
3. **"All songs" option** in the Player's setlist selector (v1: "All Regions"), plus the selector's loading state.
4. **Empty and loading states:** "Loading from REAPER…", no songs in project (Refresh/Retry), empty setlist, no setlists yet.
5. **Delete-setlist confirmation dialog** and inline create/rename (v1 had a confirm modal).
6. **Non-modal status messages** (info/warning/error, dismissible, auto-clearing) — v1 statusMessage.
7. **Performer "End of setlist"** state (no next song) and play disabled at the very end; hard-stop red flash animation (2 s pulse, off under reduced motion).
8. **Status cluster popovers:** connection popover (last ping, latency colour <100 good / <300 medium / worse, "Reconnecting, attempt N of max", Reconnect) and system-stats popover (CPU, memory, project id, latency, versions); **MIDI activity indicator** lighting up on input.
9. **Help screen** (v1: set-up steps, markers, special markers, MIDI, troubleshooting) and the **marker help/editor**.
10. **Keyboard shortcuts** visible somewhere (space, ←/→, a).
11. **Record-arm ON state** (red) on Player and Performer; **pending state** for buttons awaiting extension acknowledgement (replaces v1's 2 s lockout).
12. **Setlist reordering without drag:** keep v1's up/down buttons (accessibility), drag as an extra.

**Consistency fixes:**
- Count-in label: Performer keeps v1 text "Count-in when pressing marker"; Player currently says "Count-in on cues" and "Auto-resume" vs v1 "Auto-resume playback". Align user-facing copy to v1 wording (glossary term "cue" stays internal).
- Muted text (#6F767E, wizard future steps #7D848C) is below 4.5:1 on the dark surfaces; raise to about #8B929A.
- Performer at smaller window sizes: v1 scaled type down at <1024 and <768 px; define the scaling rules (also needed for the D4 tablet remote).

### 10d. Design decisions after review 2 (owner picked items 1, 3, 4, 5, 6, 7, 8, 9 of §10c)
Designed on the canvas: timeline time bubble (Performer, Player); "All songs" option in setlist selector; loading/empty states; delete-setlist confirmation; non-modal messages (info 2 s, warning 10 s, error until dismissed); Performer end-of-setlist state; status cluster + connection and system-status popovers incl. MIDI activity; Help screen with marker name helper; copy aligned to v1 wording; muted text contrast raised.
Deferred by owner for now: Settings save pattern (§10c item 2), keyboard-shortcut hints (10), pending/record-ON states (11), up/down reorder buttons (12). Still open: wizard steps 1, 2, 4, small-window scaling of Performer.

### 10e. Design impact of D7 (screens to update on the canvas)
- **Wizard:** steps become Find REAPER → Install extension → Restart REAPER → Connect. The manual web-interface/password card is removed; the manual step is *Restart REAPER*.
- **Settings:** Connection card (host, port, Test connection) becomes status + "Repair installation"; no password field.
- **Connection states:** "Script missing/outdated" → "Extension not loaded / outdated / faulted (safe mode, one-click re-enable)".
- **Pre-show check:** "Web interface protected" row removed; "Extension up to date" kept.
- **Sidebar badge, Help, Settings copy:** "script" → "extension"; Help troubleshooting entries updated.
