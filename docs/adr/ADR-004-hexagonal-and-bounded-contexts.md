# ADR-004: Hexagonal architecture with one crate per bounded context

Status: **Accepted**. Date: 2026-10-09. Related: SPEC §14, ADR-011, ADR-012, `docs/ARCHITECTURE.md`.

## Question
The extension and the app share rules but run in different processes, and everything REAPER-specific must be replaceable by a test double. How is the code cut?

## Decision
- Domain code is pure (no I/O, no REAPER calls, no real clock) and depends only on `shared-kernel`.
- Everything REAPER-specific sits behind the `ReaperPort` trait (`ReaperRsAdapter` real, `FakeReaper` for tests).
- One crate per bounded context under `crates/`: `performance` (core domain), `setlists`, `catalogue`, `link` (Reaper Link), `protocol` (the published language). Control, installation, settings and diagnostics are modules in `app` until they grow a second user.
- Contexts talk through `protocol` and events, never through each other's domain types.
- `reaper-extension` must not depend on Tauri or tokio; `app` is the composition root only.
- `architecture-tests` enforces the dependency rules in CI (SPEC §14.5) and the glossary (banned words).

## Why
The same `performance` crate drives the extension in production and the app's simulator and tests, so there is one implementation of the rules. The fake port lets the whole show run without REAPER.

## Cost
Many small crates and strict rules; rule changes need an ADR (ADR-011, ADR-012).

## Undo if
A rule blocks real work twice: amend it in an ADR rather than bypass it.
