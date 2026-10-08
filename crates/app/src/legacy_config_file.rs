//! The v1 `config.json`.

use std::path::Path;

use serde::Deserialize;

use crate::import_error::ImportError;
use crate::legacy_midi::LegacyMidi;

/// The parts of the v1 `config.json` that v2 has a counterpart for. The rest (host, port,
/// polling and reconnect settings) disappeared because the extension runs the timing itself.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LegacyConfigFile {
    /// The MIDI section.
    pub midi: LegacyMidi,
}

impl LegacyConfigFile {
    /// Reads the file.
    ///
    /// # Errors
    /// [`ImportError`] when the file cannot be read or is not JSON of the v1 shape.
    pub fn read(path: &Path) -> Result<Self, ImportError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
}
