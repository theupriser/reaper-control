use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Phase, Transport};

/// The state that changes all the time. Pushed on change (at most ~30 Hz while playing)
/// and at least once a second as the heartbeat; a gap of more than a second means stale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Live {
    /// Counts up per sender run; the receiver drops anything not newer than the last one.
    #[ts(type = "number")]
    pub sequence: u64,
    /// Sender clock in seconds, for latency figures only.
    pub timestamp: f64,
    /// What REAPER's transport does.
    pub transport: Transport,
    /// Seconds on the project timeline.
    pub position: f64,
    /// Where the performance is.
    pub phase: Phase,
    /// The active setlist, if one is selected.
    pub setlist_id: Option<String>,
    /// Index into `Catalog.songs` of the song the performance is on.
    pub current_song: Option<u32>,
    /// Index into `Catalog.songs` of the song that follows.
    pub next_song: Option<u32>,
    /// "Auto-resume playback".
    pub autoplay: bool,
    /// "Count-in when pressing marker".
    pub count_in: bool,
    /// Recording is armed.
    pub record_armed: bool,
    /// Revision of the catalog this state belongs to.
    #[ts(type = "number")]
    pub catalog_revision: u64,
    /// Revision of the setlists this state belongs to.
    #[ts(type = "number")]
    pub setlist_revision: u64,
}

impl Default for Live {
    fn default() -> Self {
        Self {
            sequence: 0,
            timestamp: 0.0,
            transport: Transport::Stopped,
            position: 0.0,
            phase: Phase::Idle,
            setlist_id: None,
            current_song: None,
            next_song: None,
            autoplay: true,
            count_in: false,
            record_armed: false,
            catalog_revision: 0,
            setlist_revision: 0,
        }
    }
}
