//! `ReaperPort`, the deterministic `FakeReaper` and the scenario runner (SPEC §7).
//! Everything REAPER-specific hides behind the trait; scenarios replay the same
//! way in the extension tests, the app's simulator and the dry-run.

mod fake_reaper;
mod marker;
mod reaper_port;
mod region;
mod scenario;
mod transport;

pub use fake_reaper::FakeReaper;
pub use marker::Marker;
pub use reaper_port::ReaperPort;
pub use region::Region;
pub use scenario::{
    Expectation, Scenario, ScenarioError, ScenarioRunner, ScenarioSong, Step, Trace,
};
pub use transport::Transport;

#[cfg(test)]
mod properties;
#[cfg(test)]
mod tests;
