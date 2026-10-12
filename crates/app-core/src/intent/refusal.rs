//! Why an intent was refused before it became a command.

/// What the current state says about an intent, found without asking REAPER.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IntentRefusal {
    /// No song is selected to move from or restart.
    #[error("there is no song to play")]
    NothingToPlay,
    /// The active setlist has no song after the current one.
    #[error("there is no next song")]
    NoNextSong,
}
