//! Reading and writing the config file.

use std::path::PathBuf;

use crate::app_config::AppConfig;
use crate::config_error::ConfigError;
use crate::config_migration::migrate;
use crate::config_repository::ConfigRepository;

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
}

impl ConfigRepository for ConfigStore {
    fn exists(&self) -> bool {
        self.file.is_file()
    }

    fn load(&self) -> Result<AppConfig, ConfigError> {
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

    fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
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
