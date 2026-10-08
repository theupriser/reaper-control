# ADR-012: The timer loop gets its own crate

Status: **Proposed**. Date: 2026-10-08. Related: ADR-011, SPEC §14.5, WP 4.11, WP 4.12.

## Question
`TimerLoop` (read the port, step the performance, carry out the effects, build the catalog and the wire events) lived in `reaper-extension`. The app's simulator (WP 4.11) and the in-process fake extension server (WP 4.12) must run that same code over `FakeReaper`. Nothing may depend on `reaper-extension` (it is loaded by REAPER, never linked), so the app could not reach it.

## Decision
Move `TimerLoop` and its helpers unchanged into a new crate `timer-loop`. It may use `shared-kernel`, `performance`, `catalogue`, `setlists`, `protocol` and `reaper-port`. `reaper-extension` uses it; `app` may use it, plus `reaper-port` (for `FakeReaper`) and `performance` (for `TempoMap`).

## Cost
- The app now sees `reaper-port` and `performance`. It uses them only to build a simulated project; the real app path still goes through `link`.
- `TimerLoop::port_mut` is no longer limited to tests and the probe (the simulator moves its clock through it).

## Undo if
The simulator and the fake server move to a test-only crate: then `app` goes back to `shared-kernel`, `protocol`, `link`.
