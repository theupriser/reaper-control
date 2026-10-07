//! Wire protocol between the REAPER extension and the app (SPEC §2.2).
//! The types here are the single source for the UI's TypeScript types (WP 0.4).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg(test)]
mod generated;

/// Phases the stub knows about; a subset of SPEC §14.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub enum Phase {
    /// Nothing is playing.
    Idle,
    /// Playing a song.
    Playing,
    /// Playback is paused.
    Paused,
}

/// Everything the UI can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
pub enum Command {
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop playback.
    Stop,
}

/// What the UI renders. The UI never infers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
pub struct AppState {
    /// Current phase.
    pub phase: Phase,
}

impl Default for AppState {
    fn default() -> Self {
        Self { phase: Phase::Idle }
    }
}
