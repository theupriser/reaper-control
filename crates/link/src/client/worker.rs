use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use super::handshake::Handshake;
use super::link::{Link, POLL};
use super::session::{CommandQueue, Heartbeat, Session};
use super::{ClientConfig, LinkEvent};

/// The client thread: connect, run the session, report the loss, wait, try again.
pub(super) struct Worker {
    pub(super) config: ClientConfig,
    pub(super) events: Sender<LinkEvent>,
    pub(super) commands: CommandQueue,
    pub(super) connected: Arc<AtomicBool>,
    pub(super) stop: Arc<AtomicBool>,
    pub(super) last_event_id: Cell<Option<u64>>,
}

impl Worker {
    pub(super) fn run(self) {
        let mut backoff = self.config.min_backoff;
        while !self.stop.load(Ordering::SeqCst) {
            let was_up = self.connect_and_serve();
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

    /// One connection from attempt to loss. True when it got past the handshake.
    fn connect_and_serve(&self) -> bool {
        let Some(endpoint) = (self.config.endpoint)() else {
            return false;
        };
        let Some(mut link) = Link::open(&endpoint) else {
            return false;
        };
        let handshake = Handshake { stop: &self.stop };
        let Some((version, welcomed_last)) =
            handshake.perform(&mut link, endpoint.token, self.last_event_id.get())
        else {
            return false;
        };
        if self
            .last_event_id
            .get()
            .is_some_and(|seen| seen > welcomed_last)
        {
            self.last_event_id.set(Some(welcomed_last));
        }
        self.commands.discard();
        self.connected.store(true, Ordering::SeqCst);
        let _ = self.events.send(LinkEvent::Connected {
            extension_version: version,
        });
        Session {
            link,
            heartbeat: Heartbeat::new(self.config.ping_after, self.config.dead_after),
            queue: &self.commands,
            events: &self.events,
            stop: &self.stop,
            last_event_id: &self.last_event_id,
            welcomed_last,
        }
        .run();
        true
    }
}
