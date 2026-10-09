# ADR-007: Context map

**Status: Proposed (draft for the owner's review, WP 0.8).** Written 2026-10-09 from SPEC §14 and the crates as they are now.

## Contexts and where they live
| Context | Kind | Crate or module | Owns |
|---|---|---|---|
| Performance | core | `performance` | the performance rules; one implementation, used by the extension and the app |
| Setlists | supporting | `setlists` | `Setlist`, editing rules, `SetlistRepository` |
| Catalogue | supporting | `catalogue` | `Song`, `Cue`, directive grammar |
| Reaper Link | supporting | `protocol` (wire), `link` (client and server), `reaper-extension` (REAPER side) | messages, framing, health |
| Installation | generic | `app` (`ExtensionInstaller` and friends) | install, repair, uninstall, verify |
| Settings and Diagnostics | generic | `app` | config file, logs, support bundle |
| Control | supporting | `app` (`Intent`, `IntentDispatcher`, `MidiRouter`) | turning input into commands |

Not yet separate crates (SPEC §14.5 foresaw `control`, `installation`, `settings`): they live in `app` because they have no second consumer. Moving one out is a mechanical change when one appears.

## Relationships
- **Shared kernel:** `shared-kernel` (`Seconds`, `Bpm`, `SongId`). Every domain crate may use it; nothing else is shared.
- **Published language:** `protocol` is the contract between the app and the extension. Both sides use the same crate.
- **Customer/supplier:** the app is the customer of the extension. The extension runs the show; the app asks and shows.
- **Anti-corruption layer:** `app::MessageTranslator` and `app::apply_event` turn wire messages into app events and views; the UI never sees wire details beyond the generated types.
- **Conformist:** the extension follows REAPER's API through `ReaperPort`; REAPER's words (marker, region) stop at that port, except where the glossary says otherwise (region number).

## Rules (enforced by `crates/architecture-tests`)
1. Domain crates depend only on `shared-kernel`.
2. No context uses another context's domain types; they meet in `protocol`.
3. `reaper-extension` does not depend on Tauri or tokio.
4. Nothing depends on the extension crate.

## Open for the owner
Whether `control`, `installation` and `settings` should become crates now (more files in the workspace, no new consumer), or only when a second consumer appears (the current choice).
