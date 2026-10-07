use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
