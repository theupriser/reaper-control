use std::sync::Mutex;
use std::sync::mpsc::Sender;

use link::CommandHandler;
use protocol::Command;
use protocol::message::Outcome;

/// Runs on the link's connection threads: puts the command on the queue and answers at once. REAPER's
/// main thread takes the commands off the queue on its next tick.
#[derive(Debug)]
pub(super) struct QueuedCommands {
    sender: Mutex<Sender<Command>>,
}

impl QueuedCommands {
    pub(super) fn new(sender: Sender<Command>) -> Self {
        Self {
            sender: Mutex::new(sender),
        }
    }
}

impl CommandHandler for QueuedCommands {
    fn handle(&self, command: Command) -> Outcome {
        let sent = self
            .sender
            .lock()
            .map(|sender| sender.send(command).is_ok())
            .unwrap_or(false);
        if sent {
            Outcome::Done
        } else {
            Outcome::Rejected {
                reason: "the extension is not taking commands".into(),
            }
        }
    }
}
