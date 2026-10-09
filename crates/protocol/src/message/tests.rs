use super::*;
use crate::frame::FrameDecoder;
use crate::{
    Catalog, Command, CueInfo, EntryInfo, EventRecord, Live, Phase, SetlistInfo, Setting, SongInfo,
    Transport, WireEvent,
};
use serde::{Serialize, de::DeserializeOwned};

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
            resume_from_event_id: Some(41),
        },
        ClientMessage::GetCatalog,
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
            catalog_revision: 3,
            setlist_revision: 5,
            last_event_id: 99,
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
        resume_from_event_id: None,
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

fn live() -> Live {
    Live {
        sequence: 41,
        timestamp: 1234.5,
        transport: Transport::Playing,
        position: 72.25,
        phase: Phase::HandingOver,
        setlist_id: Some("set-1".into()),
        current_song: Some(2),
        next_song: None,
        autoplay: true,
        count_in: false,
        record_armed: true,
        catalog_revision: 3,
        setlist_revision: 5,
    }
}

fn opener() -> SongInfo {
    SongInfo {
        id: "{A}".into(),
        number: 1,
        name: "Opener".into(),
        start: 0.0,
        end: 200.5,
        colour: Some("#ff8800".into()),
        hard_stop: true,
        length: Some(180.0),
        bpm: None,
    }
}

fn catalog() -> Catalog {
    Catalog {
        revision: 3,
        setlist_revision: 5,
        project_id: "project-1".into(),
        songs: vec![opener()],
        project_songs: vec![opener()],
        cues: vec![CueInfo {
            id: "m1".into(),
            name: "Bridge".into(),
            position: 90.0,
        }],
        setlists: vec![SetlistInfo {
            id: "set-1".into(),
            name: "Friday".into(),
            revision: 5,
            entries: vec![EntryInfo {
                id: 1,
                song_id: "{A}".into(),
            }],
        }],
        active_setlist: Some("set-1".into()),
    }
}

fn every_event() -> Vec<WireEvent> {
    vec![
        WireEvent::PerformanceStarted,
        WireEvent::HandOverStarted {
            from: "A".into(),
            to: "B".into(),
        },
        WireEvent::HandOverCompleted {
            song_id: "B".into(),
        },
        WireEvent::HardStopReached {
            song_id: "B".into(),
        },
        WireEvent::PerformanceFinished,
        WireEvent::SettingChanged {
            setting: Setting::CountIn,
            enabled: true,
        },
        WireEvent::SeekPerformed { to: 12.5 },
        WireEvent::ProjectChanged,
        WireEvent::CommandRejected {
            reason: "no next song".into(),
        },
    ]
}

#[test]
fn live_catalog_and_events_survive_a_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = vec![
        ServerMessage::Live(live()),
        ServerMessage::Catalog(Box::new(catalog())),
        ServerMessage::EventsLost {
            oldest_available: 17,
        },
    ];
    for (index, event) in every_event().into_iter().enumerate() {
        messages.push(ServerMessage::Event(EventRecord {
            id: index as u64 + 1,
            event,
        }));
    }
    for m in messages {
        assert_eq!(roundtrip(&m)?, m);
    }
    Ok(())
}

#[test]
fn a_phase_and_a_transport_travel_by_name() -> Result<(), serde_json::Error> {
    let json = serde_json::to_value(ServerMessage::Live(live()))?;
    assert_eq!(json["type"], "Live");
    assert_eq!(json["phase"], "HandingOver");
    assert_eq!(json["transport"], "Playing");
    assert_eq!(json["next_song"], serde_json::Value::Null);
    Ok(())
}

#[test]
fn an_event_is_tagged_by_kind_next_to_its_id() -> Result<(), serde_json::Error> {
    let json = serde_json::to_string(&ServerMessage::Event(EventRecord {
        id: 9,
        event: WireEvent::PerformanceFinished,
    }))?;
    assert_eq!(
        json,
        r#"{"type":"Event","id":9,"event":{"kind":"PerformanceFinished"}}"#
    );
    Ok(())
}

#[test]
fn damaged_new_messages_are_errors_not_panics() {
    for bad in [
        &br#"{"type":"Live"}"#[..],
        br#"{"type":"Live","sequence":-1}"#,
        br#"{"type":"Event","id":1,"event":{"kind":"Explode"}}"#,
        br#"{"type":"Event","id":1}"#,
        br#"{"type":"Catalog","revision":1,"setlist_revision":1,"songs":[{"id":1}],"cues":[],"setlists":[]}"#,
        br#"{"type":"EventsLost"}"#,
        br#"{"type":"Welcome","protocol":2,"extension_version":"x"}"#,
    ] {
        assert!(
            decode_message::<ServerMessage>(bad).is_err(),
            "{}",
            String::from_utf8_lossy(bad)
        );
    }
    assert!(
        decode_message::<ClientMessage>(br#"{"type":"Hello","protocol":2,"token":"t"}"#).is_ok()
    );
}

#[test]
fn an_old_version_is_refused_by_the_handshake() {
    assert_eq!(
        check_hello("secret", &hello(1, "secret")),
        Err(HandshakeError::ProtocolMismatch { got: 1 })
    );
}
