# Domain: Reaper Link

**Status: draft for the owner's review (WP 0.8), 2026-10-09.** Source: SPEC §2.2 and §14.3, `crates/protocol`, `crates/link`, `crates/app` (`LinkConnection`, `HealthMonitor`).

## Timeline of events
`LinkEstablished` → state and catalog pushes → `Degraded` (heartbeat late) → `Lost` → reconnect with replay from the last event id → `Recovered`. Faults: `ExtensionNotLoaded`, `ExtensionOutdated`, `ExtensionFaulted`.

## `LinkSession` and health
| # | Invariant | Test |
|---|---|---|
| L1 | No endpoint file: not running, and commands are refused | `a_missing_endpoint_file_shows_not_running_and_refuses_commands` |
| L2 | The app follows the extension and its commands reach it | `it_follows_the_extension_and_sends_commands_to_it` |
| L3 | Connecting and a refused command are announced | `connecting_and_a_refused_command_are_announced_on_the_event_bus` |
| L4 | The bus sends in order, drops a rapid repeat, and hears the answers | `the_bus_sends_in_order_drops_a_rapid_repeat_and_hears_the_answers` |
| L5 | Health improves and worsens step by step; a repeat is not news | `it_starts_lost_and_gets_better_and_worse_step_by_step`, `a_repeat_is_not_news` |
| L6 | Lost is named only after the grace period | `a_lost_link_is_named_only_after_the_grace_period` |
| L7 | The cause follows whether REAPER runs | `the_cause_follows_whether_reaper_runs` |
| L8 | Another protocol version is dead at once | `another_protocol_is_dead_at_once_and_a_connection_ends_it` |
| L9 | An outdated extension becomes "REAPER not running" when REAPER quits | `an_outdated_extension_is_replaced_by_reaper_not_running_when_reaper_quits` |
| L10 | A fault file counts only while REAPER runs; a faulted but answering extension is dead | `a_running_reaper_with_a_faulted_extension_names_the_fault`, `a_fault_left_behind_is_ignored_while_reaper_is_not_running`, `an_extension_that_reports_a_fault_while_still_answering_is_dead` |

(`crates/app/tests/link_connection.rs`, `crates/app/src/health_monitor/tests.rs`; the wire rules have their own tests in `crates/link/tests/` and `crates/protocol`.)

## Open for the owner
- A real event storm on the failure cases: which failure on stage worries you most?
