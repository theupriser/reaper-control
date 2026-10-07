use shared_kernel::InvalidValue;
use thiserror::Error;

/// Why a region cannot be a song.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidSong {
    /// The region ends at or before its start.
    #[error("song ends at or before its start")]
    EmptyWindow,
    /// The window length is not a usable number.
    #[error("song window is not usable: {0}")]
    Window(#[from] InvalidValue),
}
