use protocol::EventRecord;

/// The answer to "what did I miss".
#[derive(Debug, Clone, PartialEq)]
pub enum Replay {
    /// Exactly the events after the one the app named, oldest first. May be empty.
    Events(Vec<EventRecord>),
    /// The app is too far behind, or ahead of a restarted extension: it cannot be caught up.
    Lost {
        /// The oldest event id still held.
        oldest_available: u64,
    },
}
