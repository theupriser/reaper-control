use serde::{Deserialize, Serialize};

use crate::Command;

/// App to extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// First message on a connection.
    Hello {
        /// Protocol version the app speaks.
        protocol: u32,
        /// Token from the endpoint file.
        token: String,
        /// Id of the last event the app saw; everything after it is replayed. None on first contact.
        resume_from_event_id: Option<u64>,
    },
    /// Ask for the catalog now; answered by a [`ServerMessage::Catalog`].
    GetCatalog,
    /// Ask the extension to do something; answered by an [`ServerMessage::Ack`].
    Command {
        /// Chosen by the app, echoed in the Ack.
        id: u64,
        /// What to do.
        command: Command,
    },
    /// Liveness check, answered by [`ServerMessage::Pong`].
    Ping,
}
