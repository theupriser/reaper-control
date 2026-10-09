# Domain: Performance

**Status: draft for the owner's review (WP 0.8), 2026-10-09.** Source: SPEC §14.3 and `crates/performance`. Event storming (a timeline built with the owner) is not done; the table below was derived from the code and its tests.

## Timeline of events (what happens, in order)
`PerformanceStarted` → `HandOverStarted` → `HandOverCompleted` → … → (`HardStopReached`) → `PerformanceFinished`. Side events: `FlagChanged`, `SeekPerformed`, `CommandRejected`.

## Aggregate `Performance`
| # | Invariant | Test that holds it |
|---|---|---|
| P1 | A performance of an empty setlist cannot play | `empty_setlist_cannot_play` |
| P2 | Play from idle seeks to the start of the song and plays | `play_from_idle_seeks_to_the_start_and_plays` |
| P3 | Pausing and playing again resumes where it was | `pause_then_play_resumes_where_it_was` |
| P4 | Nothing happens before the hand-over lead time | `nothing_happens_before_the_lead` |
| P5 | One song end fires one hand-over | `one_song_end_fires_one_hand_over` |
| P6 | A hand-over right after another is ignored | `a_hand_over_right_after_another_is_ignored` |
| P7 | A contiguous hand-over does not seek; one over a gap seeks to the next start | `contiguous_hand_over_does_not_seek`, `hand_over_over_a_gap_seeks_to_the_next_start` |
| P8 | A hand-over completes when the position is inside the next song | `hand_over_completes_when_the_position_is_in_the_next_song` |
| P9 | A hard stop always halts at the song end, and honours a shorter `!length` | `hard_stop_halts_at_the_song_end`, `hard_stop_honours_a_shorter_length` |
| P10 | Play after a hard stop goes into the next song; on the last song it finishes | `play_after_a_hard_stop_goes_into_the_next_song`, `hard_stop_on_the_last_song_finishes` |
| P11 | Finishing is terminal until play restarts | `finished_stays_finished_until_play_restarts`, `finished_stays_finished_until_a_command` (property) |
| P12 | Next and previous stop at the ends of the setlist | `next_and_previous_stop_at_the_ends_of_the_setlist` |
| P13 | A seek stays inside the current song | `seek_stays_inside_the_current_song` |
| P14 | A cue jump counts in only when the flag is on; manual navigation never counts in | `cue_jump_counts_in_when_enabled`, `cue_jump_without_the_flag_is_a_plain_seek`, `manual_navigation_never_counts_in` |
| P15 | Pausing during a count-in cancels it | `pausing_a_count_in_cancels_it` |
| P16 | A flag event is sent only when the flag changes | `a_flag_event_is_sent_only_when_it_changes` |
| P17 | A song window ends after it starts | `a_window_needs_an_end_after_the_start` |
| P18 | After every step: one current song at most, position inside its bounds except during a hand-over | `invariants_hold_after_every_step` (property) |
| P19 | Playing through visits every song once | `playing_through_visits_every_song_once` (property) |

All tests are in `crates/performance/src/tests.rs` and `performance_properties.rs`.

## Open for the owner
- Hand-over lead time and the count-in length: are the defaults the ones you play with?
- Any rule you rely on on stage that is not in the table.
