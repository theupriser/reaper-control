use super::MAX_FRAME_LENGTH;

/// The stream is broken. After one of these the connection must be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FrameError {
    /// The length field is above [`MAX_FRAME_LENGTH`].
    #[error("frame of {length} bytes exceeds the limit of {MAX_FRAME_LENGTH} bytes")]
    TooLarge {
        /// Length the peer announced, or the payload length when encoding.
        length: usize,
    },
    /// A frame without a payload; no message is empty.
    #[error("empty frame")]
    Empty,
}
