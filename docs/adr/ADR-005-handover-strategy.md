# ADR-005: Hand-over is a seek issued from the extension's main-thread tick

Status: **Proposed**. Spike S2 is positive for the timing: position readout, menus and dialogs, and a recording of REAPER's output all agree. One caveat stays (sound card path, below).
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
- **Sound card output:** the recording below is REAPER's render stream, not what came out of the speakers. Driver and device latency are not covered (they do not change the gap between the two songs, only when both are heard).

## Menus and modal dialogs (measured 2026-10-07, REAPER 7.82, macOS arm64)
Method: the S2 spike ran a Seek hand-over (lead 15 ms) while a script (macOS Accessibility API, Terminal granted Accessibility) opened a menu or dialog in the isolated test REAPER about 1.5 s into the run, held it past the trigger, then closed it.
- **File menu held open across the trigger** (run 1): 237 log lines, ticks stay at about 30 ms. One gap of 112.7 ms when the menu opened. Trigger at pos 10.0107, next tick reads 12.0320.
- **Project Settings dialog (modal, AX reports `modal=1`) held open across the trigger** (runs 2 and 3): 230 ticks each, ticks stay at about 30 ms. One gap when the dialog opened (135.1 ms and 95.1 ms). Run 2 trigger at pos 9.9893, next tick reads 12.0320.
- Conclusion: on macOS the main-thread tick keeps running with a menu or a modal dialog open and the hand-over fires on time. The only effect seen is one tick delayed by 95–135 ms at the moment the menu or dialog opens. A hand-over that falls in that single gap would be late by that much (not measured). Other dialogs and Windows are untested.

## Recorded output (measured 2026-10-07, REAPER 7.82, macOS arm64)
Method: no loopback driver was needed. In a copy of the click project a second track receives the click track and is armed in "record output" mode; the spike starts recording (action 1013 from the extension) instead of play, runs the same Seek hand-over (Song A end 10 s to Song B start 12 s, recording started at 6 s) and stops with "Stop (save all recorded media)" (action 40667; a plain stop opens a modal save/delete dialog that blocks the main thread). The click project's accent was changed from 4 kHz to 3 kHz because 4 kHz at 8 kHz sampling is exactly Nyquist and renders silence. Five runs, 48 kHz 24-bit, onsets found by threshold.
- Five runs: lead 15 ms three times (trigger positions 10.0000, 9.9893, 9.9893) and lead 0 ms twice (10.0107, 10.0213). In all five the clicks stay on the 0.5 s grid with intervals of 0.4999 to 0.5001 s (about 5 samples) across the hand-over, and the click at the cut is complete: burst energy 7.13 at 2.0, 4.0 and 6.0 s, so nothing is clipped, doubled or missing. (The first run was a pilot whose file was analysed and then deleted; the other four were analysed together.)
- Even at lead 0, where REAPER's position readout was already 10.7 to 21 ms past the end of Song A at the trigger, the recording shows no trace of the end of Song A. The position-based "late" figures are therefore conservative for the render stream: the seek takes effect on the next audio block, ahead of what the position readout shows. The clicks at 10.0 s and 12.0 s are identical, so the recording cannot tell which of the two is the one at the cut; it shows only that exactly one complete click sits there.
- Conclusion: no audible gap, overlap or shift at the hand-over in the render stream, at lead 15 ms and at lead 0 ms. The 15 ms lead stays as the safety margin.

## Decision (proposed)
The extension decides in its tick and issues a seek-while-playing with a lead of about 15 ms (a tunable constant, not a user setting). One hand-over code path.

## Open items before Gate 1
1. None for timing on macOS. Windows (S1/S2 on a real machine) and the sound-card path are not covered.
