//! `ReaperPort`, the deterministic `FakeReaper` and the scenario runner (SPEC §7).
//! Everything REAPER-specific hides behind the trait; scenarios replay the same
//! way in the extension tests, the app's simulator and the dry-run.

mod expectation;
mod fake_reaper;
mod marker;
mod reaper_port;
mod region;
mod scenario;
mod scenario_error;
mod scenario_runner;
mod scenario_song;
mod step;
mod trace;
mod transport;

pub use expectation::Expectation;
pub use fake_reaper::FakeReaper;
pub use marker::Marker;
pub use reaper_port::ReaperPort;
pub use region::Region;
pub use scenario::Scenario;
pub use scenario_error::ScenarioError;
pub use scenario_runner::ScenarioRunner;
pub use scenario_song::ScenarioSong;
pub use step::Step;
pub use trace::Trace;
pub use transport::Transport;

#[cfg(test)]
mod properties;
#[cfg(test)]
mod tests;
