use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::EntryInfo;

/// Everything the UI can ask for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub enum Command {
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop playback.
    Stop,
    /// Go to the next song.
    Next,
    /// Go to the previous song.
    Previous,
    /// Go back to the start of the current song.
    RestartSong,
    /// Jump to the start of a song, never with a count-in.
    GoToSong {
        /// Position of the song in the played order (the setlist, or the timeline without one), from 0.
        index: u32,
    },
    /// Jump to a position in the current song, in seconds from its start.
    Seek {
        /// Seconds from the start of the song.
        position: f64,
        /// Start with a count-in (a click on a Cue while "Count-in when pressing marker" is on).
        count_in: bool,
    },
    /// Switch "Auto-resume playback" on or off.
    ToggleAutoResume,
    /// Switch "Count-in when pressing marker" on or off.
    ToggleCountInOnMarker,
    /// Arm or disarm recording.
    ToggleRecordArm,
    /// Create or replace a setlist in the project. Refused when the stored revision is not
    /// `expected_revision` (use 0 for a new setlist), so an edit never overwrites a newer one.
    SaveSetlist {
        /// Identity of the setlist.
        id: String,
        /// Display name.
        name: String,
        /// The entries in playing order.
        entries: Vec<EntryInfo>,
        /// The revision of the setlist the edit was made on.
        #[ts(type = "number")]
        expected_revision: u64,
    },
    /// Remove a setlist from the project. Refused when the stored revision is not
    /// `expected_revision`, so a newer edit is never deleted unseen. When it is the played setlist,
    /// the songs play in timeline order again.
    DeleteSetlist {
        /// Identity of the setlist.
        id: String,
        /// The revision of the setlist when it was shown.
        #[ts(type = "number")]
        expected_revision: u64,
    },
    /// Choose the setlist that is played; `None` plays the songs in timeline order. Refused for an
    /// id the project does not have.
    SetActiveSetlist {
        /// Identity of the setlist, if any.
        id: Option<String>,
    },
}
