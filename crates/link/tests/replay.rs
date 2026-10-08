//! The real client loses its connection and comes back: it gets exactly the events it missed.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use link::{ClientConfig, CommandHandler, Endpoint, LinkClient, LinkEvent, LinkServer};
use protocol::message::Outcome;
use protocol::{Command, WireEvent};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Refuses;

impl CommandHandler for Refuses {
    fn handle(&self, _command: Command) -> Outcome {
        Outcome::Done
    }
}

/// Forwards one connection at a time to the server, and can cut it.
struct CuttableProxy {
    port: u16,
    open: Arc<Mutex<Vec<TcpStream>>>,
}

impl CuttableProxy {
    fn start(server_port: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        let open = Arc::new(Mutex::new(Vec::new()));
        let registry = Arc::clone(&open);
        thread::spawn(move || {
            for client in listener.incoming().flatten() {
                let Ok(server) = TcpStream::connect(("127.0.0.1", server_port)) else {
                    continue;
                };
                if let (Ok(a), Ok(b)) = (client.try_clone(), server.try_clone())
                    && let Ok(mut registry) = registry.lock()
                {
                    registry.push(a);
                    registry.push(b);
                }
                pump(client.try_clone(), server.try_clone());
                pump(server.try_clone(), client.try_clone());
            }
        });
        Ok(Self { port, open })
    }

    fn cut(&self) {
        if let Ok(mut open) = self.open.lock() {
            for stream in open.drain(..) {
                let _ = stream.shutdown(Shutdown::Both);
            }
        }
    }
}

fn pump(from: std::io::Result<TcpStream>, to: std::io::Result<TcpStream>) {
    let (Ok(mut from), Ok(mut to)) = (from, to) else {
        return;
    };
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(read) = from.read(&mut buffer) {
            if read == 0
                || to
                    .write_all(buffer.get(..read).unwrap_or_default())
                    .is_err()
            {
                break;
            }
        }
        let _ = to.shutdown(Shutdown::Both);
    });
}

#[allow(clippy::panic)] // a missing event fails the test
fn expect(events: &Receiver<LinkEvent>, what: &str, pick: impl Fn(&LinkEvent) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if events
            .recv_timeout(Duration::from_millis(50))
            .is_ok_and(|event| pick(&event))
        {
            return;
        }
    }
    panic!("no event: {what}");
}

fn event_id(event: &LinkEvent) -> Option<u64> {
    match event {
        LinkEvent::Event(record) => Some(record.id),
        _ => None,
    }
}

#[test]
fn a_client_that_loses_its_connection_gets_the_events_it_missed_once() -> TestResult {
    let server = LinkServer::start("0.0.0-test", Refuses)?;
    let proxy = CuttableProxy::start(server.endpoint().port)?;
    let endpoint = Endpoint {
        port: proxy.port,
        ..server.endpoint().clone()
    };
    let mut config = ClientConfig::new(move || Some(endpoint.clone()));
    config.min_backoff = Duration::from_millis(30);
    config.max_backoff = Duration::from_millis(100);
    let (_client, events) = LinkClient::start(config);
    expect(&events, "Connected", |e| {
        matches!(e, LinkEvent::Connected { .. })
    });

    server.publish_event(WireEvent::PerformanceFinished);
    server.publish_event(WireEvent::PerformanceFinished);
    expect(&events, "event 2", |e| event_id(e) == Some(2));

    proxy.cut();
    expect(&events, "Disconnected", |e| *e == LinkEvent::Disconnected);
    server.publish_event(WireEvent::PerformanceFinished);
    server.publish_event(WireEvent::PerformanceFinished);

    expect(&events, "Connected again", |e| {
        matches!(e, LinkEvent::Connected { .. })
    });
    let mut replayed = Vec::new();
    let deadline = Instant::now() + Duration::from_millis(500);
    while Instant::now() < deadline {
        if let Ok(event) = events.recv_timeout(Duration::from_millis(50)) {
            replayed.extend(event_id(&event));
        }
    }
    assert_eq!(
        replayed,
        vec![3, 4],
        "exactly the missed events, no repeats"
    );
    Ok(())
}
