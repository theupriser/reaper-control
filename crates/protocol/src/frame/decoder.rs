use super::{FrameError, HEADER_LENGTH, check_length};

/// Collects bytes from a stream and hands out complete frames.
///
/// The caller reads in chunks and drains [`FrameDecoder::next_frame`] after each
/// `push`, so the buffer holds at most one frame plus one chunk.
#[derive(Debug, Default)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
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
            self.buffer.extend_from_slice(bytes);
        }
    }

    /// The next complete payload, `None` when more bytes are needed.
    /// After an error every later call returns the same error.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        if let Some(error) = self.failed {
            return Err(error);
        }
        let Some(header) = self.buffer.first_chunk::<HEADER_LENGTH>() else {
            return Ok(None);
        };
        let length = usize::try_from(u32::from_be_bytes(*header)).unwrap_or(usize::MAX);
        if let Err(error) = check_length(length) {
            self.failed = Some(error);
            self.buffer = Vec::new();
            return Err(error);
        }
        let end = HEADER_LENGTH + length;
        let Some(payload) = self.buffer.get(HEADER_LENGTH..end) else {
            return Ok(None);
        };
        let payload = payload.to_vec();
        self.buffer.drain(..end);
        Ok(Some(payload))
    }
}
