use protocol::message::{Outcome, ServerMessage};
use protocol::{Catalog, Live};

/// What the app hears from the link.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    /// Handshake done; the extension's version.
    Connected {
        /// Version of the extension build.
        extension_version: String,
    },
    /// The extension pushed the live state.
    Live(Live),
    /// The extension pushed the project contents.
    Catalog(Catalog),
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

impl LinkEvent {
    /// The event a message stands for; `None` for messages the app does not see.
    pub(super) fn from_server(message: ServerMessage) -> Option<Self> {
        match message {
            ServerMessage::Live(live) => Some(Self::Live(live)),
            ServerMessage::Catalog(catalog) => Some(Self::Catalog(catalog)),
            ServerMessage::Ack { id, outcome } => Some(Self::Ack { id, outcome }),
            ServerMessage::Welcome { .. }
            | ServerMessage::Pong
            | ServerMessage::Event(_)
            | ServerMessage::EventsLost { .. } => None,
        }
    }
}
