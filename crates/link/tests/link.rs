//! The link over real loopback sockets: handshake, push, commands, reconnect, bad peers.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use link::{ClientConfig, CommandHandler, Endpoint, LinkClient, LinkEvent, LinkServer, SendError};
use protocol::message::{ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage, encode_message};
use protocol::{Catalog, Command, Live, Phase};

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Started = (LinkServer, Arc<Mutex<Option<Endpoint>>>);

struct Handler;

impl CommandHandler for Handler {
    #[allow(clippy::panic)] // the panic is what this test double is for
    fn handle(&self, command: Command) -> Outcome {
        match command {
            Command::Play => Outcome::Done,
            Command::Stop => panic!("handler bug"),
            _ => Outcome::Rejected {
                reason: "not in this test".into(),
            },
        }
    }
}

fn playing(position: f64) -> Live {
    Live {
        phase: Phase::Playing,
        position,
        ..Live::default()
    }
}

fn client_for(endpoint: &Arc<Mutex<Option<Endpoint>>>) -> (LinkClient, Receiver<LinkEvent>) {
    client_that_waits(endpoint, Duration::from_millis(600))
}

fn client_that_waits(
    endpoint: &Arc<Mutex<Option<Endpoint>>>,
    dead_after: Duration,
) -> (LinkClient, Receiver<LinkEvent>) {
    let endpoint = Arc::clone(endpoint);
    let mut config = ClientConfig::new(move || endpoint.lock().ok().and_then(|e| e.clone()));
    config.min_backoff = Duration::from_millis(30);
    config.max_backoff = Duration::from_millis(100);
    config.ping_after = Duration::from_millis(150);
    config.quiet_after = Duration::from_millis(300);
    config.dead_after = dead_after;
    LinkClient::start(config)
}

#[allow(clippy::panic)] // a missing event fails the test
fn expect(
    events: &Receiver<LinkEvent>,
    what: &str,
    pick: impl Fn(&LinkEvent) -> bool,
) -> LinkEvent {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(5);
    let mut seen = Vec::new();
    while Instant::now() < deadline {
        if let Ok(event) = events.recv_timeout(Duration::from_millis(50)) {
            if pick(&event) {
                return event;
            }
            seen.push((start.elapsed(), event));
        }
    }
    panic!("no event: {what}; skipped {seen:?}");
}

fn connected(events: &Receiver<LinkEvent>) {
    expect(events, "Connected", |e| {
        matches!(e, LinkEvent::Connected { .. })
    });
}

fn started() -> Result<Started, Box<dyn std::error::Error>> {
    let server = LinkServer::start("0.0.0-test", Handler)?;
    let endpoint = Arc::new(Mutex::new(Some(server.endpoint().clone())));
    Ok((server, endpoint))
}

fn raw_connect(server: &LinkServer) -> std::io::Result<TcpStream> {
    let stream = TcpStream::connect(server.address())?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    Ok(stream)
}

fn is_closed_without_reply(stream: &mut TcpStream) -> bool {
    let mut buf = [0u8; 16];
    matches!(stream.read(&mut buf), Ok(0) | Err(_))
}

#[test]
fn new_client_gets_welcome_then_the_current_state() -> TestResult {
    let (server, endpoint) = started()?;
    server.publish(playing(12.5));
    let (_client, events) = client_for(&endpoint);
    let welcome = expect(&events, "Connected", |e| {
        matches!(e, LinkEvent::Connected { .. })
    });
    assert_eq!(
        welcome,
        LinkEvent::Connected {
            extension_version: "0.0.0-test".into()
        }
    );
    assert_eq!(
        expect(&events, "Live", |e| matches!(e, LinkEvent::Live(_))),
        LinkEvent::Live(playing(12.5))
    );
    Ok(())
}

#[test]
fn a_catalog_reaches_connected_clients_and_late_ones() -> TestResult {
    let (server, endpoint) = started()?;
    let (_early, early_events) = client_for(&endpoint);
    connected(&early_events);
    let catalog = Catalog {
        revision: 2,
        ..Catalog::default()
    };
    server.publish_catalog(catalog.clone());
    expect(&early_events, "pushed catalog", |e| {
        *e == LinkEvent::Catalog(Box::new(catalog.clone()))
    });
    let (_late, late_events) = client_for(&endpoint);
    expect(&late_events, "catalog on connect", |e| {
        *e == LinkEvent::Catalog(Box::new(catalog.clone()))
    });
    Ok(())
}

#[test]
fn published_state_reaches_every_client() -> TestResult {
    let (server, endpoint) = started()?;
    let (_a, events_a) = client_for(&endpoint);
    let (_b, events_b) = client_for(&endpoint);
    connected(&events_a);
    connected(&events_b);
    server.publish(playing(3.0));
    for events in [&events_a, &events_b] {
        expect(events, "pushed state", |e| {
            *e == LinkEvent::Live(playing(3.0))
        });
    }
    Ok(())
}

