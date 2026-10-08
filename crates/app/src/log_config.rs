//! The log setting as stored in the config file.

use serde::{Deserialize, Serialize};

/// The levels the Settings screen offers, quietest first.
pub const LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];

/// How much the app writes to its log; applies at the next start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// One of [`LEVELS`].
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
        }
    }
}

impl LogConfig {
    /// The value outside its allowed set, as a message.
    #[must_use]
    pub fn problem(&self) -> Option<String> {
        (!LEVELS.contains(&self.level.as_str())).then(|| {
            format!(
                "log level {:?} is not one of {}",
                self.level,
                LEVELS.join(", ")
            )
        })
    }
}
