use protocol::message::{ServerMessage, encode_message};

/// The complete frame that tells a client REAPER is quitting. Built ahead of time, so the exit
/// hook only has to write bytes.
#[must_use]
pub fn closing_frame() -> Vec<u8> {
    encode_message(&ServerMessage::Closing).unwrap_or_default()
}
