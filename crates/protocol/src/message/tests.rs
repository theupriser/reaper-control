use super::*;
use crate::Phase;
use crate::frame::FrameDecoder;

fn roundtrip<T>(message: &T) -> Result<T, Box<dyn std::error::Error>>
where
    T: Serialize + DeserializeOwned,
{
    let mut decoder = FrameDecoder::new();
    decoder.push(&encode_message(message)?);
    let payload = decoder.next_frame()?.ok_or("no frame")?;
    Ok(decode_message(&payload)?)
}

#[test]
fn every_message_survives_a_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    for m in [
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token: "t".into(),
        },
        ClientMessage::Command {
            id: u64::MAX,
            command: Command::Seek {
                position: 12.25,
                count_in: true,
            },
        },
        ClientMessage::Ping,
    ] {
        assert_eq!(roundtrip(&m)?, m);
    }
    for m in [
        ServerMessage::Welcome {
            protocol: PROTOCOL_VERSION,
            extension_version: "0.0.0".into(),
        },
        ServerMessage::State {
            state: AppState {
                phase: Phase::Playing,
                position: 72.5,
                ..AppState::default()
            },
        },
        ServerMessage::Ack {
            id: 7,
            outcome: Outcome::Done,
        },
        ServerMessage::Ack {
            id: 8,
            outcome: Outcome::Rejected {
                reason: "not playing".into(),
            },
        },
        ServerMessage::Pong,
    ] {
        assert_eq!(roundtrip(&m)?, m);
    }
    Ok(())
}

#[test]
fn the_wire_form_is_tagged_json() -> Result<(), serde_json::Error> {
    assert_eq!(
        serde_json::to_string(&ClientMessage::Ping)?,
        r#"{"type":"Ping"}"#
    );
    Ok(())
}

#[test]
fn garbage_is_an_error_not_a_panic() {
    for bad in [
        &b"{"[..],
        b"null",
        b"[]",
        br#"{"type":"Nope"}"#,
        br#"{"type":"Hello","protocol":-1,"token":"x"}"#,
        br#"{"type":"Command","id":1,"command":"Explode"}"#,
        &[0xff, 0xfe, 0x00],
    ] {
        assert!(decode_message::<ClientMessage>(bad).is_err());
    }
    let deep = "[".repeat(10_000);
    assert!(decode_message::<ClientMessage>(deep.as_bytes()).is_err());
}

fn hello(protocol: u32, token: &str) -> ClientMessage {
    ClientMessage::Hello {
        protocol,
        token: token.into(),
    }
}

#[test]
fn the_handshake_checks_kind_token_and_version() {
    assert_eq!(
        check_hello("secret", &hello(PROTOCOL_VERSION, "secret")),
        Ok(())
    );
    assert_eq!(
        check_hello("secret", &ClientMessage::Ping),
        Err(HandshakeError::NotHello)
    );
    assert_eq!(
        check_hello("secret", &hello(PROTOCOL_VERSION, "secreT")),
        Err(HandshakeError::BadToken)
    );
    assert_eq!(
        check_hello("secret", &hello(PROTOCOL_VERSION, "secre")),
        Err(HandshakeError::BadToken)
    );
    assert_eq!(
        check_hello("secret", &hello(PROTOCOL_VERSION + 1, "secret")),
        Err(HandshakeError::ProtocolMismatch {
            got: PROTOCOL_VERSION + 1
        })
    );
}
