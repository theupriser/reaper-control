# Domain: Setlists

**Status: draft for the owner's review (WP 0.8), 2026-10-09.** Source: SPEC §14.3, `crates/setlists`, and ADR-008 (song identity).

## Timeline of events
`SetlistCreated` → `EntryAdded` / `EntryMoved` / `EntryRemoved` / `Renamed` → saved with a new revision → (`Deleted`).

## Aggregate `Setlist`
| # | Invariant | Test |
|---|---|---|
| S1 | A setlist needs a name; renaming trims it | `a_setlist_needs_a_name`, `rename_trims` |
| S2 | Add inserts or appends; entry ids are never reused | `add_inserts_and_appends`, `entry_ids_are_not_reused` |
| S3 | Move puts the entry at the index of the result | `move_puts_the_entry_at_the_index_of_the_result` |
| S4 | A refused edit changes nothing | `refused_edits_change_nothing` |
| S5 | A stale edit (old revision) is refused | `a_stale_edit_is_refused` |
| S6 | Dangling entries (unknown song) are reported, not removed | `dangling_entries_are_reported_not_removed` |
| S7 | Restoring checks ids and continues numbering after the highest | `restore_checks_ids_and_continues_after_the_highest` |
| S8 | Saving is optimistic; deleting needs the expected revision | `repository_saves_optimistically`, `repository_deletes_only_the_expected_revision` |
| S9 | Any sequence of edits keeps the setlist valid | `edits_keep_the_setlist_valid` (property) |
| S10 | Moves, adds and removes change only what they say | `moves_and_adds_and_removes_only_change_what_they_say` (property) |
| S11 | A v1 item maps to the song with the same name and region number, else the same name; the rest is reported | `the_region_number_tells_songs_with_the_same_name_apart`, `items_map_to_songs_by_name_and_a_missing_song_is_reported` (`crates/app/tests/legacy_import.rs`) |

## Open for the owner
- Per-entry settings (SPEC mentions "optional"): wanted for v2.0, or later?
