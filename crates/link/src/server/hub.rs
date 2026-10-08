use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use protocol::message::{PROTOCOL_VERSION, ServerMessage, encode_message};
use protocol::{Catalog, Live, WireEvent};

use crate::event_log::{EventLog, Replay};
use crate::wire::ReadError;

mod connected_client;

use connected_client::ConnectedClient;
pub(super) use connected_client::Frame;

/// The latest state and everyone who is listening.
#[derive(Default)]
pub(super) struct Hub {
    live: Option<Live>,
    catalog: Option<Catalog>,
    events: EventLog,
    clients: Vec<ConnectedClient>,
    next_id: u64,
}

impl Hub {
    pub(super) fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Welcome, what the client missed and the current state go out first, then the client joins
    /// the broadcasts. `resume_from` is the last event the client saw.
    pub(super) fn register(
        &mut self,
        extension_version: &str,
        resume_from: Option<u64>,
        stream: &TcpStream,
        outbox: &SyncSender<Frame>,
    ) -> Result<u64, ReadError> {
        let (catalog_revision, setlist_revision) =
            self.catalog.as_ref().map_or((0, 0), |catalog| {
                (catalog.revision, catalog.setlist_revision)
            });
        let welcome = ServerMessage::Welcome {
            protocol: PROTOCOL_VERSION,
            extension_version: extension_version.to_owned(),
            catalog_revision,
            setlist_revision,
            last_event_id: self.events.last_id(),
        };
        let mut messages = vec![welcome];
        match resume_from.map(|after| self.events.since(after)) {
            None => {}
            Some(Replay::Events(records)) => {
                messages.extend(records.into_iter().map(ServerMessage::Event));
            }
            Some(Replay::Lost { oldest_available }) => {
                messages.push(ServerMessage::EventsLost { oldest_available });
            }
        }
        messages.extend(self.live.clone().map(ServerMessage::Live));
        messages.extend(self.catalog.clone().map(ServerMessage::Catalog));
        for message in messages {
            outbox
                .try_send(Arc::new(encode_message(&message)?))
                .map_err(|_| ReadError::Closed)?;
        }
        self.next_id += 1;
        self.clients.push(ConnectedClient {
            id: self.next_id,
            outbox: outbox.clone(),
            stream: stream.try_clone()?,
        });
        Ok(self.next_id)
    }

    pub(super) fn remove(&mut self, id: u64) {
        self.clients.retain(|c| c.id != id);
    }

    /// Remember `live` and push it to everyone; a client that cannot take it at once is dropped.
    pub(super) fn broadcast(&mut self, live: Live, frame: &Frame) {
        self.live = Some(live);
        self.send_to_all(frame);
    }

    fn send_to_all(&mut self, frame: &Frame) {
        self.clients.retain(|c| {
            let sent = c.outbox.try_send(Arc::clone(frame)).is_ok();
            if !sent {
                let _ = c.stream.shutdown(Shutdown::Both);
            }
            sent
        });
    }

    /// Remember `catalog` for clients that connect later and push it to everyone.
    pub(super) fn broadcast_catalog(&mut self, catalog: Catalog, frame: &Frame) {
        self.catalog = Some(catalog);
        self.send_to_all(frame);
    }

    /// Number the event, keep it for clients that were away and push it to everyone.
    pub(super) fn broadcast_event(&mut self, event: WireEvent) {
        let record = self.events.push(event);
        if let Ok(frame) = encode_message(&ServerMessage::Event(record)) {
            self.send_to_all(&Arc::new(frame));
        }
    }

    /// The current catalog as a frame, for a client that asked for it.
    pub(super) fn catalog_frame(&self) -> Option<Frame> {
        let catalog = self.catalog.clone()?;
        encode_message(&ServerMessage::Catalog(catalog))
            .ok()
            .map(Arc::new)
    }

    pub(super) fn close_all(&mut self) {
        for c in self.clients.drain(..) {
            let _ = c.stream.shutdown(Shutdown::Both);
        }
    }
}
