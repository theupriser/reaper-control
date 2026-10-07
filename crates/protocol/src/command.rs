use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Everything the UI can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
pub enum Command {
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop playback.
    Stop,
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
}
