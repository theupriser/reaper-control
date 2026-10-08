//! The app's settings file.

use serde::{Deserialize, Serialize};

use protocol::{NoteMapping, Settings, SettingsView};

use crate::config_error::ConfigError;
use crate::log_config::LogConfig;
use crate::midi_config::MidiConfig;
use crate::queue_config::QueueConfig;

/// The schema version this build writes and understands.
pub const SCHEMA_VERSION: u32 = 1;

/// Everything the user can set; missing fields take their defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Which layout of the file this is; see `config_migration`.
    pub schema_version: u32,
    /// Command queue limits.
    pub queue: QueueConfig,
    /// MIDI input.
    pub midi: MidiConfig,
    /// The log level.
    pub log: LogConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            queue: QueueConfig::default(),
            midi: MidiConfig::default(),
            log: LogConfig::default(),
        }
    }
}

impl AppConfig {
    /// Checks every value against its allowed range.
    ///
    /// # Errors
    /// [`ConfigError::Invalid`] naming the first value out of range.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self
            .queue
            .problem()
            .or_else(|| self.midi.problem())
            .or_else(|| self.log.problem())
        {
            Some(problem) => Err(ConfigError::Invalid(problem)),
            None => Ok(()),
        }
    }
}

impl AppConfig {
    /// The values the Settings screen edits.
    #[must_use]
    pub fn settings(&self) -> Settings {
        Settings {
            queue_repeat_window_milliseconds: clamp(self.queue.repeat_window_milliseconds),
            queue_timeout_milliseconds: clamp(self.queue.timeout_milliseconds),
            queue_capacity: clamp(self.queue.capacity),
            midi_enabled: self.midi.enabled,
            midi_device_name: self.midi.device_name.clone(),
            midi_channel: self.midi.channel,
            midi_debounce_milliseconds: clamp(self.midi.debounce_milliseconds),
            log_level: self.log.level.clone(),
        }
    }

    /// What the Settings screen shows: these values, the devices found and the note table.
    #[must_use]
    pub fn view(&self, devices: Vec<String>) -> SettingsView {
        SettingsView {
            settings: self.settings(),
            devices,
            notes: self
                .midi
                .notes
                .iter()
                .map(|(note, intent)| NoteMapping {
                    note: *note,
                    action: intent.label().to_owned(),
                })
                .collect(),
        }
    }

    /// This config with the edited values in place; the note table stays as it is.
    #[must_use]
    pub fn with_settings(&self, settings: &Settings) -> Self {
        let mut config = self.clone();
        config.queue.repeat_window_milliseconds = settings.queue_repeat_window_milliseconds.into();
        config.queue.timeout_milliseconds = settings.queue_timeout_milliseconds.into();
        config.queue.capacity = usize::try_from(settings.queue_capacity).unwrap_or(usize::MAX);
        config.midi.enabled = settings.midi_enabled;
        config.midi.device_name = settings
            .midi_device_name
            .clone()
            .filter(|name| !name.is_empty());
        config.midi.channel = settings.midi_channel;
        config.midi.debounce_milliseconds = settings.midi_debounce_milliseconds.into();
        config.log.level.clone_from(&settings.log_level);
        config
    }
}

fn clamp<T: TryInto<u32>>(value: T) -> u32 {
    value.try_into().unwrap_or(u32::MAX)
}
