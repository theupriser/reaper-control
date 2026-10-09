//! Reading and saving the settings the Settings screen edits.

use std::sync::{Arc, Mutex};

use protocol::{Settings, SettingsView};

use crate::app_config::AppConfig;
use crate::command_bus::CommandBus;
use crate::config_error::ConfigError;
use crate::config_repository::ConfigRepository;

/// Keeps the saved config and applies what can change while the app runs.
pub struct SettingsService {
    store: Arc<dyn ConfigRepository>,
    config: Mutex<AppConfig>,
    bus: Arc<CommandBus>,
}

impl SettingsService {
    /// A service over `store`, starting from the config the app started with.
    #[must_use]
    pub fn new(store: Arc<dyn ConfigRepository>, config: AppConfig, bus: Arc<CommandBus>) -> Self {
        Self {
            store,
            config: Mutex::new(config),
            bus,
        }
    }

    /// The saved values with the MIDI devices found now.
    #[must_use]
    pub fn view(&self, devices: Vec<String>) -> SettingsView {
        self.config
            .lock()
            .map(|config| config.view(devices))
            .unwrap_or_else(|poisoned| poisoned.into_inner().view(Vec::new()))
    }

    /// Validates and saves `settings`; the queue limits apply at once.
    ///
    /// # Errors
    /// [`ConfigError::Invalid`] naming the first value out of range, or the write error. Nothing
    /// changes then.
    pub fn save(&self, settings: &Settings) -> Result<(), ConfigError> {
        let mut config = self
            .config
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let changed = config.with_settings(settings);
        self.store.save(&changed)?;
        self.bus.apply(changed.queue.settings());
        *config = changed;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
