//! A command that went out and has not been answered yet.

use std::time::Duration;

use protocol::Command;

/// A sent command waiting for the extension's answer.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCommand {
    /// The id the link gave the command.
    pub id: u64,
    /// What was sent.
    pub command: Command,
    /// When it was sent, on the queue's clock.
    pub sent_at: Duration,
}
