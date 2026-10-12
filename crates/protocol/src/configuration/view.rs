use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ActionChoice, Settings};

/// What the Settings screen shows: the values, the MIDI devices found and the actions a note can have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SettingsView {
    /// The saved values.
    // Boxed: with the note table the settings are over 100 bytes.
    pub settings: Box<Settings>,
    /// The names of the MIDI input devices found now.
    pub devices: Vec<String>,
    /// Every action a note can trigger.
    pub actions: Vec<ActionChoice>,
}
