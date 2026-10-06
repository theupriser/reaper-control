# ADR-002: Playback is owned by a native Rust REAPER extension

Status: **Proposed**. Spike S1 (macOS Apple Silicon half) is positive. Gate 1 stays open until S2, S3, S4, S7 and the Windows half of S1 are done.
Date: 2026-10-06. Related: SPEC D7, §2, §13; PLAN WP 1.1–1.8.

## Question (S1)
Can a Rust extension built with `reaper-rs` load into a current REAPER, run on REAPER's main thread, and read what the app needs (transport, position, tempo, regions, markers) correctly on a real project?

## What was run
- Spike crate `spikes/s1-hello-extension` (not part of the workspace), `reaper-low/medium/macros` from git rev `659b22b` of helgoboss/reaper-rs (medium-level API only).
- REAPER 7.78, then 7.82 (updated during the spike), universal binary, running as arm64 on Apple Silicon.
- A *separate* REAPER instance started with `-cfgfile <test dir>/reaper.ini`, so the real REAPER config is untouched. The extension was copied to `<test dir>/UserPlugins/reaper_rc2s1-arm64.dylib`.
- Projects: a small synthetic sample (`testing/projects/s1-sample.rpp`) and copies of two real live-set projects (kept out of git).
- The extension registers a control surface; REAPER calls its `run()` on the main thread. It logs the tick interval, play state, position, tempo, marker/region counts, and dumps every region/marker.
- Cross-check: the regions read by the extension were compared by script against what v1 reads over REAPER's web interface (`/_/REGION`) from a running REAPER with the same project.

## Results (measured)
| Question | Result |
|---|---|
| Does the dylib load? | Yes. `lsof` showed it mapped in the REAPER process. Built as arm64, ~640 KB, **ad-hoc (linker-signed)**; no signing step needed for this local load. Name must match `reaper_*-arm64.dylib`. |
| Main-thread tick | `ControlSurface::run()` average **30.0 ms** (≈33 Hz) over 25 windows of ~3 s; typical min/max 24–36 ms; worst window max **78.7 ms**. |
| Main thread blocking | During project load the tick stopped for **3.6 s** (one window reported max 3603 ms). Anything that runs on the main thread (timer or control surface) stalls while REAPER loads a project. Relevant for S2 and risk R12. |
| Reads | Play state, position (updates live), tempo at position (160 BPM read correctly), marker/region counts, and enumeration of every region/marker all work. |
| Parity with v1's data path | **12 of 12 regions identical** (id, name, start, end to 1 ms) between the extension and v1's web-interface read, on the same live-set project. A second project read 17 regions and 5 markers. |
| Isolation | `-cfgfile` makes the given folder REAPER's resource path. The real `~/Library/Application Support/REAPER` stayed untouched. |

## Findings that affect the design
1. **REAPER is a universal binary** (x86_64 + arm64). The installer must detect which architecture is *running*, not only what the file contains (S4).
2. **`reaper-rs` crates on crates.io are stale (0.1.0)**; use the git repo pinned to a revision. Its dependency tree includes a git fork of `nutype`; note for supply-chain review (`cargo-deny`).
3. **Library code can panic** (`require_valid_project`, `expect` in `get_resource_path`). `catch_unwind` at every entry point (S-9) is mandatory, not optional. Use `ProjectContext::CurrentProject` and avoid the unchecked variants.
4. `get_play_position_2_ex` is available on the audio thread (`AnyThread`), which keeps the audio-hook trigger idea (S2) open.
5. **Marker and region ids overlap** (a marker and a region can both have id 1), but every item has a GUID in the project file. Song identity must not be a bare numeric id (S6).
6. **Directive placement (data from real projects):** `!bpm:N` markers sit at the song's start edge (+0.00002 s); `!1008` sits *inside* the song, near its end (0.4–1.5 s before it); `!1008 !length:N` combine in one marker. v1 decides membership with `marker.position` anywhere in `[region.start, region.end]`. The "within 0.001 s of the boundary" sentence in v1's Help (copied into SPEC/Help design) is wrong and is being corrected separately.

## Not tested yet (so not claimed)
Windows x64; REAPER versions other than 7.78/7.82; Intel REAPER under Rosetta; playback and seeking (S2); the local socket (S3); signing/notarisation/quarantine of a *downloaded* dylib and the installer flow (S4); panic containment and safe mode (S7); behaviour with menus/modal dialogs open.

## Decision (so far)
Continue with the extension as the primary design (D7). The fallback (SPEC §2.5) stays documented until Gate 1.

## How to reproduce
See `spikes/s1-hello-extension/README.md`.
