use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One action a MIDI note can trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ActionChoice {
    /// The id a `NoteMapping` refers to.
    pub id: String,
    /// What it does, in words.
    pub label: String,
}
