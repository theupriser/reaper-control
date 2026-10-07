/// A command was not sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SendError {
    /// No live connection. The command is dropped, not queued.
    #[error("not connected to the extension")]
    NotConnected,
}
