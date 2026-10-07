use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Phase;

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
