use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::WireEvent;

/// An event with its place in the order of events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EventRecord {
    /// Starts at 1 and rises by one per event, so a receiver can tell what it missed.
    #[ts(type = "number")]
    pub id: u64,
    /// What happened.
    pub event: WireEvent,
}
