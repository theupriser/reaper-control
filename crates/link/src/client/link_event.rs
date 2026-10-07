use protocol::AppState;
use protocol::message::Outcome;

/// What the app hears from the link.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkEvent {
    /// Handshake done; the extension's version.
    Connected {
        /// Version of the extension build.
        extension_version: String,
    },
    /// The extension pushed a state.
    State(AppState),
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
