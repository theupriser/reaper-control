use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The only phase vocabulary (SPEC §14.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Phase {
    /// Nothing is playing.
    Idle,
    /// Playing a song.
    Playing,
    /// Playback is paused.
    Paused,
    /// The count-in before a cue jump is running.
    CountingIn,
    /// Stopped at a hard-stop song, waiting for Play.
    HardStopped,
    /// Moving from one song into the next.
    HandingOver,
    /// The last song is done.
    Finished,
}
