//! A process check that tests control.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::process_check::ProcessCheck;

/// Answers whatever the test last set; REAPER is not running at first.
#[derive(Debug, Default)]
pub struct FakeProcessCheck {
    running: AtomicBool,
}

impl FakeProcessCheck {
    /// Sets whether REAPER counts as running.
    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
    }
}

impl ProcessCheck for FakeProcessCheck {
    fn reaper_is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
