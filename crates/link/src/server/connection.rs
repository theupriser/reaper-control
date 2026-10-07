use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use protocol::message::{
    ClientMessage, Outcome, PROTOCOL_VERSION, ServerMessage, check_hello, encode_message,
};

use super::connected_client::{Client, Frame};
use super::shared::Shared;
use crate::read_error::ReadError;
use crate::reader::MessageReader;

const MAX_CONNECTIONS: usize = 16;
const OUTBOX: usize = 64;
const POLL: Duration = Duration::from_millis(20);
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const WRITE_TIMEOUT: Duration = Duration::from_secs(1);

pub(super) fn accept_loop(listener: &TcpListener, shared: &Arc<Shared>) {
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
