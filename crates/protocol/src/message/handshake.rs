use super::{ClientMessage, HandshakeError, PROTOCOL_VERSION};

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
