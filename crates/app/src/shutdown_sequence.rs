//! The steps that end the app, run in the order they were added. A step that panics is logged and the rest still run.

use std::panic::{AssertUnwindSafe, catch_unwind};

type Step = Box<dyn FnOnce() + Send>;

/// An ordered list of named steps.
#[derive(Default)]
pub struct ShutdownSequence {
    steps: Vec<(&'static str, Step)>,
}

impl ShutdownSequence {
    /// Appends a step; it runs after the ones added before it.
    pub fn add(&mut self, name: &'static str, step: impl FnOnce() + Send + 'static) {
        self.steps.push((name, Box::new(step)));
    }

    /// Runs every step once and returns the names of the steps that failed.
    #[must_use]
    pub fn run(self) -> Vec<&'static str> {
        let mut failed = Vec::new();
        for (name, step) in self.steps {
            tracing::info!(step = name, "shutting down");
            if catch_unwind(AssertUnwindSafe(step)).is_err() {
                tracing::error!(step = name, "shutdown step failed");
                failed.push(name);
            }
        }
        failed
    }
}
