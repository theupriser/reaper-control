//! Parses arbitrary payloads as both message types and checks the handshake on whatever parses.
//! Covers every message type, new ones included, since they are variants of these two enums.
//! Invariants: no panic, and anything that parses survives a trip through encode and decode unchanged.
#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::{Serialize, de::DeserializeOwned};
use protocol::message::{
    ClientMessage, ServerMessage, check_hello, decode_message, encode_message,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = decode_message::<ClientMessage>(data) {
        let _ = check_hello("token", &m);
        assert_eq!(again(&m), Some(m));
    }
    if let Ok(m) = decode_message::<ServerMessage>(data) {
        assert_eq!(again(&m), Some(m));
    }
});

/// Encode, then decode the payload of the frame again.
fn again<T: Serialize + DeserializeOwned>(message: &T) -> Option<T> {
    let framed = encode_message(message).ok()?;
    decode_message(framed.get(4..)?).ok()
}
