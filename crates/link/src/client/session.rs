use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use protocol::message::ClientMessage;

use super::LinkEvent;
use super::link::Link;

mod command_queue;
mod heartbeat;

use crate::wire::ReadError;
pub(super) use command_queue::CommandQueue;
pub(super) use heartbeat::Heartbeat;

/// A connection that got past the handshake: forwards commands, reports what arrives,
/// pings when quiet and ends when the extension goes silent.
pub(super) struct Session<'a> {
    pub(super) link: Link,
    pub(super) heartbeat: Heartbeat,
    pub(super) queue: &'a CommandQueue,
    pub(super) events: &'a Sender<LinkEvent>,
    pub(super) stop: &'a AtomicBool,
    pub(super) last_event_id: &'a Cell<Option<u64>>,
    pub(super) welcomed_last: u64,
    pub(super) quiet: bool,
}

impl Session<'_> {
    pub(super) fn run(mut self) {
        let _ = self.talk();
    }

    fn talk(&mut self) -> Result<(), ReadError> {
        while !self.stop.load(Ordering::SeqCst) {
            match self.link.poll()? {
                Some(message) => {
                    self.heartbeat.heard();
                    if std::mem::take(&mut self.quiet) {
                        let _ = self.events.send(LinkEvent::Recovered);
                    }
                    if let Some(event) = LinkEvent::from_server(message) {
                        self.note(&event)?;
                        let _ = self.events.send(event);
                    }
                }
                None if self.heartbeat.is_dead() => return Err(ReadError::Closed),
                None => {}
            }
            if self.heartbeat.is_quiet() && !std::mem::replace(&mut self.quiet, true) {
                let _ = self.events.send(LinkEvent::Quiet);
            }
            if self.heartbeat.ping_due() {
                self.link
                    .send(&ClientMessage::Ping)
                    .ok_or(ReadError::Closed)?;
            }
            while let Some((id, command)) = self.queue.next() {
                self.link
                    .send(&ClientMessage::Command { id, command })
                    .ok_or(ReadError::Closed)?;
            }
        }
        Ok(())
    }

    /// Remember how far the events go, and ask for the catalog again when some were lost.
    fn note(&mut self, event: &LinkEvent) -> Result<(), ReadError> {
        match event {
            LinkEvent::Event(record) => self.last_event_id.set(Some(record.id)),
            LinkEvent::EventsLost { .. } => {
                self.last_event_id.set(Some(self.welcomed_last));
                self.link
                    .send(&ClientMessage::GetCatalog)
                    .ok_or(ReadError::Closed)?;
            }
            _ => {}
        }
        Ok(())
    }
}
