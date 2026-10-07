//! Reads messages from a stream with a read timeout, without ever blocking for long.

use std::io::{self, Read};
use std::net::TcpStream;

use protocol::frame::FrameDecoder;
use protocol::message::{CodecError, decode_message};
use serde::de::DeserializeOwned;

/// Why a connection has to be closed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadError {
    #[error("peer closed the connection")]
    Closed,
    #[error("socket: {0}")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Codec(#[from] CodecError),
}

pub(crate) struct MessageReader {
    stream: TcpStream,
    decoder: FrameDecoder,
}

impl MessageReader {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            decoder: FrameDecoder::new(),
        }
    }

    /// The next message, or `None` when the socket timeout passed with no complete message.
    pub(crate) fn poll<T: DeserializeOwned>(&mut self) -> Result<Option<T>, ReadError> {
        loop {
            if let Some(payload) = self.decoder.next_frame().map_err(CodecError::from)? {
                return Ok(Some(decode_message(&payload)?));
            }
            let mut chunk = [0u8; 4096];
            match self.stream.read(&mut chunk) {
                Ok(0) => return Err(ReadError::Closed),
                Ok(n) => self.decoder.push(chunk.get(..n).unwrap_or_default()),
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e.into()),
            }
        }
    }
}
