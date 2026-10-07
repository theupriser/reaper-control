//! Wire protocol between the REAPER extension and the app (SPEC §2.2).
//! The types here are the single source for the UI's TypeScript types (WP 0.4).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod frame;
pub mod message;

#[cfg(test)]
mod generated;

/// Phases the stub knows about; a subset of SPEC §14.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Phase {
    /// Nothing is playing.
    Idle,
    /// Playing a song.
    Playing,
    /// Playback is paused.
    Paused,
}

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

/// What the UI renders. The UI never infers it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
pub struct AppState {
    /// Current phase.
    pub phase: Phase,
    /// Seconds from the start of the current song.
    pub position: f64,
    /// "Auto-resume playback".
    pub auto_resume: bool,
    /// "Count-in when pressing marker".
    pub count_in_on_marker: bool,
    /// Recording is armed.
    pub record_armed: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            position: 0.0,
            auto_resume: true,
            count_in_on_marker: false,
            record_armed: false,
        }
    }
}