#[test]
fn command_is_answered_with_the_handlers_outcome() -> TestResult {
    let (_server, endpoint) = started()?;
    let (client, events) = client_for(&endpoint);
    connected(&events);
    let play = client.send(Command::Play)?;
    let seek = client.send(Command::Pause)?;
    let done = expect(
        &events,
        "Ack Done",
        |e| matches!(e, LinkEvent::Ack { id, .. } if *id == play),
    );
    assert_eq!(
        done,
        LinkEvent::Ack {
            id: play,
            outcome: Outcome::Done
        }
    );
    let refused = expect(
        &events,
        "Ack Rejected",
        |e| matches!(e, LinkEvent::Ack { id, .. } if *id == seek),
    );
    assert!(matches!(
        refused,
        LinkEvent::Ack {
            outcome: Outcome::Rejected { .. },
            ..
        }
    ));
    Ok(())
}

#[test]
fn a_panicking_handler_rejects_the_command_and_keeps_the_connection() -> TestResult {
    let (_server, endpoint) = started()?;
    // The first panic on a slow Windows runner can stall the connection thread; this test is
    // about the rejection, not about how long a silent link is tolerated.
    let (client, events) = client_that_waits(&endpoint, Duration::from_secs(4));
    connected(&events);
    let bad = client.send(Command::Stop)?;
    let answer = expect(
        &events,
        "Ack",
        |e| matches!(e, LinkEvent::Ack { id, .. } if *id == bad),
    );
    assert!(matches!(
        answer,
        LinkEvent::Ack {
            outcome: Outcome::Rejected { .. },
            ..
        }
    ));
    let good = client.send(Command::Play)?;
    expect(&events, "Ack Done", |e| {
        *e == LinkEvent::Ack {
            id: good,
            outcome: Outcome::Done,
        }
    });
    Ok(())
}

#[test]
fn wrong_token_and_wrong_version_are_closed_without_a_reply() -> TestResult {
    let (server, _endpoint) = started()?;
    for hello in [
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token: "nope".into(),
            resume_from_event_id: None,
        },
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION + 1,
            token: server.endpoint().token.clone(),
            resume_from_event_id: None,
        },
        ClientMessage::Ping,
    ] {
        let mut stream = raw_connect(&server)?;
        stream.write_all(&encode_message(&hello)?)?;
        assert!(is_closed_without_reply(&mut stream), "{hello:?}");
    }
    assert_eq!(server.client_count(), 0);
    Ok(())
}

#[test]
fn garbage_and_oversized_frames_close_only_that_connection() -> TestResult {
    let (server, endpoint) = started()?;
    let (_client, events) = client_for(&endpoint);
    connected(&events);
    let mut garbage = raw_connect(&server)?;
    garbage.write_all(&[0xff; 64])?;
    assert!(is_closed_without_reply(&mut garbage));
    let mut oversized = raw_connect(&server)?;
    oversized.write_all(&(u32::MAX).to_be_bytes())?;
    assert!(is_closed_without_reply(&mut oversized));
    assert_eq!(server.client_count(), 1);
    server.publish(playing(1.0));
    expect(&events, "state after bad peers", |e| {
        *e == LinkEvent::Live(playing(1.0))
    });
    Ok(())
}

#[test]
fn a_client_that_never_reads_is_dropped_and_publish_stays_fast() -> TestResult {
    let (server, endpoint) = started()?;
    let (_client, events) = client_for(&endpoint);
    connected(&events);
    let mut stuck = raw_connect(&server)?;
    stuck.write_all(&encode_message(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
        token: server.endpoint().token.clone(),
        resume_from_event_id: None,
    })?)?;
    let until = Instant::now() + Duration::from_secs(2);
    while server.client_count() < 2 && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(server.client_count(), 2);

    let started = Instant::now();
    let mut worst = Duration::ZERO;
    for i in 0..200_000 {
        let t = Instant::now();
        server.publish(playing(f64::from(i)));
        worst = worst.max(t.elapsed());
        if server.client_count() == 1 {
            break;
        }
    }
    println!(
        "stuck client dropped after {:?}, slowest publish {worst:?}",
        started.elapsed()
    );
    assert_eq!(server.client_count(), 1, "the stuck client must be dropped");
    assert!(
        worst < Duration::from_millis(50),
        "publish blocked for {worst:?}"
    );
    Ok(())
}

#[test]
fn commands_are_refused_not_queued_while_the_link_is_down() -> TestResult {
    let endpoint = Arc::new(Mutex::new(None));
    let (client, _events) = client_for(&endpoint);
    assert_eq!(client.send(Command::Play), Err(SendError::NotConnected));
    Ok(())
}

