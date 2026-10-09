# ADR-001: The app is a Tauri 2 shell with a Rust core and a Svelte 5 UI

Status: **Accepted**. Date: 2026-10-09. Related: SPEC §5, ADR-010, WP 1.8.

## Question
v1 is Electron. v2 is used on stage, ships to macOS (arm64) and Windows (x64) only (D6, D8), and the core (link client, MIDI, command queue) is Rust already because the performance rules are shared with the extension. Which shell hosts the UI?

## Decision
Tauri 2: the Rust core runs in the app process, the Svelte 5 UI runs in the system webview (WKWebView on macOS, WebView2 on Windows). The UI renders state and sends intents; it never infers state (AGENTS.md).

## Why
- One language for the core: the app reuses the `performance`, `protocol` and `link` crates directly, no Node bridge.
- Small installer (about 10 MB) and no bundled browser; memory-safe core.
- Evidence: ADR-010 measured 59.8 fps with a 30 Hz state push on macOS. WebView2 renders and runs the Svelte UI on Windows (WP 1.8, 2026-10-09).

## Cost
- Two webview engines to test, and the UI may only use features both have.
- Gotcha: a plain `cargo build` points the window at the dev server (`devUrl`); a release build must use the `custom-protocol` feature to embed `ui/dist` (CI package job does).

## Undo if
The webview shows a stage-blocking defect on a supported platform that cannot be worked around. Then the UI moves to another shell; the Rust core and the protocol stay.
