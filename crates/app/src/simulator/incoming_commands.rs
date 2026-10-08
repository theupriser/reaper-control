//! The commands sent to the simulator that it has not carried out yet.

use std::sync::Mutex;

use protocol::Command;

/// Commands wait here, with the id their answer will carry, until the simulator's next step.
#[derive(Debug, Default)]
pub(super) struct IncomingCommands {
    waiting: Mutex<(u64, Vec<(u64, Command)>)>,
}

impl IncomingCommands {
    /// Queues `command` and returns its id; `None` when the queue is unusable.
    pub(super) fn push(&self, command: Command) -> Option<u64> {
        let mut waiting = self.waiting.lock().ok()?;
        waiting.0 += 1;
        let id = waiting.0;
        waiting.1.push((id, command));
        Some(id)
    }

    /// Everything queued so far, oldest first.
    pub(super) fn take(&self) -> Vec<(u64, Command)> {
        self.waiting
            .lock()
            .map(|mut waiting| std::mem::take(&mut waiting.1))
            .unwrap_or_default()
    }
}
