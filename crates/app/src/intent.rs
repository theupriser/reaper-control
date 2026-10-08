//! What a person wants, whichever device it came from.

use serde::{Deserialize, Serialize};

/// What a button, key or note asks for. Every controller produces these; one translator turns
/// them into commands. The mapping from notes to intents is data in the config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Back to the start of the current song.
    RestartSong,
    /// Switch "Auto-resume playback" on or off.
    ToggleAutoResume,
    /// Switch "Count-in when pressing marker" on or off.
    ToggleCountInOnMarker,
    /// Arm or disarm recording.
    ToggleRecordArm,
    /// Go to the previous song.
    Previous,
    /// Pause playback.
    Pause,
    /// Pause when the performance runs, otherwise play.
    TogglePlay,
    /// Go to the next song.
    Next,
}

impl Intent {
    /// The intent behind a name v1 stored in its config; `None` for a name v2 does not know.
    #[must_use]
    pub fn from_legacy_name(name: &str) -> Option<Self> {
        match name {
            "seekToCurrentRegionStart" => Some(Self::RestartSong),
            "toggleAutoplay" => Some(Self::ToggleAutoResume),
            "toggleCountIn" => Some(Self::ToggleCountInOnMarker),
            "toggleRecordingArmed" => Some(Self::ToggleRecordArm),
            "previousRegion" => Some(Self::Previous),
            "pause" => Some(Self::Pause),
            "togglePlay" => Some(Self::TogglePlay),
            "nextRegion" => Some(Self::Next),
            _ => None,
        }
    }
}
