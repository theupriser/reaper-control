//! The app's side: keeps one connection to the extension alive and reports what happens.
//! It reconnects by itself with back-off, re-reading the endpoint file on every attempt
//! (a restarted extension has a new port and token). Commands are never queued while the
//! link is down: a stale command played back on stage later is worse than a refused one.

use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use protocol::message::{ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage, encode_message};
use protocol::{AppState, Command};

use crate::endpoint::Endpoint;
use crate::reader::{MessageReader, ReadError};

const POLL: Duration = Duration::from_millis(20);

/// How the client finds and watches the extension.
pub struct ClientConfig {
    /// Called before every connection attempt; returns the current endpoint.
    pub endpoint: Box<dyn Fn() -> Option<Endpoint> + Send>,
    /// First wait after a failure; doubles up to `max_backoff`.
    pub min_backoff: Duration,
    /// Longest wait between attempts.
    pub max_backoff: Duration,
    /// Send a Ping after this much silence.
    pub ping_after: Duration,
    /// Declare the link dead after this much silence.
    pub dead_after: Duration,
}

impl ClientConfig {
    /// Defaults for the real app.
    pub fn new(endpoint: impl Fn() -> Option<Endpoint> + Send + 'static) -> Self {
        Self {
            endpoint: Box::new(endpoint),
            min_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(2),
            ping_after: Duration::from_secs(1),
            dead_after: Duration::from_secs(3),
        }
    }
}

/// What the app hears from the link.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    /// Handshake done; the extension's version.
    Connected {
        /// Version of the extension build.
        extension_version: String,
    },
    /// The extension pushed a state.
    State(AppState),
    /// A command was answered.
    Ack {
        /// Id returned by [`LinkClient::send`].
        id: u64,
        /// What happened.
        outcome: Outcome,
    },
    /// The connection is gone; the client is trying again.
    Disconnected,
}

/// A command was not sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SendError {
    /// No live connection. The command is dropped, not queued.
    #[error("not connected to the extension")]
    NotConnected,
}

/// A running client. Dropping it stops the thread.
pub struct LinkClient {
    outgoing: Sender<(u64, Command)>,
    connected: Arc<AtomicBool>,
    next_id: AtomicU64,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl LinkClient {
    /// Start connecting. Events arrive on the returned receiver.
    pub fn start(config: ClientConfig) -> (Self, Receiver<LinkEvent>) {
        let (event_tx, event_rx) = channel();
        let (outgoing, command_rx) = channel();
        let connected = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let worker = Worker {
            config,
            events: event_tx,
            commands: Mutex::new(command_rx),
            connected: Arc::clone(&connected),
            stop: Arc::clone(&stop),
        };
        let thread = thread::Builder::new()
            .name("link-client".into())
            .spawn(move || worker.run())
            .ok();
        let client = Self {
            outgoing,
            connected,
            next_id: AtomicU64::new(1),
            stop,
            thread,
        };
        (client, event_rx)
    }

    /// Whether the handshake is done and the connection is believed alive.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Send a command; the answer arrives as [`LinkEvent::Ack`] with the returned id.
    pub fn send(&self, command: Command) -> Result<u64, SendError> {
        if !self.is_connected() {
            return Err(SendError::NotConnected);
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.outgoing
            .send((id, command))
            .map_err(|_| SendError::NotConnected)?;
        Ok(id)
    }
}

impl Drop for LinkClient {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Worker {
    config: ClientConfig,
    events: Sender<LinkEvent>,
    commands: Mutex<Receiver<(u64, Command)>>,
    connected: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}

impl Worker {
    fn run(self) {
        let mut backoff = self.config.min_backoff;
        while !self.stop.load(Ordering::SeqCst) {
            let was_up = self.session().is_some_and(|up| up);
            if self.connected.swap(false, Ordering::SeqCst) || was_up {
                let _ = self.events.send(LinkEvent::Disconnected);
                backoff = self.config.min_backoff;
            }
            self.wait(backoff);
            backoff = (backoff * 2).min(self.config.max_backoff);
        }
    }

    fn wait(&self, total: Duration) {
        let until = Instant::now() + total;
        while Instant::now() < until && !self.stop.load(Ordering::SeqCst) {
            thread::sleep(POLL);
        }
    }

    /// One connection from attempt to loss. `Some(true)` when it got past the handshake.
    fn session(&self) -> Option<bool> {
        let endpoint = (self.config.endpoint)()?;
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
        let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(1)).ok()?;
        stream.set_nodelay(true).ok()?;
        stream.set_read_timeout(Some(POLL)).ok()?;
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .ok()?;
        let mut reader = MessageReader::new(stream.try_clone().ok()?);
        send(
            &mut stream,
            &ClientMessage::Hello {
                protocol: PROTOCOL_VERSION,
                token: endpoint.token,
            },
        )?;
        let version = self.await_welcome(&mut reader)?;
        // Anything typed while the link was down is stale.
        let stale = self.commands.lock().unwrap_or_else(PoisonError::into_inner);
        while stale.try_recv().is_ok() {}
        drop(stale);
        self.connected.store(true, Ordering::SeqCst);
        let _ = self.events.send(LinkEvent::Connected {
            extension_version: version,
        });
        let _ = self.talk(&mut stream, &mut reader);
        Some(true)
    }

    fn await_welcome(&self, reader: &mut MessageReader) -> Option<String> {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) && !self.stop.load(Ordering::SeqCst) {
            match reader.poll::<ServerMessage>().ok()? {
                Some(ServerMessage::Welcome {
                    protocol,
                    extension_version,
                }) if protocol == PROTOCOL_VERSION => return Some(extension_version),
                Some(_) => return None,
                None => {}
            }
        }
        None
    }

    fn talk(&self, stream: &mut TcpStream, reader: &mut MessageReader) -> Result<(), ReadError> {
        let mut last_heard = Instant::now();
        let mut last_ping = Instant::now();
        while !self.stop.load(Ordering::SeqCst) {
            match reader.poll::<ServerMessage>()? {
                Some(message) => {
                    last_heard = Instant::now();
                    self.report(message);
                }
                None if last_heard.elapsed() > self.config.dead_after => {
                    return Err(ReadError::Closed);
                }
                None => {}
            }
            if last_heard.elapsed() > self.config.ping_after
                && last_ping.elapsed() > self.config.ping_after
            {
                last_ping = Instant::now();
                send(stream, &ClientMessage::Ping).ok_or(ReadError::Closed)?;
            }
            loop {
                let next = self
                    .commands
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .try_recv();
                match next {
                    Ok((id, command)) => {
                        send(stream, &ClientMessage::Command { id, command })
                            .ok_or(ReadError::Closed)?;
                    }
                    Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
                }
            }
        }
        Ok(())
    }

    fn report(&self, message: ServerMessage) {
        let event = match message {
            ServerMessage::State { state } => LinkEvent::State(state),
            ServerMessage::Ack { id, outcome } => LinkEvent::Ack { id, outcome },
            ServerMessage::Welcome { .. } | ServerMessage::Pong => return,
        };
        let _ = self.events.send(event);
    }
}

fn send(stream: &mut TcpStream, message: &ClientMessage) -> Option<()> {
    stream.write_all(&encode_message(message).ok()?).ok()
}
