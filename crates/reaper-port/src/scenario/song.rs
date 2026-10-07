use serde::Deserialize;

/// One song of a scenario: a region that is also in the setlist.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioSong {
    /// Identity, for example "A".
    pub id: String,
    /// Where it starts, in seconds.
    pub start: f64,
    /// Where it ends, in seconds.
    pub end: f64,
    /// Whether playback halts at its end (`!1008`).
    #[serde(default)]
    pub hard_stop: bool,
}
