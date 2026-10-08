//! The app's settings file.

use serde::{Deserialize, Serialize};

use crate::config_error::ConfigError;
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            queue: QueueConfig::default(),
            midi: MidiConfig::default(),
        }
    }
}

impl AppConfig {
    /// Checks every value against its allowed range.
    ///
    /// # Errors
    /// [`ConfigError::Invalid`] naming the first value out of range.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self.queue.problem().or_else(|| self.midi.problem()) {
            Some(problem) => Err(ConfigError::Invalid(problem)),
            None => Ok(()),
        }
    }
}
