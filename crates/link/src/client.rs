//! The app's side: keeps one connection to the extension alive and reports what happens.
//! It reconnects by itself with back-off, re-reading the endpoint file on every attempt
//! (a restarted extension has a new port and token). Commands are never queued while the
//! link is down: a stale command played back on stage later is worse than a refused one.

mod client_config;
mod link_event;
mod send_error;
mod worker;

pub use client_config::ClientConfig;
pub use link_event::LinkEvent;
pub use send_error::SendError;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use protocol::Command;

use worker::Worker;

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
