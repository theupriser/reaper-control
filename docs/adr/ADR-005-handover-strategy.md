# ADR-005: Hand-over is a seek issued from the extension's main-thread tick

Status: **Proposed**. Spike S2 is positive for the timing, with one open item (below). Gate 1 stays open until it is closed.
Date: 2026-10-07. Related: ADR-002, SPEC §5 (hand-over), PLAN WP 1.2, risk R12.

## Question (S2)
When a song ends, how precisely can the extension start the next song, which method is best, and what can stall it?

## What was run
- Spike crate `spikes/s2-timing` (outside the workspace), REAPER 7.82 as a separate `-cfgfile` instance (arm64), synthetic click project `testing/projects/s2/` (120 BPM clicks, Song A 0–10 s, Song B 12–22 s, Song C 24–34 s).
- Every main-thread tick (about 30 ms) the extension samples `GetPlayPosition2Ex`. When `position + lead >= end of Song A` it jumps to the start of Song B. Two methods: **Seek** (`SetEditCurPos2` with seek-play, while playing) and **StopSeek** (stop, set cursor, play). Leads 0, 15, 30, 45 ms, 3 runs each (`run_matrix.sh`, `analyze.py`).
- **Cut error** = position at the trigger tick minus the end of Song A (negative = Song A cut early). **Delay** = time from the trigger tick until Song B is seen moving, from its start position.

## Results (measured, REAPER's own position readout, not recorded audio)
| Method | Lead ms | Cut error ms (min/mean/max) | Delay to Song B ms (mean) |
|---|---|---|---|
| Seek | 0 | +10.7 / +16.0 / +21.3 | 63 |
| Seek | 15 | −10.7 / −7.1 / 0.0 | 62 |
| Seek | 30 | −21.3 / −10.7 / 0.0 | 62 |
| Seek | 45 | −32.0 / −28.4 / −21.3 | 59 |
| StopSeek | 0 | 0.0 / +10.7 / +21.3 | 50 |
| StopSeek | 15 | −10.7 / −3.6 / 0.0 | 53 |
| StopSeek | 30 | −10.7 / −10.7 / −10.7 | 62 |
| StopSeek | 45 | −42.7 / −35.6 / −32.0 | 65 |

- Position moves in 10.7 ms steps (512 samples at 48 kHz); that is the resolution of these numbers.
- The seek call itself takes about 0.03 ms. The delay of 50–70 ms is audio-buffer and output latency.
- **Seek and StopSeek are indistinguishable** within the spread. Seek-while-playing is simpler (one call, no state flicker).
- A trigger lead of about 15 ms puts the end of Song A within 11 ms and never late.

## Not tested
- **Native region playlist:** REAPER's API has no region-playlist functions (none in `reaper-low`), so it cannot be driven programmatically. v1 also builds its own playlist. Not an option.
- **Real audio:** the audible gap was not recorded; the numbers are position-based.

## Menus and modal dialogs (measured 2026-10-07, REAPER 7.82, macOS arm64)
Method: the S2 spike ran a Seek hand-over (lead 15 ms) while a script (macOS Accessibility API, Terminal granted Accessibility) opened a menu or dialog in the isolated test REAPER about 1.5 s into the run, held it past the trigger, then closed it.
- **File menu held open across the trigger** (run 1): 237 log lines, ticks stay at about 30 ms. One gap of 112.7 ms when the menu opened. Trigger at pos 10.0107, next tick reads 12.0320.
- **Project Settings dialog (modal, AX reports `modal=1`) held open across the trigger** (runs 2 and 3): 230 ticks each, ticks stay at about 30 ms. One gap when the dialog opened (135.1 ms and 95.1 ms). Run 2 trigger at pos 9.9893, next tick reads 12.0320.
- Conclusion: on macOS the main-thread tick keeps running with a menu or a modal dialog open and the hand-over fires on time. The only effect seen is one tick delayed by 95–135 ms at the moment the menu or dialog opens. A hand-over that falls in that single gap would be late by that much (not measured). Other dialogs and Windows are untested.

## Decision (proposed)
The extension decides in its tick and issues a seek-while-playing with a lead of about 15 ms (a tunable constant, not a user setting). One hand-over code path.

## Open items before Gate 1
1. Record the audio output once (loopback or a render of the click project) to confirm the audible gap against the position-based numbers.
