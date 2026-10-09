//! A config repository in memory.

use std::sync::Mutex;

use crate::app_config::AppConfig;
use crate::config_error::ConfigError;
use crate::config_repository::ConfigRepository;

/// For tests: keeps the last saved config, and can be told to refuse writes.
#[derive(Debug, Default)]
pub struct FakeConfigRepository {
    // Boxed: a config is 120 bytes.
    saved: Mutex<Option<Box<AppConfig>>>,
    refuse_writes: Mutex<bool>,
}

impl FakeConfigRepository {
    /// Makes every following `save` fail like a full disk would.
    pub fn refuse_writes(&self) {
        if let Ok(mut refuse) = self.refuse_writes.lock() {
            *refuse = true;
        }
    }
}

impl ConfigRepository for FakeConfigRepository {
    fn exists(&self) -> bool {
        self.saved.lock().is_ok_and(|saved| saved.is_some())
    }

    fn load(&self) -> Result<AppConfig, ConfigError> {
        Ok(self
            .saved
            .lock()
            .ok()
            .and_then(|saved| saved.as_deref().cloned())
            .unwrap_or_default())
    }

    fn save(&self, config: &AppConfig) -> Result<(), ConfigError> {
        config.validate()?;
        if self.refuse_writes.lock().is_ok_and(|refuse| *refuse) {
            return Err(std::io::Error::other("the disk is full").into());
        }
        if let Ok(mut saved) = self.saved.lock() {
            *saved = Some(Box::new(config.clone()));
        }
        Ok(())
    }
}
