use super::PROTOCOL_VERSION;

/// Why a connection did not get past the handshake. The extension closes it without a reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HandshakeError {
    /// The first message was not a Hello.
    #[error("first message is not Hello")]
    NotHello,
    /// The app speaks another protocol version.
    #[error("protocol {got} not supported, expected {PROTOCOL_VERSION}")]
    ProtocolMismatch {
        /// Version the app announced.
        got: u32,
    },
    /// The token does not match the endpoint file.
    #[error("bad token")]
    BadToken,
}
