use super::MAX_FRAME_LEN;

/// The stream is broken. After one of these the connection must be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FrameError {
    /// The length field is above [`MAX_FRAME_LEN`].
    #[error("frame of {len} bytes exceeds the limit of {MAX_FRAME_LEN} bytes")]
    TooLarge {
        /// Length the peer announced, or the payload length when encoding.
        len: usize,
    },
    /// A frame without a payload; no message is empty.
    #[error("empty frame")]
    Empty,
}
