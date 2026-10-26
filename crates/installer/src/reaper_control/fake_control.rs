//! A REAPER control that tests control.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::FakeProcessCheck;
use crate::ReaperControl;
use crate::ReaperInstall;

/// Flips the shared fake process check when asked to quit or launch, and remembers the calls.
#[derive(Debug)]
pub struct FakeReaperControl {
    processes: Arc<FakeProcessCheck>,
    closes_when_asked: AtomicBool,
    calls: Mutex<Vec<&'static str>>,
}

impl FakeReaperControl {
    /// A control over `processes`; REAPER closes when asked.
    #[must_use]
    pub fn new(processes: Arc<FakeProcessCheck>) -> Self {
        Self {
            processes,
            closes_when_asked: AtomicBool::new(true),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Whether REAPER obeys a quit (false: the person answered "Cancel" to a save prompt).
    pub fn set_closes_when_asked(&self, closes: bool) {
        self.closes_when_asked.store(closes, Ordering::SeqCst);
    }

    /// `"quit"` and `"launch"` in the order they were asked.
    #[must_use]
    pub fn calls(&self) -> Vec<&'static str> {
        self.calls
            .lock()
            .map(|calls| calls.clone())
            .unwrap_or_default()
    }

    fn record(&self, call: &'static str) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call);
        }
    }
}

impl ReaperControl for FakeReaperControl {
    fn quit(&self) -> Result<(), String> {
        self.record("quit");
        if self.closes_when_asked.load(Ordering::SeqCst) {
            self.processes.set_running(false);
        }
        Ok(())
    }

    fn launch(&self, _install: &ReaperInstall) -> Result<(), String> {
        self.record("launch");
        self.processes.set_running(true);
        Ok(())
    }
}
