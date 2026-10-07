use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use protocol::Command;
use protocol::message::{ClientMessage, PROTOCOL_VERSION, ServerMessage, encode_message};

use super::{ClientConfig, LinkEvent};
use crate::read_error::ReadError;
use crate::reader::MessageReader;

const POLL: Duration = Duration::from_millis(20);

pub(super) struct Worker {
    pub(super) config: ClientConfig,
    pub(super) events: Sender<LinkEvent>,
    pub(super) commands: Mutex<Receiver<(u64, Command)>>,
    pub(super) connected: Arc<AtomicBool>,
    pub(super) stop: Arc<AtomicBool>,
}

impl Worker {
    pub(super) fn run(self) {
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
