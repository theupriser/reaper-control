use serde::{Deserialize, Serialize};

use super::Outcome;
use crate::AppState;

/// Extension to app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Answer to an accepted Hello.
    Welcome {
        /// Protocol version the extension speaks.
        protocol: u32,
        /// Version of the extension build.
        extension_version: String,
    },
    /// Pushed whenever the state changes.
    State {
        /// The new state.
        state: AppState,
    },
    /// Answer to a Command.
    Ack {
        /// The id from the Command.
        id: u64,
        /// What happened to it.
        outcome: Outcome,
    },
    /// Answer to a Ping.
    Pong,
}
