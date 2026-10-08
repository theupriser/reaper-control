//! The MIDI settings as stored in the config file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::midi_action::MidiAction;

/// MIDI input settings; missing fields take the defaults v1 had.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MidiConfig {
    /// Whether MIDI input is used at all.
    pub enabled: bool,
    /// The only device to listen to; all devices when empty, as in v1.
    pub device_name: Option<String>,
    /// Only listen to this channel (0 to 15, as v1 counted); all channels when empty.
    pub channel: Option<u8>,
    /// Milliseconds inside which the same note counts as one press (0 to 5000).
    pub debounce_milliseconds: u64,
    /// Which note triggers which action (notes 0 to 127).
    pub notes: BTreeMap<u8, MidiAction>,
}

impl Default for MidiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            device_name: None,
            channel: None,
            debounce_milliseconds: 200,
            notes: BTreeMap::from([
                (44, MidiAction::RestartSong),
                (45, MidiAction::ToggleAutoResume),
                (46, MidiAction::ToggleCountInOnMarker),
                (47, MidiAction::ToggleRecordArm),
                (48, MidiAction::Previous),
                (49, MidiAction::Pause),
                (50, MidiAction::TogglePlay),
                (51, MidiAction::Next),
            ]),
        }
    }
}

impl MidiConfig {
    /// The first value outside its allowed range, as a message.
    #[must_use]
    pub fn problem(&self) -> Option<String> {
        if self.channel.is_some_and(|channel| channel > 15) {
            return Some("midi.channel must be between 0 and 15".into());
        }
        if self.debounce_milliseconds > 5000 {
            return Some("midi.debounce_milliseconds must be at most 5000".into());
        }
        if self.notes.keys().any(|note| *note > 127) {
            return Some("midi.notes must be notes between 0 and 127".into());
        }
        None
    }
}
