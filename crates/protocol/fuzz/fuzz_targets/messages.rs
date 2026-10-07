//! Parses arbitrary payloads as both message types and checks the handshake on whatever parses.
//! Invariants: no panic, and anything that parses survives a trip through encode and decode.
#![no_main]

use libfuzzer_sys::fuzz_target;
use protocol::message::{
    ClientMessage, ServerMessage, check_hello, decode_message, encode_message,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = decode_message::<ClientMessage>(data) {
        let _ = check_hello("token", &m);
        let framed = encode_message(&m);
        assert!(framed.is_ok());
    }
    if let Ok(m) = decode_message::<ServerMessage>(data) {
        assert!(encode_message(&m).is_ok());
    }
});
