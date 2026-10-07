use serde::Deserialize;

use crate::{ScenarioError, ScenarioSong, Step};

/// A replayable script: a small project, settings, and steps with checks.
/// The same file runs in the extension tests, the simulator and the dry-run.
#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    /// What the scenario shows.
    pub name: String,
    /// The songs, in setlist order.
    pub songs: Vec<ScenarioSong>,
    /// Start with autoplay on.
    #[serde(default)]
    pub autoplay: bool,
    /// Start with count-in on.
    #[serde(default)]
    pub count_in: bool,
    /// What happens, in order.
    pub steps: Vec<Step>,
}

impl Scenario {
    /// Reads a scenario from its JSON text.
    pub fn from_json(text: &str) -> Result<Self, ScenarioError> {
        Ok(serde_json::from_str(text)?)
    }
}
