use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

use protocol::AppState;
use protocol::message::{PROTOCOL_VERSION, ServerMessage, encode_message};

use crate::wire::ReadError;

mod connected_client;

use connected_client::ConnectedClient;
pub(super) use connected_client::Frame;

/// The latest state and everyone who is listening.
#[derive(Default)]
pub(super) struct Hub {
    state: AppState,
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
        for message in [welcome, ServerMessage::State { state: self.state }] {
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

    /// Remember `state` and push it to everyone; a client that cannot take it at once is dropped.
    pub(super) fn broadcast(&mut self, state: AppState, frame: &Frame) {
        self.state = state;
        self.clients.retain(|c| {
            let sent = c.outbox.try_send(Arc::clone(frame)).is_ok();
            if !sent {
                let _ = c.stream.shutdown(Shutdown::Both);
            }
            sent
        });
    }

    pub(super) fn close_all(&mut self) {
        for c in self.clients.drain(..) {
            let _ = c.stream.shutdown(Shutdown::Both);
        }
    }
}
