//! The port through which the app keeps its config.

use crate::app_config::AppConfig;
use crate::config_error::ConfigError;

/// Where the saved config lives; the file on disk in production, memory in tests.
pub trait ConfigRepository: Send + Sync {
    /// Whether a config was saved before.
    fn exists(&self) -> bool;

    /// The saved config, migrated and validated; the defaults when nothing was saved yet.
    ///
    /// # Errors
    /// A [`ConfigError`] when the saved config cannot be read or is invalid.
    fn load(&self) -> Result<AppConfig, ConfigError>;

    /// Saves a valid config.
    ///
    /// # Errors
    /// A [`ConfigError`] when the config is invalid or cannot be written.
    fn save(&self, config: &AppConfig) -> Result<(), ConfigError>;
}
