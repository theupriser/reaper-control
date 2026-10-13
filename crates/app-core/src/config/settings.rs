//! Reading and saving the settings the Settings screen edits.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use protocol::{Settings, SettingsView};

use crate::command::CommandBus;
use crate::config::AppConfig;
use crate::config::ConfigError;
use crate::config::ConfigRepository;
use crate::config::MidiConfig;

/// Keeps the saved config and applies what can change while the app runs.
pub struct SettingsService {
    store: Arc<dyn ConfigRepository>,
    // Boxed: a config is 120 bytes.
    config: Mutex<Box<AppConfig>>,
    bus: Arc<CommandBus>,
    // Told the new MIDI settings when they change, so input follows without a restart.
    midi_changed: Arc<dyn Fn(&MidiConfig) + Send + Sync>,
    // Follows the "close with REAPER" choice, so it applies without a restart.
    close_with_reaper: Arc<AtomicBool>,
}

impl SettingsService {
    /// A service over `store`, starting from the config the app started with.
    #[must_use]
    pub fn new(
        store: Arc<dyn ConfigRepository>,
        config: AppConfig,
        bus: Arc<CommandBus>,
        midi_changed: Arc<dyn Fn(&MidiConfig) + Send + Sync>,
        close_with_reaper: Arc<AtomicBool>,
    ) -> Self {
        close_with_reaper.store(config.close_with_reaper, Ordering::Relaxed);
        Self {
            store,
            config: Mutex::new(Box::new(config)),
            bus,
            midi_changed,
            close_with_reaper,
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

    /// Validates and saves `settings`; the queue limits and the MIDI settings apply at once.
    ///
    /// # Errors
    /// [`ConfigError::Invalid`] naming the first value out of range, or the write error. Nothing
    /// changes then.
    pub fn save(&self, settings: &Settings) -> Result<(), ConfigError> {
        let mut config = self
            .config
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let changed = config.with_settings(settings)?;
        self.store.save(&changed)?;
        self.bus.apply(changed.queue.settings());
        let midi_differs = changed.midi != config.midi;
        self.close_with_reaper
            .store(changed.close_with_reaper, Ordering::Relaxed);
        **config = changed;
        if midi_differs {
            (self.midi_changed)(&config.midi);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
