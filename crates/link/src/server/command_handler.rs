use protocol::Command;
use protocol::message::Outcome;

/// What the extension does with a command. Called on a connection thread: hand the work to
/// the right thread and return at once.
pub trait CommandHandler: Send + Sync + 'static {
    /// Decide and report; never block.
    fn handle(&self, command: Command) -> Outcome;
}
