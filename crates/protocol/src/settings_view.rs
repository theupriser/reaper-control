use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{NoteMapping, Settings};

/// What the Settings screen shows: the values, the MIDI devices found and the note table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SettingsView {
    /// The saved values.
    pub settings: Settings,
    /// The names of the MIDI input devices found now.
    pub devices: Vec<String>,
    /// The note table, read only until MIDI learn exists.
    pub notes: Vec<NoteMapping>,
}
