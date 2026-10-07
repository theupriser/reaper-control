use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CueInfo, SetlistInfo, SongInfo};

/// What the project contains. Sent when a revision changes or when asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Catalog {
    /// Raised whenever songs or cues change.
    #[ts(type = "number")]
    pub rev: u64,
    /// Raised whenever a setlist changes.
    #[ts(type = "number")]
    pub setlist_rev: u64,
    /// Songs in timeline order.
    pub songs: Vec<SongInfo>,
    /// Cues in timeline order.
    pub cues: Vec<CueInfo>,
    /// The setlists of the project.
    pub setlists: Vec<SetlistInfo>,
}
