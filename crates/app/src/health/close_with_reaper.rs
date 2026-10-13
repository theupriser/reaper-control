//! Closing the app when REAPER quits cleanly.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use app_core::{LinkCause, LinkHealth};

/// Closes the app once a link that worked has ended because REAPER quit. It wants proof of a clean
/// exit: the extension's `running` file was there while connected and is gone now. A crash leaves
/// it behind, and then the app stays open to show what happened; a file that was never seen (a
/// different folder, an older extension) counts as no proof. Does nothing unless the person
/// switched it on, and never closes an app that was not connected.
pub struct CloseWithReaper {
    enabled: Arc<AtomicBool>,
    running_file: PathBuf,
    was_connected: AtomicBool,
    marker_seen: AtomicBool,
    close: Box<dyn Fn() + Send + Sync>,
}

impl CloseWithReaper {
    /// `enabled` follows the setting, `running_file` is the extension's safe-mode marker and
    /// `close` ends the app.
    pub fn new(
        enabled: Arc<AtomicBool>,
        running_file: PathBuf,
        close: Box<dyn Fn() + Send + Sync>,
    ) -> Self {
        Self {
            enabled,
            running_file,
            was_connected: AtomicBool::new(false),
            marker_seen: AtomicBool::new(false),
            close,
        }
    }

    /// Looks at every change of the link's health.
    pub fn observe(&self, health: &LinkHealth) {
        match health {
            LinkHealth::Connected | LinkHealth::Degraded => {
                self.was_connected.store(true, Ordering::Relaxed);
                if self.running_file.exists() {
                    self.marker_seen.store(true, Ordering::Relaxed);
                }
            }
            LinkHealth::Dead(LinkCause::ReaperNotRunning)
                if self.was_connected.load(Ordering::Relaxed)
                    && self.marker_seen.load(Ordering::Relaxed)
                    && self.enabled.load(Ordering::Relaxed)
                    && !self.running_file.exists() =>
            {
                tracing::info!("REAPER quit cleanly, closing the app as the settings say");
                (self.close)();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
