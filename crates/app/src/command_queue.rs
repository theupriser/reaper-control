//! The rules for commands in flight: no rapid repeats, a limit, a deadline.

use std::sync::Arc;
use std::time::Duration;

use protocol::Command;

use crate::clock::Clock;
use crate::pending_command::PendingCommand;
use crate::queue_rejection::QueueRejection;
use crate::queue_settings::QueueSettings;

/// Remembers what was sent and is still unanswered. It decides; the bus acts.
pub struct CommandQueue {
    clock: Arc<dyn Clock>,
    settings: QueueSettings,
    pending: Vec<PendingCommand>,
    last: Option<(Command, Duration)>,
}

impl CommandQueue {
    /// An empty queue.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, settings: QueueSettings) -> Self {
        Self {
            clock,
            settings,
            pending: Vec::new(),
            last: None,
        }
    }

    /// Uses these limits from now on; commands already waiting keep waiting.
    pub fn apply(&mut self, settings: QueueSettings) {
        self.settings = settings;
    }

    /// Whether `command` may go out now.
    pub fn admit(&self, command: &Command) -> Result<(), QueueRejection> {
        let now = self.clock.now();
        if let Some((previous, at)) = &self.last
            && previous == command
            && now.saturating_sub(*at) < self.settings.repeat_window
        {
            return Err(QueueRejection::Repeat);
        }
        if self.pending.len() >= self.settings.capacity {
            return Err(QueueRejection::Full);
        }
        Ok(())
    }

    /// Records that `command` went out under the link's `id`.
    pub fn track(&mut self, id: u64, command: Command) {
        let now = self.clock.now();
        self.last = Some((command.clone(), now));
        self.pending.push(PendingCommand {
            id,
            command,
            sent_at: now,
        });
    }

    /// The extension answered `id`; false when it was not waiting (late or unknown).
    pub fn acknowledge(&mut self, id: u64) -> bool {
        let before = self.pending.len();
        self.pending.retain(|pending| pending.id != id);
        self.pending.len() != before
    }

    /// Removes and returns the commands that waited longer than the timeout, oldest first.
    pub fn expire(&mut self) -> Vec<PendingCommand> {
        let now = self.clock.now();
        let timeout = self.settings.timeout;
        let (expired, waiting) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|pending| now.saturating_sub(pending.sent_at) >= timeout);
        self.pending = waiting;
        expired
    }

    /// How many commands wait for an answer.
    #[must_use]
    pub fn waiting(&self) -> usize {
        self.pending.len()
    }
}

impl std::fmt::Debug for CommandQueue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CommandQueue")
            .field("waiting", &self.pending.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests;
