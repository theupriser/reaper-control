use crate::frame::FrameError;

/// A message could not be turned into or out of a frame.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// The frame itself is invalid.
    #[error(transparent)]
    Frame(#[from] FrameError),
    /// The payload is not a valid message.
    #[error("invalid message: {0}")]
    Json(#[from] serde_json::Error),
}
