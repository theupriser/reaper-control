//! A named background thread that does one job at a fixed interval and stops promptly when asked.

use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

/// A handle to the running thread.
pub struct PeriodicThread {
    name: &'static str,
    stop: Sender<()>,
    handle: JoinHandle<()>,
}

impl PeriodicThread {
    /// Runs `work` every `interval` until [`stop`](Self::stop). A thread that cannot start is logged and
    /// left out: the app never stops for a helper.
    pub fn start(
        name: &'static str,
        interval: Duration,
        mut work: impl FnMut() + Send + 'static,
    ) -> Option<Self> {
        let (stop, stopped) = channel::<()>();
        let spawned = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || {
                while stopped
                    .recv_timeout(interval)
                    .is_err_and(|error| matches!(error, std::sync::mpsc::RecvTimeoutError::Timeout))
                {
                    work();
                }
            });
        match spawned {
            Ok(handle) => Some(Self { name, stop, handle }),
            Err(error) => {
                tracing::error!(%error, thread = name, "background thread failed to start");
                None
            }
        }
    }

    /// Asks the thread to end and waits for it. Returns `false` when it had panicked.
    pub fn stop(self) -> bool {
        let _ = self.stop.send(());
        let ended = self.handle.join().is_ok();
        tracing::info!(thread = self.name, ended, "background thread stopped");
        ended
    }
}
