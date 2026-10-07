//! Length-prefixed framing (SPEC §2.2): a 4-byte big-endian length, then that many payload bytes.
//! The decoder never allocates for a length it will not accept and never panics on any input.

/// Largest payload a frame may carry (1 MiB). Anything bigger is a protocol error.
pub const MAX_FRAME_LEN: usize = 1 << 20;

const HEADER_LEN: usize = 4;

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

/// Prefix `payload` with its length.
pub fn encode(payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    check_len(payload.len())?;
    let len =
        u32::try_from(payload.len()).map_err(|_| FrameError::TooLarge { len: payload.len() })?;
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

fn check_len(len: usize) -> Result<(), FrameError> {
    match len {
        0 => Err(FrameError::Empty),
        l if l > MAX_FRAME_LEN => Err(FrameError::TooLarge { len: l }),
        _ => Ok(()),
    }
}

/// Collects bytes from a stream and hands out complete frames.
///
/// The caller reads in chunks and drains [`FrameDecoder::next_frame`] after each
/// `push`, so the buffer holds at most one frame plus one chunk.
#[derive(Debug, Default)]
pub struct FrameDecoder {
    buf: Vec<u8>,
    failed: Option<FrameError>,
}

impl FrameDecoder {
    /// An empty decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bytes read from the stream. Ignored once the decoder has failed.
    pub fn push(&mut self, bytes: &[u8]) {
        if self.failed.is_none() {
            self.buf.extend_from_slice(bytes);
        }
    }

    /// The next complete payload, `None` when more bytes are needed.
    /// After an error every later call returns the same error.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        if let Some(err) = self.failed {
            return Err(err);
        }
        let Some(header) = self.buf.first_chunk::<HEADER_LEN>() else {
            return Ok(None);
        };
        let len = usize::try_from(u32::from_be_bytes(*header)).unwrap_or(usize::MAX);
        if let Err(err) = check_len(len) {
            self.failed = Some(err);
            self.buf = Vec::new();
            return Err(err);
        }
        let end = HEADER_LEN + len;
        let Some(payload) = self.buf.get(HEADER_LEN..end) else {
            return Ok(None);
        };
        let payload = payload.to_vec();
        self.buf.drain(..end);
        Ok(Some(payload))
    }
}

#[cfg(test)]
mod tests;
