use serde::Deserialize;

use crate::Expectation;

/// One line of a scenario: a command, time passing, or a check.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "do", rename_all = "snake_case")]
pub enum Step {
    /// Start or resume.
    Play,
    /// Pause.
    Pause,
    /// Next song.
    Next,
    /// Previous song.
    Previous,
    /// Back to the start of the current song.
    Restart,
    /// Jump inside the current song.
    Seek {
        /// Target position in seconds.
        position: f64,
    },
    /// Jump to a cue, counting in when enabled.
    SeekCue {
        /// Cue position in seconds.
        position: f64,
    },
    /// Switch a setting: "autoplay" or "count_in".
    SetFlag {
        /// Name of the setting.
        flag: String,
        /// New value.
        enabled: bool,
    },
    /// Let time pass.
    Advance {
        /// How many seconds.
        seconds: f64,
    },
    /// Check the state.
    Expect(Expectation),
}
