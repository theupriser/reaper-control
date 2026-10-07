# ADR-011: Three changes to the crate dependency rules

Status: **Proposed**. Date: 2026-10-07. Related: SPEC §14.5, `crates/architecture-tests/tests/dependency_rules.rs`, WP 2.7, WP 2.8, WP 2.9.

## Question
The dependency rules say which workspace crates each crate may use. Changing one needs an ADR. Three were changed in earlier work packages without one. This ADR records them, with the reason and what would make us undo them.

## Decisions
| Change | Why | Cost |
|---|---|---|
| `app` may use `protocol` (WP 2.x, stub link) | The app is the link client's user and must name the wire types (`Command`, `AppState`) to show them. | The app sees wire types. It must still never see the `link` internals, which the rule keeps. |
| `reaper-port` may use `performance` (PR #28, WP 2.7) | The scenario runner steps the real `Performance` over a `FakeReaper` and needs `TempoMap` for the count-in lead-in. PLAN WP 2.7 puts the runner in this crate. | `reaper-port` is no longer a leaf. `performance` still may not use `reaper-port`, so the pure core stays pure. |
| `projections` may use `performance`, `catalogue`, `setlists` (PR #29, WP 2.8) | Building a `PlannedSong` list needs the setlist and the catalogue together. The contexts may not use each other (SPEC §14.1), so one application-layer crate joins them. | `projections` knows three contexts. Nothing below it may use it, and `reaper-extension` is the only crate allowed to. |

## What stays true
- Domain crates (`catalogue`, `setlists`, `performance`) depend only on `shared-kernel`.
- `protocol` depends only on `shared-kernel` and holds plain wire types. It does not use `projections`: mapping a view into a `Live` or `Catalog` message is done by the link side (WP 3.x), so the wire format never follows a view by accident.
- `reaper-extension` must not use Tauri or tokio (the `HEAVY` list).

## Undo if
- A second crate needs to join the contexts: then `projections` is the wrong place and the join belongs in a use-case layer of its own.
- The scenario runner moves to `testing/`: then `reaper-port` goes back to `shared-kernel` only.
