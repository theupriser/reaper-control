//! What a MIDI note can be mapped to.

use protocol::Command;
use serde::{Deserialize, Serialize};

/// An action a note can trigger; the mapping from notes to actions is data in the config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiAction {
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

impl MidiAction {
    /// The command for this action; `playing` says whether the performance runs now.
    #[must_use]
    pub fn command(self, playing: bool) -> Command {
        match self {
            Self::RestartSong => Command::RestartSong,
            Self::ToggleAutoResume => Command::ToggleAutoResume,
            Self::ToggleCountInOnMarker => Command::ToggleCountInOnMarker,
            Self::ToggleRecordArm => Command::ToggleRecordArm,
            Self::Previous => Command::Previous,
            Self::Pause => Command::Pause,
            Self::TogglePlay if playing => Command::Pause,
            Self::TogglePlay => Command::Play,
            Self::Next => Command::Next,
        }
    }

    /// The action behind a name v1 stored in its config; `None` for a name v2 does not know.
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
