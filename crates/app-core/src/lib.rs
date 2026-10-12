//! The core of the app: commands and the bus they go through, app events, the configuration,
//! intents, the clock and the doubles for them. No Tauri, no REAPER, no device access.

mod clock;
mod command;
mod config;
mod control;
mod event;
mod fake;
mod intent;
mod link;

pub use clock::{Clock, SystemClock};
pub use command::{
    CommandBus, CommandQueue, CommandRefusal, DispatchError, Driver, DriverError, LatencyStats,
    MeteredDriver, PendingCommand, QueueConfig, QueueRejection, QueueSettings, check,
};
pub use config::{
    AppConfig, AppearanceConfig, ChaosSettings, ConfigError, ConfigRepository, ConfigStore,
    DENSITIES, LEVELS, LogConfig, MidiConfig, SCHEMA_VERSION, SettingsService, THEMES, TOUCH_SIZES,
    config_file, diagnostics_directory, legacy_config_file, legacy_setlists_directory,
    log_directory, migrate, mirror_directory,
};
pub use control::{ControlBinding, ControlSource};
pub use event::{AppEvent, AppFault, EventBus, link_problem, log_event, notice_for_event};
pub use fake::{FakeClock, FakeConfigRepository, FakeDriver};
pub use intent::{Intent, IntentDispatcher, IntentError, IntentRefusal, IntentTranslator};
pub use link::{LinkCause, LinkHealth, performance_runs};
