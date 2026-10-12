use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One MIDI note and the action it triggers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct NoteMapping {
    /// The MIDI note number (0 to 127).
    pub note: u8,
    /// The id of the action, as listed in `SettingsView::actions`.
    pub action: String,
}
