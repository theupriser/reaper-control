//! The MIDI section of the v1 `config.json`.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::intent::Intent;
use crate::midi_config::MidiConfig;

/// What v1 stored about MIDI; v1 named notes and actions as text. Values are read loosely so one
/// odd entry is reported instead of failing the whole import.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LegacyMidi {
    enabled: Option<bool>,
    device_name: Option<String>,
    channel: Value,
    note_mapping: BTreeMap<String, Value>,
}

impl LegacyMidi {
    /// The v2 settings, and what v1 stored that has no v2 counterpart (`note: action`, or the channel).
    /// Like v1, notes the file does not map keep their default action.
    #[must_use]
    pub fn into_config(self) -> (MidiConfig, Vec<String>) {
        let mut skipped = Vec::new();
        let channel = match &self.channel {
            Value::Null => None,
            other => match other
                .as_u64()
                .and_then(|channel| u8::try_from(channel).ok())
            {
                Some(channel) if channel <= 15 => Some(channel),
                _ => {
                    skipped.push(format!("channel: {other}"));
                    None
                }
            },
        };
        let mut config = MidiConfig {
            enabled: self.enabled.unwrap_or(true),
            device_name: self.device_name,
            channel,
            ..MidiConfig::default()
        };
        for (note, name) in self.note_mapping {
            let action = name.as_str().and_then(Intent::from_legacy_name);
            match (note.parse::<u8>().ok().filter(|note| *note <= 127), action) {
                (Some(note), Some(action)) => {
                    config.notes.insert(note, action);
                }
                _ => skipped.push(format!("{note}: {}", name.as_str().unwrap_or("?"))),
            }
        }
        (config, skipped)
    }
}
