use serde::{Deserialize, Serialize};

use super::Outcome;
use crate::{AppState, Catalog, EventRecord, Live};

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
        /// Revision of the catalog the extension holds.
        catalog_rev: u64,
        /// Revision of the setlists the extension holds.
        setlist_rev: u64,
        /// Id of the newest event, 0 when there is none yet.
        last_event_id: u64,
    },
    /// Pushed on change and as heartbeat; see [`Live`].
    Live(Live),
    /// The project contents; see [`Catalog`].
    Catalog(Catalog),
    /// An event, live or replayed after a reconnect.
    Event(EventRecord),
    /// The app asked to resume from an event the extension no longer holds.
    /// It must ask for the catalog and treat its state as new.
    EventsLost {
        /// The oldest event id still available.
        oldest_available: u64,
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
