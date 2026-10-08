use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::log::Log;

/// Set once when anything inside the extension panics. From then on the extension does nothing
/// more but stays loaded, so REAPER keeps running (SPEC S-9.1).
#[derive(Debug, Default)]
pub struct Fault {
    faulted: AtomicBool,
}

impl Fault {
    /// A fault state that has not tripped.
    pub const fn new() -> Self {
        Self {
            faulted: AtomicBool::new(false),
        }
    }

    /// True once a panic was caught or reported.
    pub fn is_faulted(&self) -> bool {
        self.faulted.load(Ordering::Relaxed)
    }

    /// Marks the extension as faulted.
    pub fn trip(&self) {
        self.faulted.store(true, Ordering::Relaxed);
    }

    /// Runs `work` unless already faulted. A panic is caught, trips the fault and gives `None`,
    /// so it never reaches REAPER.
    pub fn guard<T>(&self, work: impl FnOnce() -> T) -> Option<T> {
        if self.is_faulted() {
            return None;
        }
        match catch_unwind(AssertUnwindSafe(work)) {
            Ok(value) => Some(value),
            Err(_) => {
                self.trip();
                None
            }
        }
    }

    /// Makes every panic anywhere in the process, also on our own threads, trip this fault and
    /// log where it happened (ADR-009).
    pub fn install_panic_hook(&'static self, log: &'static Log) {
        std::panic::set_hook(Box::new(move |info| {
            self.trip();
            log.line(&format!("PANIC, extension faulted: {info}"));
        }));
    }
}

#[cfg(test)]
mod tests;
