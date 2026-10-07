use serde::{Serialize, de::DeserializeOwned};

use super::CodecError;
use crate::frame;

/// Serialise `message` into one complete frame, ready to write.
pub fn encode_message<T: Serialize>(message: &T) -> Result<Vec<u8>, CodecError> {
    Ok(frame::encode(&serde_json::to_vec(message)?)?)
}

/// Parse one frame payload (as returned by the decoder) into a message.
pub fn decode_message<T: DeserializeOwned>(payload: &[u8]) -> Result<T, CodecError> {
    Ok(serde_json::from_slice(payload)?)
}
