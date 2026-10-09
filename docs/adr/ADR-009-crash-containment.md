# ADR-009: Crash containment and safe mode (S-9)

Status: Proposed. Date: 2026-10-07. Spike S7 (WP 1.7), code in `spikes/s7-crash`.

## Question
Can a panic in our extension be kept from taking REAPER down, how does the extension notice it is broken, and how does safe mode (SPEC S-9.4) learn that REAPER did not shut down cleanly?

## Setup
macOS arm64, REAPER 7.82, isolated instance, `reaper-rs` rev 659b22b. Spike extension with fault injection by trigger files, a panic hook that sets a global `FAULTED` flag, a `s7-running` marker, and a 30 ms control-surface tick. Position and tick counters are logged every 3 s. Two builds: `panic = "unwind"` and `panic = "abort"`.

## Results
| Test | Result |
|---|---|
| Panic inside the main-thread tick, unwind build | REAPER stayed alive. Hook logged the panic with file and line, extension went Faulted (`alive_ticks` frozen at 234), REAPER kept calling the tick (`calls` kept growing), play position kept advancing 3.0 s per 3 s |
| Panic on a worker thread, unwind build | Thread died, REAPER alive, the same hook faulted the whole extension, tick idle afterwards |
| Same panic in the abort build | REAPER process died at once. The marker stayed |
| Start with a stale marker (after the abort death and after SIGTERM) | `SAFE MODE` logged, extension stayed disabled, no ticks |
| Clean quit (Cmd+Q equivalent) | `close_no_reset` not called, `Drop` of the control surface not called, marker stayed. A C `atexit` handler ran and removed the marker |
| SIGTERM to REAPER | No hook ran, marker stayed (treated as unclean) |
| Panic in the audio hook (second round, 2026-10-07), unwind build | REAPER alive, hook logged the panic and set Faulted, the audio callback kept being called (`audio_calls` 1414 to 3110) and play position kept advancing. Clean quit afterwards removed the marker |
| Panic in the project-marker callback while `Main_openProject` ran (project load), unwind build | REAPER alive, `Main_openProject` returned, Faulted set, ticks of the extension idle afterwards |
| Real SIGSEGV (null write) in the main-thread tick | REAPER died at once, no panic hook ran, no Faulted line, marker stayed, REAPER wrote a crash report. Next start logged `SAFE MODE` and stayed disabled |

`reaper-rs` already wraps its entry point, control-surface callbacks, hooks and similar in `catch_unwind` (`firewall` in `reaper-low/src/util.rs`). The containment in the first row therefore comes from the library plus `panic = "unwind"`.

## Proposed decision
1. `panic = "unwind"` is a hard requirement; CI must fail if the extension profile sets `abort`. Without it, one panic kills the show.
2. Keep our own `catch_unwind` for everything REAPER or the OS calls that `reaper-rs` does not wrap (our threads, any raw FFI we add). A panic hook sets the global Faulted flag, logs, and the extension stops acting but stays loaded.
3. Safe-mode marker: write at start, remove from an `atexit` handler (not `close_no_reset`, not `Drop`). Windows (2026-10-09, Windows 11 VM, REAPER 7.82): the `atexit` hook never ran. `#[reaper_extension_plugin]` generates a `DllMain` whose unload branch calls `reaper_low::execute_plugin_destroy_hooks`; that touches a `Fragile` static and so `thread::current()`, which panics at process exit because the thread-locals are gone. The abort comes before the C runtime reaches `atexit`, so every clean quit left `running` behind (the next start would be safe mode) and wrote a false `faulted` file. Fix: `ReaperPluginEntry` is written out by hand and our own `DllMain` removes the marker with `DeleteFileW` (`unlink` for the `atexit` hook on macOS) from a path prepared at start, with no `std` file or thread calls. This extension registers no destroy hooks, so nothing else is lost. Seen after the fix: clean quit removes `running`, writes no `faulted`, logs no panic, and the next start logs `started`; a hard kill still gives safe mode; an injected panic is contained (REAPER stays up, `faulted` written).
4. A native crash (segfault, abort, other plug-ins) cannot be contained from inside; the marker is how the next start finds out.
5. API drift (`spikes/s7b-api-drift`): `reaper-rs` loads each REAPER function as an optional pointer and its convenience methods panic when the function is absent (documented in `reaper-low`). The extension therefore checks the whole list of functions it needs at load (`GetFunc` is null when absent) and refuses to start with a clear message instead of failing later in the tick. Measured on REAPER 7.82/macOS-arm64: all 21 functions of the planned list are present, and a made-up function and the SWS function `CF_GetSWSVersion` (SWS is not installed in the test config) are reported absent, so the check detects absence. The list is a placeholder until the performance core fixes its real API use. WP 3.2 (2026-10-08): the list is now the 19 functions the adapter and the start-up call (`required_functions.rs`), checked before the safe-mode marker is claimed; with two made-up names added, REAPER 7.82 logged the refusal and stayed alive.

## Not tested / open
- Windows: panic injection, hard-kill safe mode and clean-quit marker removal were checked on one VM (2026-10-09); `slow-main` watchdog, audio-hook panic and a real access violation are not.
- Protocol fuzzing (2026-10-07): the frame decoder and the message parser now exist in `crates/protocol` (`frame.rs`, `message.rs`) with two `cargo-fuzz` targets (`crates/protocol/fuzz`, nightly toolchain). 60 s each on macOS arm64: decoder 6,927,668 runs, messages 5,799,901 runs, no crash, no panic, no oversized frame, same result for any chunking. CI runs 30 s per target. Short runs, not a campaign: the long campaign stays in WP 7.7, and later message types must be added to the targets.
- Soak tests.
- API drift: owner decision (2026-10-07): only the newest REAPER build is supported (7.82 at the time of writing, the version all spikes ran on). No older builds are tested and no minimum version is derived. The load-time function check stays as the guard: an older REAPER that lacks a function is refused with a clear message. The supported version is stated per release.
- False positives: any REAPER crash or force quit, also one caused by another plug-in, disables the extension on the next start. Needs an owner decision on how aggressive safe mode should be and how the one-click re-enable looks.
- `is_playing` (checked 2026-10-07): it reads `false` with raw state 0 while REAPER is stopped and `true` with raw state 1 once playback runs (started with action 1007). The earlier `false` readings came from runs where nothing was playing; the position readout moved anyway, so position alone does not say whether REAPER plays. Use the play state.
- Logging from several threads is still unsynchronised.
