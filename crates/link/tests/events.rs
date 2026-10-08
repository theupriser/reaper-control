//! Events over real loopback sockets: numbered, replayed after a reconnect, lost when too old.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use link::{CommandHandler, LinkServer};
use protocol::frame::FrameDecoder;
use protocol::message::{ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage};
use protocol::message::{decode_message, encode_message};
use protocol::{Catalog, Command, EventRecord, WireEvent};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Refuses;

impl CommandHandler for Refuses {
    fn handle(&self, _command: Command) -> Outcome {
        Outcome::Done
    }
}

/// A client that speaks the protocol by hand, so the test sees every message in order.
struct RawClient {
    stream: TcpStream,
    decoder: FrameDecoder,
}

impl RawClient {
    fn join(server: &LinkServer, resume_from_event_id: Option<u64>) -> std::io::Result<Self> {
        let mut client = Self {
            stream: TcpStream::connect(server.address())?,
            decoder: FrameDecoder::new(),
        };
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(300)))?;
        client.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token: server.endpoint().token.clone(),
            resume_from_event_id,
        })?;
        Ok(client)
    }

    fn send(&mut self, message: &ClientMessage) -> std::io::Result<()> {
        let bytes = encode_message(message).map_err(std::io::Error::other)?;
        self.stream.write_all(&bytes)
    }

    /// Everything that arrives until the server is quiet for a moment.
    fn everything(&mut self) -> Result<Vec<ServerMessage>, Box<dyn std::error::Error>> {
        let mut messages = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            while let Some(frame) = self.decoder.next_frame()? {
                messages.push(decode_message(&frame)?);
            }
            match self.stream.read(&mut buffer) {
                Ok(0) => return Ok(messages),
                Ok(read) => self.decoder.push(buffer.get(..read).unwrap_or_default()),
                Err(_) => return Ok(messages),
            }
        }
    }
}

fn server_with_events(count: usize) -> Result<LinkServer, Box<dyn std::error::Error>> {
    let server = LinkServer::start("0.0.0-test", Refuses)?;
    for _ in 0..count {
        server.publish_event(WireEvent::PerformanceFinished);
    }
    Ok(server)
}

fn event_ids(messages: &[ServerMessage]) -> Vec<u64> {
    messages
        .iter()
        .filter_map(|message| match message {
            ServerMessage::Event(EventRecord { id, .. }) => Some(*id),
            _ => None,
        })
        .collect()
}

#[test]
fn welcome_names_the_newest_event_and_the_revisions() -> TestResult {
    let server = server_with_events(3)?;
    server.publish_catalog(Catalog {
        revision: 4,
        setlist_revision: 2,
        ..Catalog::default()
    });
    let messages = RawClient::join(&server, None)?.everything()?;
    assert!(matches!(
        messages.first(),
        Some(ServerMessage::Welcome {
            catalog_revision: 4,
            setlist_revision: 2,
            last_event_id: 3,
            ..
        })
    ));
    assert!(
        event_ids(&messages).is_empty(),
        "first contact replays nothing"
    );
    Ok(())
}

#[test]
fn a_client_that_was_away_gets_exactly_the_events_it_missed() -> TestResult {
    let server = server_with_events(5)?;
    let messages = RawClient::join(&server, Some(2))?.everything()?;
    assert_eq!(event_ids(&messages), vec![3, 4, 5]);
    Ok(())
}

#[test]
fn a_client_that_is_up_to_date_gets_nothing_replayed() -> TestResult {
    let server = server_with_events(2)?;
    let messages = RawClient::join(&server, Some(2))?.everything()?;
    assert!(event_ids(&messages).is_empty());
    assert!(
        !messages
            .iter()
            .any(|m| matches!(m, ServerMessage::EventsLost { .. }))
    );
    Ok(())
}

#[test]
fn a_client_ahead_of_a_restarted_extension_is_told_events_were_lost() -> TestResult {
    let server = server_with_events(1)?;
    let messages = RawClient::join(&server, Some(40))?.everything()?;
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, ServerMessage::EventsLost { .. })),
        "{messages:?}"
    );
    Ok(())
}

#[test]
fn events_reach_a_connected_client_in_order() -> TestResult {
    let server = server_with_events(0)?;
    let mut client = RawClient::join(&server, None)?;
    client.everything()?;
    server.publish_event(WireEvent::PerformanceStarted);
    server.publish_event(WireEvent::PerformanceFinished);
    assert_eq!(event_ids(&client.everything()?), vec![1, 2]);
    Ok(())
}

#[test]
fn get_catalog_answers_with_the_current_catalog() -> TestResult {
    let server = server_with_events(0)?;
    let mut client = RawClient::join(&server, None)?;
    client.everything()?;
    server.publish_catalog(Catalog {
        revision: 7,
        ..Catalog::default()
    });
    client.everything()?;
    client.send(&ClientMessage::GetCatalog)?;
    let messages = client.everything()?;
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, ServerMessage::Catalog(c) if c.revision == 7)),
        "{messages:?}"
    );
    Ok(())
}
