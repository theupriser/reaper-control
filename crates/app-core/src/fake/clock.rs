//! A clock that only moves when a test says so.

use std::sync::Mutex;
use std::time::Duration;

use crate::clock::Clock;

/// A clock for tests: it stands still until `advance` is called.
#[derive(Debug, Default)]
pub struct FakeClock {
    now: Mutex<Duration>,
}

impl FakeClock {
    /// Moves time forward.
    pub fn advance(&self, by: Duration) {
        if let Ok(mut now) = self.now.lock() {
            *now += by;
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Duration {
        self.now.lock().map(|now| *now).unwrap_or_default()
    }
}
