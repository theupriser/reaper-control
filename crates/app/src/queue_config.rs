//! The command queue limits as stored in the config file.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::queue_settings::QueueSettings;

/// The stored command queue limits; every field has the default of [`QueueSettings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueConfig {
    /// Milliseconds inside which an identical command counts as a repeat (0 to 5000).
    pub repeat_window_milliseconds: u64,
    /// Milliseconds after which an unanswered command is reported (500 to 60000).
    pub timeout_milliseconds: u64,
    /// How many commands may wait for an answer (1 to 256).
    pub capacity: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        let defaults = QueueSettings::default();
        Self {
            repeat_window_milliseconds: millis(defaults.repeat_window),
            timeout_milliseconds: millis(defaults.timeout),
            capacity: defaults.capacity,
        }
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

impl QueueConfig {
    /// The first field outside its allowed range, as a message.
    #[must_use]
    pub fn problem(&self) -> Option<String> {
        if self.repeat_window_milliseconds > 5000 {
            return Some("queue.repeat_window_milliseconds must be at most 5000".into());
        }
        if !(500..=60_000).contains(&self.timeout_milliseconds) {
            return Some("queue.timeout_milliseconds must be between 500 and 60000".into());
        }
        if !(1..=256).contains(&self.capacity) {
            return Some("queue.capacity must be between 1 and 256".into());
        }
        None
    }

    /// The limits the command queue works with.
    #[must_use]
    pub fn settings(&self) -> QueueSettings {
        QueueSettings {
            repeat_window: Duration::from_millis(self.repeat_window_milliseconds),
            timeout: Duration::from_millis(self.timeout_milliseconds),
            capacity: self.capacity,
        }
    }
}