#[test]
fn client_reconnects_to_a_restarted_extension_with_a_new_port_and_token() -> TestResult {
    let (mut server, endpoint) = started()?;
    let first_port = server.endpoint().port;
    let (client, events) = client_for(&endpoint);
    connected(&events);

    server.stop();
    expect(&events, "Disconnected", |e| *e == LinkEvent::Disconnected);
    assert!(!client.is_connected());
    assert_eq!(client.send(Command::Play), Err(SendError::NotConnected));

    let restarted = LinkServer::start("0.0.0-test", Handler)?;
    restarted.publish(playing(40.0));
    *endpoint.lock().map_err(|e| e.to_string())? = Some(restarted.endpoint().clone());
    println!(
        "restarted: port {first_port} -> {}",
        restarted.endpoint().port
    );
    connected(&events);
    expect(&events, "replayed state", |e| {
        *e == LinkEvent::Live(playing(40.0))
    });
    let id = client.send(Command::Play)?;
    expect(&events, "Ack", |e| {
        *e == LinkEvent::Ack {
            id,
            outcome: Outcome::Done,
        }
    });
    Ok(())
}

#[test]
fn an_extension_that_goes_silent_is_declared_dead() -> TestResult {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let endpoint = Arc::new(Mutex::new(Some(Endpoint {
        port,
        token: "t".into(),
        protocol: PROTOCOL_VERSION,
    })));
    let (client, events) = client_for(&endpoint);
    let (mut held, _) = listener.accept()?;
    let mut hello = [0u8; 256];
    held.set_read_timeout(Some(Duration::from_secs(2)))?;
    let _ = held.read(&mut hello)?;
    held.write_all(&encode_message(&ServerMessage::Welcome {
        protocol: PROTOCOL_VERSION,
        extension_version: "silent".into(),
        catalog_revision: 0,
        setlist_revision: 0,
        last_event_id: 0,
    })?)?;
    connected(&events);
    assert!(client.is_connected());
    // Welcome only, then silence: no Pong for the pings.
    expect(&events, "Disconnected", |e| *e == LinkEvent::Disconnected);
    assert!(!client.is_connected());
    Ok(())
}

#[test]
fn endpoint_file_round_trips_and_tokens_differ() -> TestResult {
    let dir = std::env::temp_dir().join(format!("link-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("endpoint.json");
    let endpoint = Endpoint {
        port: 4711,
        token: Endpoint::new_token()?,
        protocol: PROTOCOL_VERSION,
    };
    endpoint.write(&path)?;
    assert_eq!(Endpoint::read(&path)?, endpoint);
    assert_eq!(endpoint.token.len(), 64);
    assert_ne!(endpoint.token, Endpoint::new_token()?);
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}

#[test]
fn an_endpoint_with_another_protocol_is_reported_once_and_not_connected_to() -> TestResult {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let endpoint = Arc::new(Mutex::new(Some(Endpoint {
        port: listener.local_addr()?.port(),
        token: "t".into(),
        protocol: 0,
    })));
    let (_client, events) = client_for(&endpoint);
    let first = expect(&events, "Outdated", |e| {
        matches!(e, LinkEvent::Outdated { .. })
    });
    assert_eq!(first, LinkEvent::Outdated { found: 0 });
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        events
            .try_iter()
            .all(|e| !matches!(e, LinkEvent::Outdated { .. })),
        "reported again on a retry"
    );
    assert!(listener.accept().is_err(), "the client connected anyway");
    Ok(())
}

#[test]
fn a_quiet_extension_is_reported_and_then_recovers() -> TestResult {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = Arc::new(Mutex::new(Some(Endpoint {
        port: listener.local_addr()?.port(),
        token: "t".into(),
        protocol: PROTOCOL_VERSION,
    })));
    let (client, events) = client_that_waits(&endpoint, Duration::from_secs(5));
    let (mut held, _) = listener.accept()?;
    let mut hello = [0u8; 256];
    held.set_read_timeout(Some(Duration::from_secs(2)))?;
    let _ = held.read(&mut hello)?;
    held.write_all(&encode_message(&ServerMessage::Welcome {
        protocol: PROTOCOL_VERSION,
        extension_version: "quiet".into(),
        catalog_revision: 0,
        setlist_revision: 0,
        last_event_id: 0,
    })?)?;
    connected(&events);
    expect(&events, "Quiet", |e| *e == LinkEvent::Quiet);
    held.write_all(&encode_message(&ServerMessage::Pong)?)?;
    expect(&events, "Recovered", |e| *e == LinkEvent::Recovered);
    assert!(client.is_connected());
    Ok(())
}

#[test]
fn an_idle_extension_that_answers_its_pings_is_never_quiet() -> TestResult {
    let (server, endpoint) = started()?;
    let (_client, events) = client_for(&endpoint);
    connected(&events);
    std::thread::sleep(Duration::from_millis(1200));
    assert!(
        events.try_iter().all(|e| e != LinkEvent::Quiet),
        "a healthy idle link was reported quiet"
    );
    drop(server);
    Ok(())
}
