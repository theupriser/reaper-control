//! Messages on the link (SPEC §2.2) and the handshake rule. Every message is one JSON
//! document inside one frame.

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::frame::{self, FrameError};
use crate::{AppState, Command};

/// Bumped on every incompatible change; checked in the handshake.
pub const PROTOCOL_VERSION: u32 = 1;

/// App to extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// First message on a connection.
    Hello {
        /// Protocol version the app speaks.
        protocol: u32,
        /// Token from the endpoint file.
        token: String,
    },
    /// Ask the extension to do something; answered by an [`ServerMessage::Ack`].
    Command {
        /// Chosen by the app, echoed in the Ack.
        id: u64,
        /// What to do.
        command: Command,
    },
    /// Liveness check, answered by [`ServerMessage::Pong`].
    Ping,
}

/// Extension to app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Answer to an accepted Hello.
    Welcome {
        /// Protocol version the extension speaks.
        protocol: u32,
        /// Version of the extension build.
        extension_version: String,
    },
    /// Pushed whenever the state changes.
    State {
        /// The new state.
        state: AppState,
    },
    /// Answer to a Command.
    Ack {
        /// The id from the Command.
        id: u64,
        /// What happened to it.
        outcome: Outcome,
    },
    /// Answer to a Ping.
    Pong,
}

/// How a Command ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum Outcome {
    /// Done.
    Done,
    /// Refused, with a reason for the log.
    Rejected {
        /// Why.
        reason: String,
    },
}

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

/// Serialise `message` into one complete frame, ready to write.
pub fn encode_message<T: Serialize>(message: &T) -> Result<Vec<u8>, CodecError> {
    Ok(frame::encode(&serde_json::to_vec(message)?)?)
}

/// Parse one frame payload (as returned by the decoder) into a message.
pub fn decode_message<T: DeserializeOwned>(payload: &[u8]) -> Result<T, CodecError> {
    Ok(serde_json::from_slice(payload)?)
}

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

/// Accept the first message of a connection only if it is a Hello with the right version and token.
pub fn check_hello(expected_token: &str, first: &ClientMessage) -> Result<(), HandshakeError> {
    let ClientMessage::Hello { protocol, token } = first else {
        return Err(HandshakeError::NotHello);
    };
    if !same_token(expected_token, token) {
        return Err(HandshakeError::BadToken);
    }
    if *protocol != PROTOCOL_VERSION {
        return Err(HandshakeError::ProtocolMismatch { got: *protocol });
    }
    Ok(())
}

/// Compares without stopping at the first difference.
fn same_token(expected: &str, given: &str) -> bool {
    expected.len() == given.len()
        && expected
            .bytes()
            .zip(given.bytes())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
}

#[cfg(test)]
mod tests;
