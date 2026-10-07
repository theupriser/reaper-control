# ADR-010: The webview keeps up with a 30 Hz state push

Status: **Proposed**. Spike S5 (WP 1.5), code in `spikes/s5-webview`. macOS arm64 only; Windows (WebView2) is not measured.
Date: 2026-10-07. Related: ADR-003, SPEC §14, PLAN WP 1.5.

## Question (S5)
Can the Tauri webview show a Performer-style screen fed with state pushed at 30 Hz without dropped frames, and how long does a button press (IPC round trip) take?

## What was run
- Standalone Tauri 2 crate (`spikes/s5-webview`, plain JS, no Svelte), debug build, macOS arm64, WKWebView, retina display (dpr 2), window 1280x800 in front.
- A Rust thread emits a `state` event every 1/30 s (scheduled against a fixed start time) with sequence number, send time, position, song. The page updates the clock text, the progress bar (`transform: scaleX`) and the highlight in a song list on every event, and measures itself for 30 s: `requestAnimationFrame` intervals, gaps between events, send-to-receive latency, time spent in the event handler, and the round trip of an `invoke` call every 250 ms. The result is printed by the Rust side (`S5 RESULT`).
- **light:** 12 list rows, only the highlight changes. **heavy:** 60 rows, every row's text and class rewritten on every event (a deliberately wasteful worst case).

## Results (30 s each, 902 events each)
| Measure | light | heavy (run 1) | heavy (run 2) |
|---|---|---|---|
| Frames per second | 59.84 | 59.81 | 59.84 |
| Frame interval p50 / p99 / max ms | 17 / 18 / 93 | 17 / 18 / 97 | 17 / 20 / 92 |
| Frames over 25 ms | 1 | 2 | 2 |
| Events missed (sequence gaps) | 0 | 0 | 0 |
| Event gap p50 / p99 / max ms | 33 / 38 / 40 | 33 / 38 / 44 | 33 / 38 / 48 |
| Send to receive p99 ms | 0.8 | 0.9 | 0.8 |
| Handler time p99 / max ms | 1 / 1 | 1 / 1 | 1 / 1 |
| `invoke` round trip p50 / p99 / max ms | 2 / 3 / 3 | 2 / 4 / 4 | 2 / 5 / 5 |

- The slow frames are at start-up: run 2 logged them at 0.09 s (92 ms) and 0.17 s (27 ms), none later. Steady state shows no frame over 25 ms in 30 s.
- Send-to-receive uses `Date.now()` (1 ms resolution) against the Rust clock; the p50 is about 0 and only the p99 is meaningful as "under 1 ms".
- Timer resolution of the measurements is 1 ms for latency and the frame interval is quantised to the 60 Hz display (16.7 ms).

## Not tested
- **Windows / WebView2.** Different engine, must be measured on a real Windows machine (Gate 1).
- Svelte 5 on top (the real UI); the reactivity cost is expected to be small next to the 1 ms handler time, but it is not measured.
- Window hidden, minimised or behind another window (the page is then throttled; state still arrives but frames are not drawn).
- Touch input and a touch screen; a long session (soak); a larger state with real setlist data.

## Decision (proposed)
Push full state at 30 Hz from Rust to the webview as events; keep UI updates to text and `transform` changes. The webview is not a risk for the Performer screen on macOS. Button presses use `invoke` (about 2 ms round trip). Windows must be measured before Gate 1 closes.
