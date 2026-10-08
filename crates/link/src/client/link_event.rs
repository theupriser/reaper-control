use protocol::message::{Outcome, ServerMessage};
use protocol::{Catalog, EventRecord, Live};

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
    /// Something happened in the performance, live or replayed after a reconnect.
    Event(EventRecord),
    /// The app was away too long to be caught up. The catalog is asked for again; treat the
    /// state as new.
    EventsLost {
        /// The oldest event id the extension still holds.
        oldest_available: u64,
    },
    /// A command was answered.
    Ack {
        /// Id returned by [`LinkClient::send`].
        id: u64,
        /// What happened.
        outcome: Outcome,
    },
    /// The connection is gone; the client is trying again.
    Disconnected,
    /// The extension speaks another protocol than the app, so the client does not connect.
    Outdated {
        /// Protocol version the extension announced; 0 when it announced none.
        found: u32,
    },
    /// Connected, but the extension has been silent for a while.
    Quiet,
    /// The extension spoke again after being quiet.
    Recovered,
}

impl LinkEvent {
    /// The event a message stands for; `None` for messages the app does not see.
    pub(super) fn from_server(message: ServerMessage) -> Option<Self> {
        match message {
            ServerMessage::Live(live) => Some(Self::Live(live)),
            ServerMessage::Catalog(catalog) => Some(Self::Catalog(catalog)),
            ServerMessage::Event(record) => Some(Self::Event(record)),
            ServerMessage::EventsLost { oldest_available } => {
                Some(Self::EventsLost { oldest_available })
            }
            ServerMessage::Ack { id, outcome } => Some(Self::Ack { id, outcome }),
            ServerMessage::Welcome { .. } | ServerMessage::Pong => None,
        }
    }
}
