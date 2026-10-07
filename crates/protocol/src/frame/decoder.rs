use super::{FrameError, HEADER_LEN, check_len};

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
