//! Command intake over a raw socket: ids are answered once, unknown commands are refused.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use link::{CommandHandler, LinkServer};
use protocol::Command;
use protocol::frame::{self, FrameDecoder};
use protocol::message::{
    ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage, decode_message, encode_message,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Counting(Arc<AtomicUsize>);

impl CommandHandler for Counting {
    fn handle(&self, _command: Command) -> Outcome {
        self.0.fetch_add(1, Ordering::SeqCst);
        Outcome::Done
    }
}

struct RawClient {
    stream: TcpStream,
    decoder: FrameDecoder,
}

impl RawClient {
    fn connect(server: &LinkServer) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(server.address())?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut client = Self {
            stream,
            decoder: FrameDecoder::new(),
        };
        client.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token: server.endpoint().token.clone(),
            resume_from_event_id: None,
        })?;
        Ok(client)
    }

    fn send(&mut self, message: &ClientMessage) -> TestResult {
        self.stream.write_all(&encode_message(message)?)?;
        Ok(())
    }

    fn send_text(&mut self, text: &str) -> TestResult {
        self.stream.write_all(&frame::encode(text.as_bytes())?)?;
        Ok(())
    }

    fn next_ack(&mut self) -> Result<(u64, Outcome), Box<dyn std::error::Error>> {
        loop {
            while let Some(payload) = self.decoder.next_frame()? {
                if let ServerMessage::Ack { id, outcome } = decode_message(&payload)? {
                    return Ok((id, outcome));
                }
            }
            let mut chunk = [0u8; 4096];
            let read = self.stream.read(&mut chunk)?;
            if read == 0 {
                return Err("closed before an ack".into());
            }
            self.decoder.push(chunk.get(..read).unwrap_or_default());
        }
    }
}

fn command(id: u64, command: Command) -> ClientMessage {
    ClientMessage::Command { id, command }
}

#[test]
fn a_command_sent_again_with_the_same_id_runs_once_and_is_answered_again() -> TestResult {
    let runs = Arc::new(AtomicUsize::new(0));
    let server = LinkServer::start("0.0.0-test", Counting(runs.clone()))?;
    let mut client = RawClient::connect(&server)?;

    client.send(&command(7, Command::Next))?;
    assert_eq!(client.next_ack()?, (7, Outcome::Done));
    client.send(&command(7, Command::Next))?;
    assert_eq!(client.next_ack()?, (7, Outcome::Done));
    assert_eq!(runs.load(Ordering::SeqCst), 1);

    client.send(&command(8, Command::Next))?;
    assert_eq!(client.next_ack()?, (8, Outcome::Done));
    assert_eq!(runs.load(Ordering::SeqCst), 2);
    Ok(())
}

#[test]
fn an_id_is_forgotten_after_many_newer_ones() -> TestResult {
    let runs = Arc::new(AtomicUsize::new(0));
    let server = LinkServer::start("0.0.0-test", Counting(runs.clone()))?;
    let mut client = RawClient::connect(&server)?;
    for id in 1..=300 {
        client.send(&command(id, Command::Play))?;
        client.next_ack()?;
    }
    assert_eq!(runs.load(Ordering::SeqCst), 300);
    client.send(&command(1, Command::Play))?;
    client.next_ack()?;
    assert_eq!(runs.load(Ordering::SeqCst), 301);
    client.send(&command(300, Command::Play))?;
    client.next_ack()?;
    assert_eq!(runs.load(Ordering::SeqCst), 301);
    Ok(())
}

#[test]
fn an_unknown_command_is_refused_and_the_connection_stays() -> TestResult {
    let runs = Arc::new(AtomicUsize::new(0));
    let server = LinkServer::start("0.0.0-test", Counting(runs.clone()))?;
    let mut client = RawClient::connect(&server)?;

    client.send_text(r#"{"type":"Command","id":5,"command":{"type":"FlyToTheMoon"}}"#)?;
    let (id, outcome) = client.next_ack()?;
    assert_eq!(id, 5);
    assert!(matches!(outcome, Outcome::Rejected { .. }), "{outcome:?}");
    assert_eq!(runs.load(Ordering::SeqCst), 0);

    client.send(&command(6, Command::Play))?;
    assert_eq!(client.next_ack()?, (6, Outcome::Done));
    Ok(())
}
