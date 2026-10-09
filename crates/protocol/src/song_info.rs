use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A song (a region) with the meaning of its markers already worked out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct SongInfo {
    /// Identity that survives renames and moves (ADR-008).
    pub id: String,
    /// The region number REAPER shows.
    pub number: u32,
    /// Display name.
    pub name: String,
    /// Start on the project timeline, in seconds.
    pub start: f64,
    /// End on the project timeline, in seconds.
    pub end: f64,
    /// Region colour as `#rrggbb`, if it has one.
    pub colour: Option<String>,
    /// A `!1008` cue inside the song.
    pub hard_stop: bool,
    /// A `!length:N` cue inside the song, in seconds.
    pub length: Option<f64>,
    /// A `!bpm:N` cue inside the song.
    pub bpm: Option<f64>,
}
