# Domain: Catalogue

**Status: draft for the owner's review (WP 0.8), 2026-10-09.** Source: SPEC §4 and §14.3, `crates/catalogue`, `testing/vectors/`.

## Timeline of events
The project changes in REAPER → the extension rebuilds the catalog (new `revision`) → the app shows it. The catalogue never mutates anything.

## Read model `Song`, `Cue`, `Directives`
| # | Invariant | Test |
|---|---|---|
| C1 | A song needs a window (end after start) | `song_needs_a_window` |
| C2 | Only cues inside the song window count, edges included | `only_cues_inside_the_window_count_edges_included` |
| C3 | A song's length falls back to its region when no `!length` is given | `length_falls_back_to_the_region` |
| C4 | A cue keeps its name; command-only markers are hidden | `cue_keeps_its_name_and_hides_command_only_names` |
| C5 | Directives parse exactly as the shared test vectors say | `shared_vectors_parse_as_written` |

## Open for the owner
- The catalog does not carry the region colour yet (`SongInfo.colour` is always empty) or the song's tempo number beyond `!bpm`. Needed for the Player screen?
