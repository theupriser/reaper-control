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

`reaper-rs` already wraps its entry point, control-surface callbacks, hooks and similar in `catch_unwind` (`firewall` in `reaper-low/src/util.rs`). The containment in the first row therefore comes from the library plus `panic = "unwind"`.

## Proposed decision
1. `panic = "unwind"` is a hard requirement; CI must fail if the extension profile sets `abort`. Without it, one panic kills the show.
2. Keep our own `catch_unwind` for everything REAPER or the OS calls that `reaper-rs` does not wrap (our threads, any raw FFI we add). A panic hook sets the global Faulted flag, logs, and the extension stops acting but stays loaded.
3. Safe-mode marker: write at start, remove from an `atexit` handler (not `close_no_reset`, not `Drop`). Windows equivalent still to verify.
4. A native crash (segfault, abort, other plug-ins) cannot be contained from inside; the marker is how the next start finds out.

## Not tested / open
- Windows: `atexit` and DLL unload behaviour, CI only compiles.
- A real native fault (SIGSEGV); `abort` stands in for it.
- Panic in the audio hook, panic during project load, protocol fuzzing, REAPER API drift between versions (second half of WP 1.7), soak tests.
- False positives: any REAPER crash or force quit, also one caused by another plug-in, disables the extension on the next start. Needs an owner decision on how aggressive safe mode should be and how the one-click re-enable looks.
- `get_play_state_ex(..).is_playing` read `false` in every run while the position advanced, so playback state must be read with care. To be checked before the performance core relies on it.
- Logging from several threads is still unsynchronised.
