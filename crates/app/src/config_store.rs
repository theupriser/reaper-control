//! Reading and writing the config file.

use std::path::PathBuf;

use crate::app_config::AppConfig;
use crate::config_error::ConfigError;
use crate::config_migration::migrate;

/// The config file on disk.
#[derive(Debug, Clone)]
pub struct ConfigStore {
    file: PathBuf,
}

impl ConfigStore {
    /// A store for this file; nothing is read yet.
    #[must_use]
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    /// The saved config, migrated and validated; the defaults when there is no file yet.
    ///
    /// # Errors
    /// A [`ConfigError`] when the file cannot be read, parsed, migrated or fails validation.
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let text = match std::fs::read_to_string(&self.file) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(AppConfig::default());
            }
            Err(error) => return Err(error.into()),
        };
        let config: AppConfig = serde_json::from_value(migrate(serde_json::from_str(&text)?)?)?;
        config.validate()?;
        Ok(config)
    }

    /// Writes the config next to the file and renames it into place, so a crash never leaves half a file.
    ///
    /// # Errors
    /// A [`ConfigError`] when the config is invalid or the file cannot be written.
    pub fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        config.validate()?;
        if let Some(directory) = self.file.parent() {
            std::fs::create_dir_all(directory)?;
        }
        let temporary = self.file.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(config)?)?;
        std::fs::rename(&temporary, &self.file)?;
        Ok(())
    }
}
