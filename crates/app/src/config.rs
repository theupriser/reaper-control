//! Config.

mod app;
mod appearance;
mod chaos;
mod error;
mod location;
mod log;
mod migration;
mod repository;
mod settings;
mod store;

pub use app::{AppConfig, SCHEMA_VERSION};
pub use appearance::{AppearanceConfig, DENSITIES, THEMES, TOUCH_SIZES};
pub use chaos::ChaosSettings;
pub use error::ConfigError;
pub use location::{
    config_file, diagnostics_directory, legacy_config_file, legacy_setlists_directory,
    log_directory, mirror_directory,
};
pub use log::{LEVELS, LogConfig};
pub use migration::migrate;
pub use repository::ConfigRepository;
pub use settings::SettingsService;
pub use store::ConfigStore;
