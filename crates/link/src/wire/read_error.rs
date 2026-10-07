use std::io;

use protocol::message::CodecError;

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
