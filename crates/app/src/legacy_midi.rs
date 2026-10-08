//! The MIDI section of the v1 `config.json`.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::midi_action::MidiAction;
use crate::midi_config::MidiConfig;

/// What v1 stored about MIDI; v1 named notes and actions as text.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LegacyMidi {
    enabled: Option<bool>,
    device_name: Option<String>,
    channel: Option<u8>,
    note_mapping: BTreeMap<String, String>,
}

impl LegacyMidi {
    /// The v2 settings, and the v1 mappings (`note: action`) that have no v2 counterpart.
    #[must_use]
    pub fn into_config(self) -> (MidiConfig, Vec<String>) {
        let mut config = MidiConfig {
            enabled: self.enabled.unwrap_or(true),
            device_name: self.device_name,
            channel: self.channel,
            ..MidiConfig::default()
        };
        if self.note_mapping.is_empty() {
            return (config, Vec::new());
        }
        config.notes.clear();
        let mut skipped = Vec::new();
        for (note, name) in self.note_mapping {
            match (
                note.parse::<u8>().ok().filter(|note| *note <= 127),
                MidiAction::from_legacy_name(&name),
            ) {
                (Some(note), Some(action)) => {
                    config.notes.insert(note, action);
                }
                _ => skipped.push(format!("{note}: {name}")),
            }
        }
        (config, skipped)
    }
}
