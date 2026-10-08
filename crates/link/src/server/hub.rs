use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use protocol::message::{PROTOCOL_VERSION, ServerMessage, encode_message};
use protocol::{Catalog, Live};

use crate::wire::ReadError;

mod connected_client;

use connected_client::ConnectedClient;
pub(super) use connected_client::Frame;

/// The latest state and everyone who is listening.
#[derive(Default)]
pub(super) struct Hub {
    live: Option<Live>,
    catalog: Option<Catalog>,
    clients: Vec<ConnectedClient>,
    next_id: u64,
}

impl Hub {
    pub(super) fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Welcome and the current state go out first, then the client joins the broadcasts.
    pub(super) fn register(
        &mut self,
        extension_version: &str,
        stream: &TcpStream,
        outbox: &SyncSender<Frame>,
    ) -> Result<u64, ReadError> {
        let welcome = ServerMessage::Welcome {
            protocol: PROTOCOL_VERSION,
            extension_version: extension_version.to_owned(),
            catalog_revision: 0,
            setlist_revision: 0,
            last_event_id: 0,
        };
        let mut messages = vec![welcome];
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

    pub(super) fn close_all(&mut self) {
        for c in self.clients.drain(..) {
            let _ = c.stream.shutdown(Shutdown::Both);
        }
    }
}
