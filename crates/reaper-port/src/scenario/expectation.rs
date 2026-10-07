use serde::Deserialize;

/// What must be true at this point of a scenario. Fields left out are not checked.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Expectation {
    /// The performance phase, for example "Playing".
    pub phase: Option<String>,
    /// The id of the current song.
    pub song: Option<String>,
    /// The play position in seconds.
    pub position: Option<f64>,
    /// The transport: "Stopped", "Playing" or "Paused".
    pub transport: Option<String>,
    /// Whether REAPER's count-in is on.
    pub count_in: Option<bool>,
    /// The events since the previous expectation, by name, in order.
    pub events: Option<Vec<String>>,
}
