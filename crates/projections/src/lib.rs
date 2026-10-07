//! Projections: read models the UI renders, built from the contexts' state.
//! SPEC §14.4 (CQRS-lite). Pure: no I/O, no REAPER calls, no real clock.
//! Also builds the `PlannedSong` list from a setlist and the catalogue.

mod applied;
mod entry_view;
mod freshness;
mod live_feed;
mod performance_view;
mod plan;
mod player_view;
mod setlist_view;

pub use applied::Applied;
pub use entry_view::EntryView;
pub use freshness::Freshness;
pub use live_feed::LiveFeed;
pub use performance_view::PerformanceView;
pub use plan::Plan;
pub use player_view::PlayerView;
pub use setlist_view::SetlistView;

#[cfg(test)]
mod tests;
