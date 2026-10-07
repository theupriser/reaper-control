//! The extension's side: accepts the app, pushes state, answers commands. Every socket
//! operation runs on its own thread, so a slow or broken client never holds up the caller.

use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use protocol::message::{
    ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage, check_hello, encode_message,
};
use protocol::{AppState, Command};

use crate::endpoint::Endpoint;
use crate::reader::{MessageReader, ReadError};

const MAX_CONNECTIONS: usize = 16;
const OUTBOX: usize = 64;
const POLL: Duration = Duration::from_millis(20);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);

/// What the extension does with a command. Called on a connection thread: hand the work to
/// the right thread and return at once.
pub trait CommandHandler: Send + Sync + 'static {
    /// Decide and report; never block.
    fn handle(&self, command: Command) -> Outcome;
}

/// The server could not start.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Binding or configuring the listener failed.
    #[error("listener: {0}")]
    Io(#[from] io::Error),
    /// No token could be made.
    #[error(transparent)]
    Endpoint(#[from] crate::endpoint::EndpointError),
}

type Frame = Arc<Vec<u8>>;

struct Client {
    id: u64,
    outbox: SyncSender<Frame>,
    stream: TcpStream,
}

#[derive(Default)]
struct Hub {
    state: AppState,
    clients: Vec<Client>,
    next_id: u64,
}

struct Shared {
    token: String,
    handler: Box<dyn CommandHandler>,
    extension_version: String,
    hub: Mutex<Hub>,
    stop: AtomicBool,
    connections: AtomicUsize,
}

impl Shared {
    fn hub(&self) -> MutexGuard<'_, Hub> {
        self.hub.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A running server. Dropping it stops it and closes every client.
pub struct LinkServer {
    shared: Arc<Shared>,
    endpoint: Endpoint,
    accept: Option<JoinHandle<()>>,
}

impl LinkServer {
    /// Listen on a free loopback port with a fresh token.
    pub fn start(
        extension_version: &str,
        handler: impl CommandHandler,
    ) -> Result<Self, ServerError> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let endpoint = Endpoint {
            port: listener.local_addr()?.port(),
            token: Endpoint::new_token()?,
        };
        let shared = Arc::new(Shared {
            token: endpoint.token.clone(),
            handler: Box::new(handler),
            extension_version: extension_version.to_owned(),
            hub: Mutex::new(Hub::default()),
            stop: AtomicBool::new(false),
            connections: AtomicUsize::new(0),
        });
        let accept = {
            let shared = Arc::clone(&shared);
            thread::Builder::new()
                .name("link-accept".into())
                .spawn(move || accept_loop(&listener, &shared))?
        };
        Ok(Self {
            shared,
            endpoint,
            accept: Some(accept),
        })
    }

    /// What to write to the endpoint file.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Address the server listens on.
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::from((Ipv4Addr::LOCALHOST, self.endpoint.port))
    }

    /// Number of clients past the handshake.
    pub fn client_count(&self) -> usize {
        self.shared.hub().clients.len()
    }

    /// Remember `state` for clients that connect later and push it to everyone now.
    /// A client that cannot take it at once is dropped; the caller never waits.
    pub fn publish(&self, state: AppState) {
        let Ok(frame) = encode_message(&ServerMessage::State { state }) else {
            return;
        };
        let frame = Arc::new(frame);
        let mut hub = self.shared.hub();
        hub.state = state;
        hub.clients.retain(|c| {
            let sent = c.outbox.try_send(Arc::clone(&frame)).is_ok();
            if !sent {
                let _ = c.stream.shutdown(std::net::Shutdown::Both);
            }
            sent
        });
    }

    /// Stop accepting and close every connection.
    pub fn stop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            let _ = accept.join();
        }
        for c in self.shared.hub().clients.drain(..) {
            let _ = c.stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

impl Drop for LinkServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn accept_loop(listener: &TcpListener, shared: &Arc<Shared>) {
    while !shared.stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => admit(stream, shared),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(POLL),
            Err(_) => thread::sleep(POLL),
        }
    }
}

fn admit(stream: TcpStream, shared: &Arc<Shared>) {
    if shared.connections.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
        shared.connections.fetch_sub(1, Ordering::SeqCst);
        return;
    }
    let shared = Arc::clone(shared);
    let spawned = thread::Builder::new()
        .name("link-conn".into())
        .spawn(move || {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                let _ = serve(stream, &shared);
            }));
            shared.connections.fetch_sub(1, Ordering::SeqCst);
        });
    drop(spawned);
}

fn serve(stream: TcpStream, shared: &Arc<Shared>) -> Result<(), ReadError> {
    stream.set_nonblocking(false)?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(POLL))?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
    let mut reader = MessageReader::new(stream.try_clone()?);

    let started = Instant::now();
    let first = loop {
        if shared.stop.load(Ordering::SeqCst) || started.elapsed() > HANDSHAKE_TIMEOUT {
            return Ok(());
        }
        if let Some(message) = reader.poll::<ClientMessage>()? {
            break message;
        }
    };
    if check_hello(&shared.token, &first).is_err() {
        return Ok(());
    }

    let (outbox, inbox) = sync_channel::<Frame>(OUTBOX);
    let writer = spawn_writer(stream.try_clone()?, inbox)?;
    let id = register(shared, &stream, &outbox)?;
    let result = serve_commands(&mut reader, shared, &outbox);
    shared.hub().clients.retain(|c| c.id != id);
    let _ = stream.shutdown(std::net::Shutdown::Both);
    drop(outbox);
    let _ = writer.join();
    result
}

fn register(
    shared: &Shared,
    stream: &TcpStream,
    outbox: &SyncSender<Frame>,
) -> Result<u64, ReadError> {
    let welcome = ServerMessage::Welcome {
        protocol: PROTOCOL_VERSION,
        extension_version: shared.extension_version.clone(),
    };
    let mut hub = shared.hub();
    let state = ServerMessage::State { state: hub.state };
    for message in [welcome, state] {
        let frame = encode_message(&message)?;
        outbox
            .try_send(Arc::new(frame))
            .map_err(|_| ReadError::Closed)?;
    }
    hub.next_id += 1;
    let id = hub.next_id;
    hub.clients.push(Client {
        id,
        outbox: outbox.clone(),
        stream: stream.try_clone()?,
    });
    Ok(id)
}

fn serve_commands(
    reader: &mut MessageReader,
    shared: &Shared,
    outbox: &SyncSender<Frame>,
) -> Result<(), ReadError> {
    while !shared.stop.load(Ordering::SeqCst) {
        let reply = match reader.poll::<ClientMessage>()? {
            None => continue,
            Some(ClientMessage::Command { id, command }) => {
                let outcome = catch_unwind(AssertUnwindSafe(|| shared.handler.handle(command)))
                    .unwrap_or_else(|_| Outcome::Rejected {
                        reason: "handler failed".into(),
                    });
                ServerMessage::Ack { id, outcome }
            }
            Some(ClientMessage::Ping) => ServerMessage::Pong,
            Some(ClientMessage::Hello { .. }) => return Ok(()),
        };
        let frame = encode_message(&reply)?;
        outbox
            .try_send(Arc::new(frame))
            .map_err(|_| ReadError::Closed)?;
    }
    Ok(())
}

fn spawn_writer(mut stream: TcpStream, inbox: Receiver<Frame>) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name("link-write".into())
        .spawn(move || {
            while let Ok(frame) = inbox.recv() {
                if stream.write_all(&frame).is_err() {
                    break;
                }
            }
            let _ = stream.shutdown(std::net::Shutdown::Both);
        })
}
