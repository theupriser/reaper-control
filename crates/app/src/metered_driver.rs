//! A driver that measures how long the extension takes to answer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use protocol::Command;

use crate::app_event::AppEvent;
use crate::clock::Clock;
use crate::driver::Driver;
use crate::driver_error::DriverError;
use crate::event_bus::EventBus;
use crate::latency_stats::LatencyStats;

#[derive(Default)]
struct Measured {
    waiting: HashMap<u64, Duration>,
    stats: LatencyStats,
}

/// Wraps a driver: remembers when each command went out and, when its answer is announced on
/// the bus, records the delay and logs it.
pub struct MeteredDriver {
    inner: Arc<dyn Driver>,
    clock: Arc<dyn Clock>,
    measured: Arc<Mutex<Measured>>,
}

impl MeteredDriver {
    /// Measures `inner`; answers are heard on `events`.
    #[must_use]
    pub fn new(inner: Arc<dyn Driver>, clock: Arc<dyn Clock>, events: &EventBus) -> Self {
        let measured = Arc::new(Mutex::new(Measured::default()));
        let hearing = Arc::clone(&measured);
        let timer = Arc::clone(&clock);
        events.subscribe(move |event| {
            let (AppEvent::CommandAcknowledged { id } | AppEvent::ExtensionRefused { id, .. }) =
                event
            else {
                return;
            };
            let Ok(mut measured) = hearing.lock() else {
                return;
            };
            let Some(sent_at) = measured.waiting.remove(id) else {
                return;
            };
            let delay = timer.now().saturating_sub(sent_at);
            measured.stats.answered += 1;
            measured.stats.last = delay;
            measured.stats.slowest = measured.stats.slowest.max(delay);
            tracing::debug!(id, delay_ms = delay.as_millis() as u64, "command answered");
        });
        Self {
            inner,
            clock,
            measured,
        }
    }

    /// What was measured so far.
    #[must_use]
    pub fn stats(&self) -> LatencyStats {
        self.measured
            .lock()
            .map(|measured| measured.stats)
            .unwrap_or_default()
    }
}

impl Driver for MeteredDriver {
    fn send(&self, command: Command) -> Result<u64, DriverError> {
        let id = self.inner.send(command)?;
        if let Ok(mut measured) = self.measured.lock() {
            measured.waiting.insert(id, self.clock.now());
        }
        Ok(id)
    }
}

#[cfg(test)]
mod tests;
