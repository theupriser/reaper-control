//! Closing the app when REAPER quits cleanly.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use app_core::{LinkCause, LinkHealth};

use crate::health::HealthMonitor;

/// Closes the app when the link ends after REAPER said it is quitting. The extension sends that
/// message from its exit hook, so a crash, a kill or a stalled REAPER never counts: the app stays
/// open and shows the problem. Does nothing unless the person switched it on.
pub struct CloseWithReaper {
    enabled: Arc<AtomicBool>,
    health: Arc<HealthMonitor>,
    close: Box<dyn Fn() + Send + Sync>,
}

impl CloseWithReaper {
    /// `enabled` follows the setting, `health` knows what REAPER announced and `close` ends the
    /// app.
    pub fn new(
        enabled: Arc<AtomicBool>,
        health: Arc<HealthMonitor>,
        close: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        Self {
            enabled,
            health,
            close,
        }
    }

    /// Looks at every change of the link's health.
    pub fn observe(&self, health: &LinkHealth) {
        let ended = matches!(
            health,
            LinkHealth::Lost | LinkHealth::Dead(LinkCause::ReaperNotRunning)
        );
        if ended && self.enabled.load(Ordering::Relaxed) && self.health.reaper_is_closing() {
            tracing::info!("REAPER quit cleanly, closing the app as the settings say");
            (self.close)();
        }
    }
}

#[cfg(test)]
mod tests;
