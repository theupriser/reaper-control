//! Why the queue did not take a command.

/// Why the queue refused to let a command through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueRejection {
    /// The same command was sent a moment ago.
    Repeat,
    /// Too many commands are waiting for an answer.
    Full,
}
