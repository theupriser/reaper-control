//! Taking the MIDI settings over from v1 on the first start.

use std::path::Path;

use crate::app_config::AppConfig;
use crate::config_store::ConfigStore;
use crate::legacy_config_file::LegacyConfigFile;

/// When the app has no config file yet and v1 left one, the MIDI settings of v1 become the first
/// config, saved at once so this happens once. Returns that config; `None` when there was nothing
/// to take over. What v1 stored without a v2 counterpart is logged, never dropped silently.
#[must_use]
pub fn import_legacy_config(store: &ConfigStore, legacy_file: &Path) -> Option<AppConfig> {
    if store.exists() || !legacy_file.is_file() {
        return None;
    }
    let (midi, skipped) = match LegacyConfigFile::read(legacy_file) {
        Ok(file) => file.midi.into_config(),
        Err(error) => {
            tracing::warn!(%error, "v1 config not imported");
            return None;
        }
    };
    let config = AppConfig {
        midi,
        ..AppConfig::default()
    };
    if let Err(error) = store.save(&config) {
        tracing::warn!(%error, "imported v1 settings not saved");
        return None;
    }
    tracing::info!(?skipped, "v1 midi settings imported");
    Some(config)
}
