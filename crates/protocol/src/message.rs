//! Messages on the link (SPEC §2.2) and the handshake rule. Every message is one JSON
//! document inside one frame.

mod client_message;
mod codec;
mod codec_error;
mod handshake;
mod handshake_error;
mod outcome;
mod server_message;

pub use client_message::ClientMessage;
pub use codec::{decode_message, encode_message};
pub use codec_error::CodecError;
pub use handshake::check_hello;
pub use handshake_error::HandshakeError;
pub use outcome::Outcome;
pub use server_message::ServerMessage;

/// Bumped on every incompatible change; checked in the handshake.
pub const PROTOCOL_VERSION: u32 = 2;

#[cfg(test)]
mod tests;
