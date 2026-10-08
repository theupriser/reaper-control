use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CueInfo, SetlistInfo, SongInfo};

/// What the project contains. Sent when a revision changes or when asked for.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
pub struct Catalog {
    /// Raised whenever songs or cues change.
    #[ts(type = "number")]
    pub revision: u64,
    /// Raised whenever a setlist changes.
    #[ts(type = "number")]
    pub setlist_revision: u64,
    /// Songs in playing order: the played setlist's, or timeline order without one. `Live::current_song` indexes this list.
    pub songs: Vec<SongInfo>,
    /// Every song of the project in timeline order, whatever the played setlist is.
    pub project_songs: Vec<SongInfo>,
    /// Cues in timeline order.
    pub cues: Vec<CueInfo>,
    /// The setlists of the project.
    pub setlists: Vec<SetlistInfo>,
    /// The id of the setlist being played, if any.
    pub active_setlist: Option<String>,
}
